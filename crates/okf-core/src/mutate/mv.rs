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
//!    from its new location so they still resolve to the same targets (bundle-relative
//!    links are location-independent and untouched).
//! 3. **Move.** The file is written at its new path and the old file removed.
//!
//! Every touched concept is rewritten through the validated concept writer, so unknown frontmatter values,
//! key order, and untouched body text survive; YAML comments/presentation may normalize.
use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, Parser, Tag};
use serde_yaml::Value;

use crate::bundle::loader::load_bundle;
use crate::error::{OkfError, Result};

use crate::model::concept::{Concept, ConceptId};
use crate::model::link::{classify, resolve_link, LinkKind};
use crate::ontology::load::try_load;
use crate::ontology::schema::Ontology;
use crate::query::selector::Selector;

use super::edit::{id_to_path, render_validated};
use super::transaction::{commit, FileChange};

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
    let ontology = try_load(root)?;
    mv_with_ontology(root, old_id, new_id, ontology.as_ref())
}

/// Move with an explicitly selected ontology, including a configured sidecar.
pub fn mv_with_ontology(
    root: &Path,
    old_id: &str,
    new_id: &str,
    ontology: Option<&Ontology>,
) -> Result<MvResult> {
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

    // 1. Rewrite inbound links in each referrer, staging changed concepts.
    let mut staged: Vec<Concept> = Vec::new();
    let mut rewritten: Vec<ConceptId> = Vec::new();
    for concept in &bundle.concepts {
        if concept.id == old {
            continue;
        }
        let mut concept = concept.clone();
        let from = concept.id.clone();
        let old_t = old.clone();
        let new_t = new.clone();
        let mut changed = apply_rewrite(&mut concept, ontology, &move |raw: &str| {
            if classify(raw) == LinkKind::External {
                return None;
            }
            if resolve_link(&from, raw) != old_t {
                return None;
            }
            let rebuilt = rewrite_link_string(&from, raw, &new_t);
            (rebuilt != raw).then_some(rebuilt)
        });
        changed |= rewrite_sources(&mut concept, &old, &new);
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
        apply_rewrite(&mut moved, ontology, &move |raw: &str| {
            if classify(raw) == LinkKind::External {
                return None;
            }
            // The target stays fixed (resolved from the OLD location); rebuild from the NEW.
            let target = resolve_link(&old_from, raw);
            let target = if target == old_from {
                new_from.clone()
            } else {
                target
            };
            let rebuilt = rewrite_link_string(&new_from, raw, &target);
            (rebuilt != raw).then_some(rebuilt)
        });
    }
    rewrite_sources(&mut moved, &old, &new);
    moved.id = new.clone();

    // Render the complete proposed state before publishing any file.
    let mut changes = Vec::new();
    for concept in staged.iter().chain(std::iter::once(&moved)) {
        let path = id_to_path(root, &concept.id)?;
        let before = if path.exists() {
            Some(std::fs::read(&path)?)
        } else {
            None
        };
        changes.push(FileChange {
            path: path.strip_prefix(root).unwrap().to_path_buf(),
            before,
            after: Some(render_validated(root, concept)?.into_bytes()),
        });
    }
    for (id, path, before) in structural_documents(root)? {
        let (after, changed) = rewrite_in_body(&before, &|raw| {
            (classify(raw) != LinkKind::External && resolve_link(&id, raw) == old)
                .then(|| rewrite_link_string(&id, raw, &new))
        });
        if changed {
            rewritten.push(id);
            changes.push(FileChange {
                path,
                before: Some(before.into_bytes()),
                after: Some(after.into_bytes()),
            });
        }
    }
    changes.push(FileChange {
        path: old_path.strip_prefix(root).unwrap().to_path_buf(),
        before: Some(std::fs::read(&old_path)?),
        after: None,
    });
    commit(root, changes)?;

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
    if let Some(definition) = concept
        .concept_type()
        .and_then(|name| ontology.and_then(|o| o.concepts.get(name)))
    {
        let mut metadata = Value::Mapping(
            concept
                .frontmatter
                .map
                .iter()
                .map(|(k, v)| (Value::String(k.clone()), v.clone()))
                .collect(),
        );
        for rule in definition.references.values() {
            if let Some(selector) = &rule.selector {
                if let Ok(selector) = Selector::parse(selector) {
                    changed |= rewrite_selected(&mut metadata, &selector.steps, rewrite);
                }
            }
        }
        if changed {
            if let Value::Mapping(map) = metadata {
                for (key, value) in map {
                    if let Value::String(key) = key {
                        concept.frontmatter.map.insert(key, value);
                    }
                }
            }
        }
    }
    let keys: Vec<String> = concept
        .concept_type()
        .and_then(|name| ontology.and_then(|o| o.concepts.get(name)))
        .map(|ct| {
            ct.references
                .iter()
                .filter(|(_, rule)| rule.selector.is_none())
                .map(|(key, _)| key.clone())
                .collect()
        })
        .unwrap_or_default();
    for key in keys {
        if let Some(value) = concept.frontmatter.map.get_mut(&key) {
            if rewrite_in_value(value, rewrite) {
                changed = true;
            }
        }
    }
    // Legacy untyped source references use the concept-link namespace. Typed
    // File, line-range, and markdown-heading resources use the bundle root.
    if let Some(Value::Sequence(sources)) = concept.frontmatter.map.get_mut("sources") {
        for source in sources {
            let Some(map) = source.as_mapping_mut() else {
                continue;
            };
            if !map.contains_key(Value::String("kind".into())) {
                if let Some(resource) = map.get_mut(Value::String("resource".into())) {
                    changed |= rewrite_in_value(resource, rewrite);
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
            if !s.trim().is_empty() {
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
/// replaced (after the closing label, leaving labels and titles alone). Edits are applied from the end so byte offsets stay valid.
fn rewrite_in_body<F>(body: &str, rewrite: &F) -> (String, bool)
where
    F: Fn(&str) -> Option<String>,
{
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    let parser = Parser::new(body);
    for (_, definition) in parser.reference_definitions().iter() {
        if let Some(new) = rewrite(&definition.dest) {
            let span = &body[definition.span.clone()];
            if let Some(colon) = span.find("]: ").or_else(|| span.find("]:")) {
                if let Some(offset) = span[colon + 2..].find(definition.dest.as_ref()) {
                    let start = definition.span.start + colon + 2 + offset;
                    edits.push((start, start + definition.dest.len(), new));
                }
            }
        }
    }
    for (event, range) in parser.into_offset_iter() {
        if let Event::Start(
            Tag::Link {
                dest_url,
                link_type: pulldown_cmark::LinkType::Inline,
                ..
            }
            | Tag::Image {
                dest_url,
                link_type: pulldown_cmark::LinkType::Inline,
                ..
            },
        ) = event
        {
            let dest = dest_url.to_string();
            if dest.is_empty() {
                continue;
            }
            if let Some(new) = rewrite(&dest) {
                let span = &body[range.clone()];
                if let Some(open) = span.rfind("](") {
                    let destination = &span[open + 2..];
                    if let Some(pos) = destination.find(dest.as_str()) {
                        let start = range.start + open + 2 + pos;
                        edits.push((start, start + dest.len(), new));
                    }
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

/// Structural navigation is authored Markdown and participates in mutation integrity.
pub(crate) fn structural_documents(root: &Path) -> Result<Vec<(ConceptId, PathBuf, String)>> {
    crate::bundle::walk::walk_markdown(root)?
        .into_iter()
        .filter(|path| {
            matches!(
                path.file_name().and_then(|s| s.to_str()),
                Some("index.md" | "log.md")
            )
        })
        .map(|path| {
            let body = std::fs::read_to_string(root.join(&path))?;
            Ok((
                ConceptId::from_relative(&path.with_extension("").to_string_lossy()),
                path,
                body,
            ))
        })
        .collect()
}

pub(crate) fn structural_referrers(root: &Path, target: &ConceptId) -> Result<Vec<ConceptId>> {
    Ok(structural_documents(root)?
        .into_iter()
        .filter_map(|(id, _, body)| {
            let (_, changed) = rewrite_in_body(&body, &|raw| {
                (classify(raw) != LinkKind::External && resolve_link(&id, raw) == *target)
                    .then(|| raw.to_string())
            });
            changed.then_some(id)
        })
        .collect())
}

// File/text source resources are bundle-root paths, unlike concept-relative links.
fn rewrite_sources(concept: &mut Concept, old: &ConceptId, new: &ConceptId) -> bool {
    let Some(Value::Sequence(sources)) = concept.frontmatter.map.get_mut("sources") else {
        return false;
    };
    let mut changed = false;
    for source in sources {
        let Some(map) = source.as_mapping_mut() else {
            continue;
        };
        if !matches!(
            map.get(Value::String("kind".into()))
                .and_then(Value::as_str),
            Some("file" | "line-range" | "markdown-heading")
        ) {
            continue;
        }
        let Some(Value::String(raw)) = map.get_mut(Value::String("resource".into())) else {
            continue;
        };
        let from = ConceptId::from_relative("_root");
        if classify(raw) != LinkKind::External && resolve_link(&from, raw) == *old {
            *raw = rewrite_link_string(&from, raw, new);
            changed = true;
        }
    }
    changed
}

fn rewrite_selected<F: Fn(&str) -> Option<String>>(
    value: &mut Value,
    steps: &[crate::query::selector::Step],
    rewrite: &F,
) -> bool {
    use crate::query::selector::Step;
    match steps.split_first() {
        None => rewrite_in_value(value, rewrite),
        Some((Step::Key(key), rest)) => value
            .as_mapping_mut()
            .and_then(|m| m.get_mut(Value::String(key.clone())))
            .is_some_and(|v| rewrite_selected(v, rest, rewrite)),
        Some((Step::Each, rest)) => {
            let mut changed = false;
            if let Some(items) = value.as_sequence_mut() {
                for item in items {
                    changed |= rewrite_selected(item, rest, rewrite);
                }
            }
            changed
        }
    }
}

/// Retarget references without moving the original record (for replacement/split plans).
/// `referrers` optionally restricts the authored records whose links are reassigned.
pub fn retarget(
    root: &Path,
    old: &str,
    new: &str,
    ontology: Option<&Ontology>,
    referrers: Option<&[ConceptId]>,
) -> Result<Vec<ConceptId>> {
    let old = ConceptId::parse(old)?;
    let new = ConceptId::parse(new)?;
    let bundle = load_bundle(root)?;
    let mut changes = Vec::new();
    let mut rewritten = Vec::new();
    for mut concept in bundle.concepts {
        if referrers.is_some_and(|ids| !ids.contains(&concept.id)) {
            continue;
        }
        let from = concept.id.clone();
        let changed = apply_rewrite(&mut concept, ontology, &|raw| {
            (classify(raw) != LinkKind::External && resolve_link(&from, raw) == old)
                .then(|| rewrite_link_string(&from, raw, &new))
        }) | rewrite_sources(&mut concept, &old, &new);
        if changed {
            let path = id_to_path(root, &concept.id)?;
            changes.push(FileChange {
                path: path.strip_prefix(root).unwrap().to_path_buf(),
                before: Some(std::fs::read(path)?),
                after: Some(render_validated(root, &concept)?.into_bytes()),
            });
            rewritten.push(concept.id);
        }
    }
    for (id, path, before) in structural_documents(root)? {
        if referrers.is_some_and(|ids| !ids.contains(&id)) {
            continue;
        }
        let (after, changed) = rewrite_in_body(&before, &|raw| {
            (classify(raw) != LinkKind::External && resolve_link(&id, raw) == old)
                .then(|| rewrite_link_string(&id, raw, &new))
        });
        if changed {
            rewritten.push(id);
            changes.push(FileChange {
                path,
                before: Some(before.into_bytes()),
                after: Some(after.into_bytes()),
            });
        }
    }
    commit(root, changes)?;
    Ok(rewritten)
}

/// Referrers from the same occurrence rules used by rewriting, including bare fields.
pub(crate) fn references_target(
    concept: &Concept,
    ontology: Option<&Ontology>,
    target: &ConceptId,
) -> bool {
    let mut copy = concept.clone();
    let from = concept.id.clone();
    apply_rewrite(&mut copy, ontology, &|raw| {
        (classify(raw) != LinkKind::External && resolve_link(&from, raw) == *target)
            .then(|| raw.to_owned())
    }) | rewrite_sources(&mut copy, target, target)
}
