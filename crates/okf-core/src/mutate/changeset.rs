//! Explicit, single-bundle change plans evaluated in isolation before publication.
use super::transaction::{self, FileChange};
use crate::error::{OkfError, Result};
use crate::model::concept::ConceptId;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeSet {
    pub version: u32,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Operation {
    Create {
        path: String,
        content: String,
    },
    Replace {
        path: String,
        content: String,
    },
    Edit {
        concept: String,
        #[serde(default)]
        patch: Vec<super::structured::PatchOperation>,
        body: Option<String>,
    },
    Move {
        from: String,
        to: String,
    },
    Retarget {
        from: String,
        to: String,
        within: Option<Vec<String>>,
    },
    Remove {
        path: String,
    },
    PutArtifact {
        path: String,
        content: String,
        #[serde(default)]
        replace: bool,
    },
}

pub struct Plan {
    pub base_digest: String,
    pub changes: Vec<FileChange>,
    before: BTreeMap<PathBuf, Vec<u8>>,
    interpretation: String,
}

fn usage(s: impl Into<String>) -> OkfError {
    OkfError::Usage(s.into())
}

fn snapshot(root: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut out = BTreeMap::new();
    for path in crate::bundle::walk::walk_files(root)? {
        if path == Path::new(transaction::LOCK) {
            continue;
        }
        // Preserve hidden authored files in the isolated view too. The walker excludes
        // symlinks and Git metadata; publication still forbids editing control paths.
        out.insert(path.clone(), std::fs::read(root.join(&path))?);
    }
    Ok(out)
}

fn digest(files: &BTreeMap<PathBuf, Vec<u8>>, interpretation: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(interpretation.as_bytes());
    for (path, content) in files {
        let path = path.to_string_lossy();
        hash.update((path.len() as u64).to_le_bytes());
        hash.update(path.as_bytes());
        hash.update((content.len() as u64).to_le_bytes());
        hash.update(content);
    }
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Result<Self> {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        loop {
            let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("okf-plan-{}-{n}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        }
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn concept_path(path: &str) -> Result<(ConceptId, PathBuf)> {
    let id = ConceptId::parse(path)?;
    let relative = PathBuf::from(format!("{}.md", id.0.trim_start_matches('/')));
    if matches!(
        relative.file_name().and_then(|s| s.to_str()),
        Some("index.md" | "log.md")
    ) {
        return Err(usage(
            "create/replace require a concept; use put-artifact for structural files",
        ));
    }
    Ok((id, relative))
}

fn write_document(root: &Path, path: &str, content: &str, replace: bool) -> Result<()> {
    let (id, rel) = concept_path(path)?;
    let target = transaction::checked_path(root, &rel)?;
    if target.exists() != replace {
        return Err(usage(format!(
            "{} requires {}: {path}",
            if replace { "replace" } else { "create" },
            if replace {
                "an existing concept"
            } else {
                "an absent concept"
            }
        )));
    }
    let concept = crate::parse::parse_concept(id.clone(), content)?;
    if concept.frontmatter.get("verified").is_some() {
        return Err(usage(
            "change-set content cannot supply verification; use verify after review",
        ));
    }
    if replace {
        let old = super::edit::load_concept(root, &id)?;
        let mut spec = super::edit::EditSpec::default();
        spec.unsets = old
            .frontmatter
            .map
            .keys()
            .filter(|k| !concept.frontmatter.map.contains_key(*k) && k.as_str() != "verified")
            .cloned()
            .collect();
        spec.structured.sets = concept.frontmatter.map.into_iter().collect();
        spec.set_body = Some(concept.body);
        super::edit::edit(root, &id.0, &spec)?;
    } else {
        super::edit::save_new_concept(root, &concept)?;
    }
    Ok(())
}

/// Missing local references include authored concept links and structural navigation.
fn missing(
    root: &Path,
    ontology: Option<&crate::ontology::schema::Ontology>,
) -> Result<BTreeSet<String>> {
    let bundle = crate::bundle::loader::load_bundle(root)?;
    let resolver = crate::query::artifact::ArtifactResolver::new(root);
    let mut out = BTreeSet::new();
    let mut check = |id: &str, raw: &str, bundle_relative: bool| {
        let from = if bundle_relative {
            "/_bundle_sources"
        } else {
            id
        };
        let resolved = resolver.resolve(Some(from), raw);
        if resolved.kind == crate::query::artifact::ArtifactKind::Missing && !bundle_relative {
            let target = crate::model::link::resolve_link(&ConceptId::from_relative(id), raw);
            if bundle.get(&target.0).is_some() {
                return;
            }
        }
        if matches!(
            resolved.kind,
            crate::query::artifact::ArtifactKind::Missing
                | crate::query::artifact::ArtifactKind::Blocked
        ) {
            out.insert(format!("{id}: {raw}"));
        }
    };
    for concept in &bundle.concepts {
        // Typed source resources do not share Markdown's document-relative namespace.
        let mut links = concept.clone();
        links.frontmatter.map.shift_remove("sources");
        for raw in crate::model::link::raw_links(&links, ontology) {
            check(&concept.id.0, &raw, false);
        }
        if let Some(definition) = concept
            .concept_type()
            .and_then(|name| ontology.and_then(|o| o.concepts.get(name)))
        {
            fn strings<'a>(value: &'a serde_yaml::Value, out: &mut Vec<&'a str>) {
                match value {
                    serde_yaml::Value::String(s) if !s.trim().is_empty() => out.push(s),
                    serde_yaml::Value::Sequence(items) => {
                        for item in items {
                            strings(item, out);
                        }
                    }
                    _ => {}
                }
            }
            for (key, rule) in &definition.references {
                if rule.selector.is_none() {
                    if let Some(value) = concept.frontmatter.get(key) {
                        let mut values = Vec::new();
                        strings(value, &mut values);
                        for raw in values {
                            check(&concept.id.0, raw, false);
                        }
                    }
                }
            }
        }
        for event in pulldown_cmark::Parser::new(&concept.body) {
            if let pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image { dest_url, .. }) = event
            {
                check(&concept.id.0, &dest_url, false);
            }
        }
        if let Some(sources) = concept
            .frontmatter
            .get("sources")
            .and_then(serde_yaml::Value::as_sequence)
        {
            for source in sources {
                let Some(raw) = source.get("resource").and_then(serde_yaml::Value::as_str) else {
                    continue;
                };
                match source.get("kind").and_then(serde_yaml::Value::as_str) {
                    Some("file" | "line-range" | "markdown-heading") => {
                        check(&concept.id.0, raw, true)
                    }
                    None => check(&concept.id.0, raw, false),
                    _ => {} // URL, Git, and producer-specific resources need other resolvers.
                }
            }
        }
        if let Some(raw) = concept
            .frontmatter
            .get("computation")
            .and_then(serde_yaml::Value::as_str)
        {
            check(&concept.id.0, raw, false);
        }
        for key in ["executor", "attester"] {
            if let Some(raw) = concept
                .frontmatter
                .get(key)
                .and_then(|v| v.get("resource"))
                .and_then(serde_yaml::Value::as_str)
            {
                check(&concept.id.0, raw, false);
            }
        }
    }
    for path in crate::bundle::walk::walk_markdown(root)? {
        if !matches!(
            path.file_name().and_then(|s| s.to_str()),
            Some("index.md" | "log.md")
        ) {
            continue;
        }
        let text = std::fs::read_to_string(root.join(&path))?;
        let id = format!("/{}", path.to_string_lossy().trim_end_matches(".md"));
        for event in pulldown_cmark::Parser::new(&text) {
            if let pulldown_cmark::Event::Start(
                pulldown_cmark::Tag::Link { dest_url, .. }
                | pulldown_cmark::Tag::Image { dest_url, .. },
            ) = event
            {
                check(&id, &dest_url, false);
            }
        }
    }
    Ok(out)
}

/// Active interpretation and selection files may live inside a bundle, but remain
/// tool configuration rather than artifacts available to coordinated content edits.
fn control_paths(root: &Path, ontology: Option<&Path>) -> Result<BTreeSet<PathBuf>> {
    let context = crate::bundle::context::current_context(Some(&root.to_string_lossy()), None)?;
    let mut paths = Vec::new();
    if let Some(config_path) = &context.config_path {
        paths.push(config_path.clone());
        let base = config_path.parent().unwrap_or(root);
        if let Some(catalog) = &context.config.catalog {
            paths.push(base.join(catalog));
        }
        let overrides = std::env::var("OKF_CATALOG_OVERRIDES")
            .ok()
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                context
                    .config
                    .catalog_overrides
                    .as_ref()
                    .map(|path| base.join(path))
            });
        if let Some(overrides) = overrides {
            paths.push(overrides);
        }
    }
    if let Some(ontology) = ontology {
        paths.push(ontology.to_path_buf());
    }
    let canonical_root = root.canonicalize()?;
    let mut relative = BTreeSet::new();
    for path in paths {
        let absolute = crate::bundle::catalog::absolute(&path)?;
        let canonical = absolute.canonicalize().unwrap_or(absolute);
        if let Ok(path) = canonical.strip_prefix(&canonical_root) {
            relative.insert(path.to_path_buf());
        }
    }
    Ok(relative)
}

/// Preview and apply share this exact preparation. No bundle files are changed here.
pub fn prepare(root: &Path, set: &ChangeSet) -> Result<Plan> {
    transaction::ensure_idle(root)?;
    if set.version != 1 || set.operations.is_empty() {
        return Err(usage(
            "change set requires version: 1 and nonempty operations",
        ));
    }
    let (settings, ontology) = crate::bundle::settings::load_for(root)?;
    let interpretation =
        serde_json::to_string(&settings).map_err(|e| OkfError::Internal(e.to_string()))?;
    let controls = control_paths(root, settings.ontology_path.as_deref())?;
    let before = snapshot(root)?;
    let base_digest = digest(&before, &interpretation);
    let workspace = Workspace::new()?;
    let stage = &workspace.0;
    for (path, content) in &before {
        let target = stage.join(path);
        std::fs::create_dir_all(target.parent().unwrap())?;
        std::fs::write(target, content)?;
    }
    let previous_missing = missing(stage, ontology.as_ref())?;
    for op in &set.operations {
        match op {
            Operation::Create { path, content } => write_document(stage, path, content, false)?,
            Operation::Replace { path, content } => write_document(stage, path, content, true)?,
            Operation::Edit {
                concept,
                patch,
                body,
            } => {
                let mut spec = super::edit::EditSpec::default();
                spec.structured.patch = patch.clone();
                spec.set_body = body.clone();
                super::edit::edit(stage, concept, &spec)?;
            }
            Operation::Move { from, to } => {
                super::mv::mv_with_ontology(stage, from, to, ontology.as_ref())?;
            }
            Operation::Retarget { from, to, within } => {
                let ids = within
                    .as_ref()
                    .map(|ids| {
                        ids.iter()
                            .map(|id| {
                                let relative = id
                                    .trim_start_matches('/')
                                    .strip_suffix(".md")
                                    .unwrap_or(id.trim_start_matches('/'));
                                let path = PathBuf::from(format!("{relative}.md"));
                                let target = transaction::checked_path(stage, &path)?;
                                if !target.is_file() {
                                    return Err(usage(format!(
                                        "retarget referrer does not exist: {id}"
                                    )));
                                }
                                Ok(ConceptId::from_relative(relative))
                            })
                            .collect::<Result<Vec<_>>>()
                    })
                    .transpose()?;
                super::mv::retarget(stage, from, to, ontology.as_ref(), ids.as_deref())?;
            }
            Operation::Remove { path } => {
                if matches!(
                    Path::new(path).file_name().and_then(|s| s.to_str()),
                    Some("okf.toml" | "ontology.yaml")
                ) {
                    return Err(usage("change sets cannot remove tool configuration"));
                }
                std::fs::remove_file(transaction::checked_path(stage, Path::new(path))?)?;
            }
            Operation::PutArtifact {
                path,
                content,
                replace,
            } => {
                crate::query::artifact::put_artifact(stage, path, content.as_bytes(), *replace)?;
            }
        }
    }
    let report = crate::check::validate::validate_bundle(stage)?;
    if !report.is_conformant() {
        return Err(usage(format!(
            "change set leaves nonconformant documents: {}",
            report
                .violations
                .iter()
                .map(|v| format!("{}: {}", v.file, v.message))
                .collect::<Vec<_>>()
                .join("; ")
        )));
    }
    let after_missing = missing(stage, ontology.as_ref())?;
    let introduced = after_missing
        .difference(&previous_missing)
        .cloned()
        .collect::<Vec<_>>();
    if !introduced.is_empty() {
        return Err(usage(format!(
            "change set introduces dangling/blocked references: {}",
            introduced.join("; ")
        )));
    }
    let after = snapshot(stage)?;
    for path in controls {
        if before.get(&path) != after.get(&path) {
            return Err(usage(format!(
                "change sets cannot modify active tool configuration: {}",
                path.display()
            )));
        }
    }
    let paths: BTreeSet<_> = before.keys().chain(after.keys()).cloned().collect();
    let changes: Vec<_> = paths
        .into_iter()
        .filter_map(|path| {
            let old = before.get(&path).cloned();
            let new = after.get(&path).cloned();
            (old != new).then_some(FileChange {
                path,
                before: old,
                after: new,
            })
        })
        .collect();
    transaction::preflight(root, &changes)?;
    Ok(Plan {
        base_digest,
        changes,
        before,
        interpretation,
    })
}

pub fn apply(root: &Path, plan: Plan, expected: Option<&str>) -> Result<()> {
    if expected.is_some_and(|value| value != plan.base_digest) {
        return Err(usage(
            "change-set base digest differs from --expect; preview again",
        ));
    }
    let interpretation = serde_json::to_string(&crate::bundle::settings::load_for(root)?.0)
        .map_err(|e| OkfError::Internal(e.to_string()))?;
    if snapshot(root)? != plan.before || interpretation != plan.interpretation {
        return Err(usage(
            "bundle or interpretation changed during planning; preview again",
        ));
    }
    transaction::commit(root, plan.changes)
}
