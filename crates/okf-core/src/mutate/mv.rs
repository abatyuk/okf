//! Rename a concept and rewrite every inbound link.
//!
//! A concept id *is* its file path, so a naive rename silently breaks every reference to it.
//! `mv` prevents that:
//!
//! 1. **Inbound rewrite (the point).** It finds all backlinks via the link graph and, in each
//!    referrer, rewrites exactly the link strings that resolve to the old id — in *both*
//!    frontmatter reference fields and body markdown links — to point at the new id. The
//!    rewrite **preserves each link's style**: a bundle-relative `/a/b.md` stays
//!    bundle-relative, a bare `a/b` stays bare, and a relative `../a/b.md` is recomputed
//!    relative to the referrer's directory. Any `#fragment`/`?query` and `.md` suffix are kept.
//! 2. **Outbound rebase.** The moved file's own *relative* links (`./`, `../`) are recomputed
//!    from its new location so they still resolve to the same targets (bundle-relative/bare
//!    links are location-independent and untouched).
//! 3. **Move.** The file is written at its new path and the old file removed.
//!
//! Every touched file is rewritten through [`write_concept`], so unknown frontmatter values,
//! key order, and untouched body text survive; YAML comments/presentation may normalize.
use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, Parser, Tag};
use serde_yaml::Value;

use crate::bundle::loader::load_bundle;
use crate::error::{OkfError, Result};
use crate::graph::backlinks::backlinks_of;
use crate::model::concept::{Concept, ConceptId};
use crate::model::link::{classify, resolve_link, LinkKind};
use crate::ontology::load::try_load;
use crate::ontology::schema::Ontology;

use super::edit::{id_to_path, save_concept};

/// What `mv` changed.
#[derive(Debug, Clone)]
pub struct MvResult {
    pub from: ConceptId,
    pub to: ConceptId,
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    /// Referrers whose inbound links were rewritten.
    pub rewritten: Vec<ConceptId>,
}

/// Move/rename the concept `old_id` to `new_id` in the bundle at `root`, rewriting every
/// inbound link and rebasing the moved file's own relative links.
pub fn mv(root: &Path, old_id: &str, new_id: &str) -> Result<MvResult> {
    let old = ConceptId::parse(old_id)?;
    let new = ConceptId::parse(new_id)?;
    if old == new {
        return Err(OkfError::Usage(
            "mv: old and new ids are the same".to_string(),
        ));
    }

    let old_path = id_to_path(root, &old)?;
    let new_path = id_to_path(root, &new)?;
    if !old_path.exists() {
        return Err(OkfError::Usage(format!("mv: no concept at {old}")));
    }
    if new_path.exists() {
        return Err(OkfError::Usage(format!("mv: target {new} already exists")));
    }

    let bundle = load_bundle(root)?;
    let ontology = try_load(root)?;
    let referrers = backlinks_of(&bundle, ontology.as_ref(), &old.0);

    // 1. Rewrite inbound links in each referrer, staging changed concepts.
    let mut staged: Vec<Concept> = Vec::new();
    let mut rewritten: Vec<ConceptId> = Vec::new();
    for rid in &referrers {
        let Some(concept) = bundle.get(&rid.0) else {
            continue;
        };
        let mut concept = concept.clone();
        let from = concept.id.clone();
        let old_t = old.clone();
        let new_t = new.clone();
        let changed = apply_rewrite(&mut concept, ontology.as_ref(), &move |raw: &str| {
            if classify(raw) == LinkKind::External {
                return None;
            }
            if resolve_link(&from, raw) != old_t {
                return None;
            }
            let rebuilt = rewrite_link_string(&from, raw, &new_t);
            (rebuilt != raw).then_some(rebuilt)
        });
        if changed {
            rewritten.push(concept.id.clone());
            staged.push(concept);
        }
    }

    // 2. Rebase the moved concept's own relative links to its new home.
    let mut moved = bundle
        .get(&old.0)
        .cloned()
        .ok_or_else(|| OkfError::Internal(format!("moved concept vanished: {old}")))?;
    {
        let old_from = old.clone();
        let new_from = new.clone();
        apply_rewrite(&mut moved, ontology.as_ref(), &move |raw: &str| {
            if classify(raw) == LinkKind::External {
                return None;
            }
            // The target stays fixed (resolved from the OLD location); rebuild from the NEW.
            let target = resolve_link(&old_from, raw);
            let rebuilt = rewrite_link_string(&new_from, raw, &target);
            (rebuilt != raw).then_some(rebuilt)
        });
    }
    moved.id = new.clone();

    // 3. Commit: write referrers, write the moved file at its new path, remove the old file.
    for concept in &staged {
        save_concept(root, concept)?;
    }
    save_concept(root, &moved)?;
    std::fs::remove_file(&old_path)
        .map_err(|e| OkfError::Io(format!("{}: {e}", old_path.display())))?;

    Ok(MvResult {
        from: old,
        to: new,
        old_path,
        new_path,
        rewritten,
    })
}

/// Apply a link-string rewrite closure over a concept's frontmatter reference fields and body
/// markdown links. Returns whether anything changed.
fn apply_rewrite<F>(concept: &mut Concept, ontology: Option<&Ontology>, rewrite: &F) -> bool
where
    F: Fn(&str) -> Option<String>,
{
    let mut changed = false;
    let keys: Vec<String> = concept
        .concept_type()
        .and_then(|name| ontology.and_then(|o| o.concepts.get(name)))
        .map(|ct| ct.references.keys().cloned().collect())
        .unwrap_or_default();
    for key in keys {
        if let Some(value) = concept.frontmatter.map.get_mut(&key) {
            if rewrite_in_value(value, rewrite) {
                changed = true;
            }
        }
    }
    // Standard OKF path-valued fields are understood without an ontology.
    if let Some(Value::Sequence(sources)) = concept.frontmatter.map.get_mut("sources") {
        for source in sources {
            if let Some(resource) = source
                .as_mapping_mut()
                .and_then(|m| m.get_mut(Value::String("resource".to_string())))
            {
                if rewrite_in_value(resource, rewrite) {
                    changed = true;
                }
            }
        }
    }
    if let Some(value) = concept.frontmatter.map.get_mut("computation") {
        if rewrite_in_value(value, rewrite) {
            changed = true;
        }
    }
    for family in ["executor", "attester"] {
        if let Some(resource) = concept
            .frontmatter
            .map
            .get_mut(family)
            .and_then(Value::as_mapping_mut)
            .and_then(|m| m.get_mut(Value::String("resource".to_string())))
        {
            if rewrite_in_value(resource, rewrite) {
                changed = true;
            }
        }
    }
    let (new_body, body_changed) = rewrite_in_body(&concept.body, rewrite);
    if body_changed {
        concept.body = new_body;
        changed = true;
    }
    changed
}

/// Rewrite link-shaped strings inside a frontmatter value (recursing into sequences).
fn rewrite_in_value<F>(value: &mut Value, rewrite: &F) -> bool
where
    F: Fn(&str) -> Option<String>,
{
    match value {
        Value::String(s) => {
            if looks_like_link(s) {
                if let Some(new) = rewrite(s) {
                    *s = new;
                    return true;
                }
            }
            false
        }
        Value::Sequence(seq) => {
            let mut changed = false;
            for item in seq.iter_mut() {
                if rewrite_in_value(item, rewrite) {
                    changed = true;
                }
            }
            changed
        }
        _ => false,
    }
}

/// Rewrite markdown link destinations in a body. Only the destination of each link is
/// replaced (the last occurrence within the link's span, so a `[dest](dest)` label is left
/// alone). Edits are applied from the end so byte offsets stay valid.
fn rewrite_in_body<F>(body: &str, rewrite: &F) -> (String, bool)
where
    F: Fn(&str) -> Option<String>,
{
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for (event, range) in Parser::new(body).into_offset_iter() {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            let dest = dest_url.to_string();
            if dest.is_empty() {
                continue;
            }
            if let Some(new) = rewrite(&dest) {
                let span = &body[range.clone()];
                if let Some(pos) = span.rfind(dest.as_str()) {
                    let start = range.start + pos;
                    edits.push((start, start + dest.len(), new));
                }
            }
        }
    }
    if edits.is_empty() {
        return (body.to_string(), false);
    }
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
    let mut out = body.to_string();
    for (start, end, replacement) in edits {
        out.replace_range(start..end, &replacement);
    }
    (out, true)
}

/// Rebuild a link string so it points at `target`, preserving the original link's *style*
/// (relative / bundle-relative / bare), its `.md` suffix, and any `#fragment`/`?query`.
fn rewrite_link_string(from: &ConceptId, raw: &str, target: &ConceptId) -> String {
    let (core, suffix) = split_suffix(raw);
    let has_md = core.ends_with(".md");
    let target_bare = target.0.trim_start_matches('/');

    let rebuilt_core = match classify(raw) {
        LinkKind::Relative => relative_path(parent_dir(&from.0), &target.0),
        LinkKind::BundleRelative => format!("/{target_bare}"),
        LinkKind::Bare => {
            let relative = relative_path(parent_dir(&from.0), &target.0);
            relative.strip_prefix("./").unwrap_or(&relative).to_string()
        }
        LinkKind::External => return raw.to_string(),
    };

    let mut out = rebuilt_core;
    if has_md {
        out.push_str(".md");
    }
    out.push_str(suffix);
    out
}

/// Split a raw link into its path core and the trailing `#fragment`/`?query` (kept verbatim,
/// including the delimiter). Whichever of `#`/`?` appears first begins the suffix.
fn split_suffix(raw: &str) -> (&str, &str) {
    match raw.find(['#', '?']) {
        Some(i) => (&raw[..i], &raw[i..]),
        None => (raw, ""),
    }
}

/// Directory portion of a concept id (`/tables/customers` → `/tables`, `/customers` → ``).
fn parent_dir(id: &str) -> &str {
    match id.rfind('/') {
        Some(i) => &id[..i],
        None => "",
    }
}

/// Build a `./`-anchored relative path from `from_dir` (a directory id like `/policies` or ``)
/// to the absolute concept id `target` (like `/tables/customers`).
fn relative_path(from_dir: &str, target: &str) -> String {
    let from: Vec<&str> = from_dir.split('/').filter(|s| !s.is_empty()).collect();
    let to: Vec<&str> = target.split('/').filter(|s| !s.is_empty()).collect();

    let mut common = 0;
    while common < from.len() && common < to.len() && from[common] == to[common] {
        common += 1;
    }

    let mut parts: Vec<String> = Vec::new();
    for _ in common..from.len() {
        parts.push("..".to_string());
    }
    for seg in &to[common..] {
        parts.push((*seg).to_string());
    }
    if parts.is_empty() {
        return ".".to_string();
    }
    let joined = parts.join("/");
    if joined.starts_with("..") {
        joined
    } else {
        format!("./{joined}")
    }
}

/// Heuristic (mirrors `model::link`): does a frontmatter scalar *look like* a concept link?
fn looks_like_link(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() || classify(t) == LinkKind::External {
        return false;
    }
    t.starts_with('/')
        || t.starts_with("./")
        || t.starts_with("../")
        || t.ends_with(".md")
        || (t.contains('/') && !t.chars().any(char::is_whitespace))
}
