//! Scoped, qualified graph over explicitly selected locally available bundles.
use crate::bundle::catalog::{absolute, normalize, CatalogEntry};
use crate::bundle::context::Context;
use crate::bundle::loader::{load_bundle, Bundle};
use crate::error::{OkfError, Result};
use crate::fingerprint::canonicalize::sha256_hex;
use crate::model::concept::{Concept, ConceptId};
use crate::model::link::{classify, raw_links, LinkKind};
use crate::model::source::Source;
use crate::ontology::load::try_load;
use crate::parse::parse_concept;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Node {
    pub bundle: String,
    pub id: String,
    pub version: String,
}
impl Node {
    pub fn key(&self) -> String {
        format!("{}:{}@{}", self.bundle, self.id, self.version)
    }
}
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SnapshotRequest {
    pub commit: Option<String>,
    #[serde(rename = "ref")]
    pub ref_: Option<String>,
    pub label: Option<String>,
    pub digest: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct BundleRef {
    pub id: String,
    pub path: String,
    pub snapshot: Option<SnapshotRequest>,
}
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotResult {
    pub requested: SnapshotRequest,
    pub status: String,
    pub evidence: String,
    pub resolved: Option<Node>,
    pub candidate: Option<Node>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Edge {
    pub source: Node,
    pub resource: String,
    pub location: String,
    pub target: Option<Node>,
    pub status: String,
    pub evidence: String,
    pub snapshot: Option<SnapshotResult>,
    pub fingerprint_status: Option<String>,
    pub relationship: Option<crate::graph::relationships::Relationship>,
    pub reference_rule: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ExaminedBundle {
    pub id: String,
    pub root: PathBuf,
    pub version: String,
    pub interpretation: crate::bundle::settings::EffectiveSettings,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct Scope {
    pub requested: Vec<String>,
    pub examined: Vec<ExaminedBundle>,
    pub unavailable: Vec<String>,
    pub snapshot_examined: Vec<Node>,
}
/// A local source dependency retained for impact queries without making artifacts
/// concept nodes or graph edges. Paths may name deleted or unavailable files.
#[derive(Debug, Clone)]
pub struct ResourceDependency {
    pub path: PathBuf,
    pub source: Node,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CatalogGraph {
    pub scope: Scope,
    pub nodes: BTreeSet<Node>,
    pub edges: Vec<Edge>,
    #[serde(skip)]
    pub resource_dependencies: Vec<ResourceDependency>,
}
/// No fetch, including Git's implicit fetching in partial clones.
fn git(root: &Path, args: &[&str]) -> Option<Vec<u8>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(args)
        .output()
        .ok()?;
    out.status.success().then_some(out.stdout)
}
fn git_text(root: &Path, args: &[&str]) -> Option<String> {
    String::from_utf8(git(root, args)?)
        .ok()
        .map(|s| s.trim().to_string())
}
fn revision(root: &Path, requested: &str) -> Option<String> {
    if requested.starts_with('-') || requested.contains(['\n', '\0', ':']) {
        return None;
    }
    git_text(
        root,
        &["rev-parse", "--verify", &format!("{requested}^{{commit}}")],
    )
}
fn historical_bytes(root: &Path, path: &Path, requested: &str) -> Option<(String, Vec<u8>)> {
    let commit = revision(root, requested)?;
    let repo = PathBuf::from(git_text(root, &["rev-parse", "--show-toplevel"])?);
    let rel = path.strip_prefix(&repo).ok()?;
    let bytes = git(
        root,
        &["show", &format!("{commit}:{}", rel.to_string_lossy())],
    )?;
    Some((commit, bytes))
}
/// Read exactly one concept from an available local Git revision, without a bundle walk.
pub fn read_concept_at(root: &Path, id: &str, revision: &str) -> Result<Option<Concept>> {
    let id = ConceptId::parse(id)?;
    let root = std::fs::canonicalize(root)?;
    let path = root.join(format!("{}.md", id.0.trim_start_matches('/')));
    let Some((_, bytes)) = historical_bytes(&root, &path, revision) else {
        return Ok(None);
    };
    let text = std::str::from_utf8(&bytes).map_err(|e| {
        OkfError::Environment(format!("historical concept is not UTF-8: {e}")).at_path(&path)
    })?;
    parse_concept(id, text)
        .map_err(|error| error.at_path(&path))
        .map(Some)
}
pub fn load_revision_limited(
    root: &Path,
    requested: &str,
    limit: Option<usize>,
) -> Result<Option<(Bundle, String, usize)>> {
    let mut failures = Vec::new();
    let mut encoding_error = None;
    let loaded = (|| {
        let commit = revision(root, requested)?;
        let root = std::fs::canonicalize(root).ok()?;
        let repo = PathBuf::from(git_text(&root, &["rev-parse", "--show-toplevel"])?);
        let prefix = root.strip_prefix(&repo).ok()?;
        let tree = if prefix.as_os_str().is_empty() {
            git(&repo, &["ls-tree", "-rz", "--name-only", &commit])?
        } else {
            git(
                &repo,
                &[
                    "ls-tree",
                    "-rz",
                    "--name-only",
                    &commit,
                    "--",
                    &prefix.to_string_lossy(),
                ],
            )?
        };
        let mut concepts = Vec::new();
        let mut total = 0usize;
        for path in tree.split(|c| *c == 0).filter(|s| !s.is_empty()) {
            let path = PathBuf::from(std::str::from_utf8(path).ok()?);
            let rel = path.strip_prefix(prefix).ok()?;
            if path.extension().and_then(|s| s.to_str()) != Some("md")
                || matches!(
                    path.file_name().and_then(|s| s.to_str()),
                    Some("index.md" | "log.md")
                )
            {
                continue;
            }
            total += 1;
            if limit.is_some_and(|limit| total > limit) {
                continue;
            }
            let bytes = git(
                &repo,
                &["show", &format!("{commit}:{}", path.to_string_lossy())],
            )?;
            let id = ConceptId::from_relative(rel.to_string_lossy().strip_suffix(".md").unwrap());
            let text = match std::str::from_utf8(&bytes) {
                Ok(text) => text,
                Err(error) => {
                    encoding_error = Some(OkfError::Environment(format!(
                        "{} at revision {commit}: {error}",
                        repo.join(&path).display()
                    )));
                    return None;
                }
            };
            match parse_concept(id, text).map_err(|error| error.at_path(&repo.join(&path))) {
                Ok(concept) => concepts.push(concept),
                Err(OkfError::Yaml(message)) => {
                    failures.push(format!("{message} (revision {commit})"))
                }
                Err(_) => return None,
            }
        }
        concepts.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        Some((Bundle { root, concepts }, format!("git:{commit}"), total))
    })();
    if let Some(error) = encoding_error {
        return Err(error);
    }
    if !failures.is_empty() {
        return Err(OkfError::yaml_files(failures));
    }
    Ok(loaded)
}
fn primary_id(context: &Context) -> String {
    context
        .primary
        .id
        .clone()
        .unwrap_or_else(|| format!("path:{}", context.primary.root.display()))
}
fn entries(context: &Context) -> BTreeMap<String, CatalogEntry> {
    let mut entries = context
        .catalog
        .as_ref()
        .map(|c| c.entries.clone())
        .unwrap_or_default();
    let id = primary_id(context);
    entries.entry(id.clone()).or_insert(CatalogEntry {
        id,
        root: context.primary.root.clone(),
        configured_root: context.primary.root.clone(),
        overridden: false,
        available: context.primary.root.is_dir(),
        settings: BTreeMap::new(),
    });
    entries
}
fn validate_scope_roots(
    entries: &BTreeMap<String, CatalogEntry>,
    requested: &BTreeSet<String>,
) -> Result<()> {
    let roots = requested
        .iter()
        .filter_map(|id| entries.get(id))
        .filter(|entry| entry.available)
        .collect::<Vec<_>>();
    for (index, a) in roots.iter().enumerate() {
        for b in &roots[index + 1..] {
            if a.root.starts_with(&b.root) || b.root.starts_with(&a.root) {
                return Err(OkfError::Usage(format!("ambiguous examination roots: {} and {} overlap; select their registered root or examine the path independently",a.id,b.id)));
            }
        }
    }
    Ok(())
}
impl CatalogGraph {
    /// Qualified targets accept `acme.finance:/policy`; unqualified IDs select the primary.
    pub fn target(&self, context: &Context, raw: &str) -> Node {
        let (bundle, id) = raw
            .split_once(":/")
            .map(|(bundle, id)| (bundle.to_string(), format!("/{id}")))
            .unwrap_or_else(|| (primary_id(context), ConceptId::from_relative(raw).0));
        let version = self
            .scope
            .examined
            .iter()
            .find(|e| e.id == bundle)
            .map(|e| e.version.clone())
            .unwrap_or_else(|| "working-tree".into());
        let id = crate::model::link::resolve_link(&ConceptId("/context".into()), &id).0;
        Node {
            bundle,
            id,
            version,
        }
    }
    /// Reverse traversal is bounded by examined nodes and records cycles once.
    pub fn affected(&self, changed: &[Node], transitive: bool, depth: Option<usize>) -> Vec<Node> {
        let seeds: BTreeSet<_> = changed.iter().cloned().collect();
        let mut visited = seeds.clone();
        let mut queue: VecDeque<_> = changed.iter().cloned().map(|n| (n, 0usize)).collect();
        let mut out = BTreeSet::new();
        while let Some((node, distance)) = queue.pop_front() {
            if distance >= depth.unwrap_or(usize::MAX) || (!transitive && distance > 0) {
                continue;
            }
            for edge in self.edges.iter().filter(|e| {
                e.target.as_ref() == Some(&node)
                    && matches!(e.status.as_str(), "resolved" | "unchecked-equivalence")
            }) {
                if visited.insert(edge.source.clone()) {
                    if !seeds.contains(&edge.source) {
                        out.insert(edge.source.clone());
                    }
                    queue.push_back((edge.source.clone(), distance + 1));
                }
            }
        }
        out.into_iter().collect()
    }
    /// Impact of changed concepts or local source files, retaining ordinary graph depth.
    /// Resource dependencies seed their consuming concepts at distance one and are not
    /// exposed as concept graph nodes. Qualified paths select their registered bundle.
    pub fn affected_resources(
        &self,
        context: &Context,
        changed: &[String],
        transitive: bool,
        depth: Option<usize>,
    ) -> Vec<Node> {
        let entries = entries(context);
        let seeds: BTreeSet<_> = changed
            .iter()
            .map(|raw| self.target(context, raw))
            .collect();
        let mut paths = BTreeSet::new();
        for raw in changed {
            let (bundle, resource) = raw
                .split_once(":/")
                .map(|(bundle, resource)| (bundle.to_string(), resource))
                .unwrap_or_else(|| (primary_id(context), raw.as_str()));
            let Some(entry) = entries.get(&bundle) else {
                continue;
            };
            if classify(resource) == LinkKind::External {
                continue;
            }
            let clean = resource.split(['#', '?']).next().unwrap_or(resource);
            let path = normalize(&entry.root.join(clean.trim_start_matches('/')));
            paths.insert(std::fs::canonicalize(&path).unwrap_or(path.clone()));
            // Concept IDs conventionally omit .md; keep that interpretation alongside
            // an exact path so extensionless opaque files also remain addressable.
            if path.extension().is_none() {
                let document = path.with_extension("md");
                paths.insert(std::fs::canonicalize(&document).unwrap_or(document));
            }
        }
        let max_hops = if transitive {
            depth.unwrap_or(usize::MAX)
        } else {
            depth.unwrap_or(1).min(1)
        };
        let mut visited = seeds.clone();
        let mut queue: VecDeque<_> = seeds.iter().cloned().map(|node| (node, 0usize)).collect();
        let mut out = BTreeSet::new();
        if max_hops > 0 {
            for dependency in &self.resource_dependencies {
                if paths.contains(&dependency.path) && visited.insert(dependency.source.clone()) {
                    out.insert(dependency.source.clone());
                    queue.push_back((dependency.source.clone(), 1));
                }
            }
        }
        while let Some((node, distance)) = queue.pop_front() {
            if distance >= max_hops {
                continue;
            }
            for edge in self.edges.iter().filter(|edge| {
                edge.target.as_ref() == Some(&node)
                    && matches!(edge.status.as_str(), "resolved" | "unchecked-equivalence")
            }) {
                if visited.insert(edge.source.clone()) {
                    out.insert(edge.source.clone());
                    queue.push_back((edge.source.clone(), distance + 1));
                }
            }
        }
        out.into_iter().collect()
    }

    pub fn neighborhood(
        &self,
        seed: &Node,
        direction: &str,
        depth: Option<usize>,
    ) -> BTreeSet<Node> {
        let mut visited = BTreeSet::from([seed.clone()]);
        let mut queue = VecDeque::from([(seed.clone(), 0usize)]);
        while let Some((node, distance)) = queue.pop_front() {
            if distance >= depth.unwrap_or(usize::MAX) {
                continue;
            }
            for edge in &self.edges {
                let next = if direction != "incoming" && edge.source == node {
                    edge.target.clone()
                } else if direction != "outgoing" && edge.target.as_ref() == Some(&node) {
                    Some(edge.source.clone())
                } else {
                    None
                };
                if let Some(next) = next {
                    if visited.insert(next.clone())
                        && matches!(edge.status.as_str(), "resolved" | "unchecked-equivalence")
                    {
                        queue.push_back((next, distance + 1));
                    }
                }
            }
        }
        visited
    }
}
/// Establish selected scope without walking or parsing concept files (query scan budgets
/// must be applied before any bundle-wide materialization).
pub fn resolve_scope(context: &Context, scope_ids: &[String], all: bool) -> Result<Scope> {
    let entries = entries(context);
    let mut requested = BTreeSet::from([primary_id(context)]);
    if all {
        requested.extend(entries.keys().cloned());
    }
    for id in scope_ids {
        if !entries.contains_key(id) {
            return Err(OkfError::Usage(format!("unknown bundle ID in scope: {id}")));
        }
        requested.insert(id.clone());
    }
    validate_scope_roots(&entries, &requested)?;
    let mut scope = Scope {
        requested: requested.iter().cloned().collect(),
        ..Default::default()
    };
    for id in requested {
        let entry = &entries[&id];
        if entry.available {
            scope.examined.push(ExaminedBundle {
                id,
                root: entry.root.clone(),
                version: "working-tree".into(),
                interpretation: crate::bundle::settings::for_context(context, &entry.root)?,
            });
        } else {
            scope.unavailable.push(id);
        }
    }
    let primary = primary_id(context);
    scope
        .examined
        .sort_by_key(|entry| (entry.id != primary, entry.id.clone()));
    Ok(scope)
}
pub fn resolve_scope_at(
    context: &Context,
    scope_ids: &[String],
    all: bool,
    requested_revision: Option<&str>,
) -> Result<Scope> {
    let mut scope = resolve_scope(context, scope_ids, all)?;
    if let Some(requested) = requested_revision {
        let mut examined = Vec::new();
        for mut entry in scope.examined {
            let Some(commit) = revision(&entry.root, requested) else {
                scope.unavailable.push(format!("{}@{requested}", entry.id));
                continue;
            };
            entry.version = format!("git:{commit}");
            entry.interpretation =
                crate::bundle::settings::for_context_at(context, &entry.root, &commit)?.0;
            examined.push(entry);
        }
        scope.examined = examined;
    }
    Ok(scope)
}
pub fn build_catalog_graph(
    context: &Context,
    scope_ids: &[String],
    all: bool,
    requested_revision: Option<&str>,
) -> Result<CatalogGraph> {
    let entries = entries(context);
    let mut requested = BTreeSet::from([primary_id(context)]);
    if all {
        requested.extend(entries.keys().cloned());
    }
    for id in scope_ids {
        if !entries.contains_key(id) {
            return Err(OkfError::Usage(format!("unknown bundle ID in scope: {id}")));
        }
        requested.insert(id.clone());
    }
    validate_scope_roots(&entries, &requested)?;
    let mut graph = CatalogGraph {
        scope: Scope {
            requested: requested.iter().cloned().collect(),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bundles = BTreeMap::new();
    for id in &requested {
        let entry = &entries[id];
        if !entry.available {
            graph.scope.unavailable.push(id.clone());
            continue;
        }
        let (bundle, version) = if let Some(rev) = requested_revision {
            match load_revision_limited(&entry.root, rev, None)? {
                Some((bundle, version, _)) => (bundle, version),
                None => {
                    graph.scope.unavailable.push(format!("{id}@{rev}"));
                    continue;
                }
            }
        } else {
            (load_bundle(&entry.root)?, "working-tree".into())
        };
        for concept in &bundle.concepts {
            graph.nodes.insert(Node {
                bundle: id.clone(),
                id: concept.id.0.clone(),
                version: version.clone(),
            });
        }
        let interpretation = if let Some(rev) = version.strip_prefix("git:") {
            crate::bundle::settings::for_context_at(context, &entry.root, rev)?.0
        } else {
            crate::bundle::settings::for_context(context, &entry.root)?
        };
        graph.scope.examined.push(ExaminedBundle {
            id: id.clone(),
            root: entry.root.clone(),
            version: version.clone(),
            interpretation,
        });
        bundles.insert(id.clone(), (bundle, version));
    }
    for (id, (bundle, version)) in &bundles {
        let ontology = if let Some(rev) = version.strip_prefix("git:") {
            crate::bundle::settings::for_context_at(context, &bundle.root, rev)?.1
        } else {
            let interpretation = crate::bundle::settings::for_context(context, &bundle.root)?;
            interpretation
                .ontology_path
                .as_ref()
                .map(|path| crate::ontology::load::load_ontology(path))
                .transpose()?
        };
        for concept in &bundle.concepts {
            let source = Node {
                bundle: id.clone(),
                id: concept.id.0.clone(),
                version: version.clone(),
            };
            let sources = concept
                .frontmatter
                .get("sources")
                .and_then(Value::as_sequence)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            // Preserve each provenance occurrence, including URL declared mappings.
            for (index, value) in sources.iter().enumerate() {
                let Some(resource) = value.get("resource").and_then(Value::as_str) else {
                    continue;
                };
                if let Some(path) = source_dependency_path(
                    &bundle.root,
                    &source.id,
                    resource,
                    value.get("kind").and_then(Value::as_str),
                ) {
                    graph.resource_dependencies.push(ResourceDependency {
                        path,
                        source: source.clone(),
                    });
                }
                if classify(resource) == LinkKind::External && value.get("bundle_ref").is_none() {
                    continue;
                }
                if !resource
                    .split(['#', '?'])
                    .next()
                    .is_some_and(|s| s.ends_with(".md"))
                    && value.get("bundle_ref").is_none()
                {
                    continue;
                }
                let declared = value
                    .get("bundle_ref")
                    .map(|v| serde_yaml::from_value::<BundleRef>(v.clone()));
                let git_kind = value
                    .get("kind")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| matches!(kind, "git-path" | "git-commit"));
                let git_root = if git_kind {
                    git_text(&bundle.root, &["rev-parse", "--show-toplevel"]).map(PathBuf::from)
                } else {
                    None
                };
                let path_source = if git_kind {
                    Node {
                        id: "/context".into(),
                        ..source.clone()
                    }
                } else {
                    source.clone()
                };
                let mut edge = resolve_edge(
                    ResolutionContext {
                        entries: &entries,
                        scope: &requested,
                        bundles: &bundles,
                    },
                    &path_source,
                    git_root.as_deref().unwrap_or(&bundle.root),
                    resource,
                    &format!("sources[{index}].resource"),
                    declared.as_ref().and_then(|r| r.as_ref().ok()),
                );
                edge.source = source.clone();
                if declared.as_ref().is_some_and(|r| r.is_err()) {
                    edge.status = "invalid-bundle-ref".into();
                    edge.evidence =
                        "bundle_ref requires string id/path and correctly typed optional snapshot"
                            .into();
                }
                if edge.status == "resolved" {
                    if let Some(source_value) = Source::from_value(value) {
                        edge.fingerprint_status =
                            fingerprint_status(&source_value, &bundle.root, &concept.id);
                    }
                }
                graph.edges.push(edge);
            }
            if let Some(ct) = concept
                .concept_type()
                .and_then(|name| ontology.as_ref().and_then(|o| o.concepts.get(name)))
            {
                let yaml = serde_yaml::to_value(&concept.frontmatter.map).unwrap_or_default();
                for (key, rule) in &ct.references {
                    let selector = crate::graph::relationships::reference_selector(
                        key,
                        rule.selector.as_deref(),
                    )?;
                    for occurrence in selector.select(&yaml).leaves {
                        let values: Vec<_> = if rule.selector.is_none() {
                            match occurrence.value.as_sequence() {
                                Some(seq) => seq
                                    .iter()
                                    .enumerate()
                                    .map(|(i, v)| (v, format!("{}[{i}]", occurrence.path)))
                                    .collect(),
                                None => vec![(occurrence.value, occurrence.path.clone())],
                            }
                        } else {
                            vec![(occurrence.value, occurrence.path.clone())]
                        };
                        for (value, location) in values {
                            if let Some(resource) = value.as_str() {
                                if classify(resource) != LinkKind::External {
                                    let mut edge = resolve_edge(
                                        ResolutionContext {
                                            entries: &entries,
                                            scope: &requested,
                                            bundles: &bundles,
                                        },
                                        &source,
                                        &bundle.root,
                                        resource,
                                        &location,
                                        None,
                                    );
                                    edge.reference_rule = Some(key.clone());
                                    graph.edges.push(edge);
                                }
                            }
                        }
                    }
                }
            }
            for (event, range) in pulldown_cmark::Parser::new(&concept.body).into_offset_iter() {
                if let pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link {
                    dest_url, ..
                }) = event
                {
                    if classify(&dest_url) != LinkKind::External {
                        let line = concept.body[..range.start]
                            .bytes()
                            .filter(|c| *c == b'\n')
                            .count()
                            + 1;
                        graph.edges.push(resolve_edge(
                            ResolutionContext {
                                entries: &entries,
                                scope: &requested,
                                bundles: &bundles,
                            },
                            &source,
                            &bundle.root,
                            &dest_url,
                            &format!("body:line:{line}"),
                            None,
                        ));
                    }
                }
            }
            if let Some(ontology) = &ontology {
                for relationship in crate::graph::relationships::relationships(concept, ontology) {
                    if classify(&relationship.raw_reference) == LinkKind::External {
                        continue;
                    }
                    let mut edge = resolve_edge(
                        ResolutionContext {
                            entries: &entries,
                            scope: &requested,
                            bundles: &bundles,
                        },
                        &source,
                        &bundle.root,
                        &relationship.raw_reference,
                        &relationship.field_path,
                        None,
                    );
                    edge.reference_rule = concept
                        .concept_type()
                        .and_then(|name| ontology.concepts.get(name))
                        .and_then(|ct| ct.relationships.get(&relationship.rule))
                        .map(|rule| rule.reference.clone());
                    if let Some(existing) = graph.edges.iter_mut().find(|e| {
                        e.source == source
                            && e.resource == relationship.raw_reference
                            && e.location == relationship.field_path
                            && e.relationship.is_none()
                    }) {
                        existing.relationship = Some(relationship);
                    } else {
                        edge.relationship = Some(relationship);
                        graph.edges.push(edge);
                    }
                }
            }
        }
    }
    for edge in &graph.edges {
        if let Some(snapshot) = &edge.snapshot {
            if let Some(node) = &snapshot.resolved {
                graph.nodes.insert(node.clone());
                if !graph.scope.snapshot_examined.contains(node) {
                    graph.scope.snapshot_examined.push(node.clone());
                }
            }
        }
    }
    Ok(graph)
}
pub(crate) fn source_dependency_path(
    root: &Path,
    id: &str,
    resource: &str,
    kind: Option<&str>,
) -> Option<PathBuf> {
    if classify(resource) == LinkKind::External {
        return None;
    }
    if !matches!(
        kind,
        Some("file" | "line-range" | "markdown-heading" | "git-path" | "git-commit")
    ) && resource.chars().any(char::is_whitespace)
    {
        return None;
    }
    let clean = resource.split('#').next().unwrap_or(resource);
    let path = match kind {
        Some("url") => return None,
        Some("file" | "line-range" | "markdown-heading") => root.join(clean),
        Some("git-path" | "git-commit") => {
            PathBuf::from(git_text(root, &["rev-parse", "--show-toplevel"])?).join(clean)
        }
        _ if clean.starts_with('/') => root.join(clean.trim_start_matches('/')),
        _ => root.join(id.trim_start_matches('/')).parent()?.join(clean),
    };
    let path = normalize(&path);
    Some(std::fs::canonicalize(&path).unwrap_or(path))
}

fn fingerprint_status(source: &Source, root: &Path, id: &ConceptId) -> Option<String> {
    if source.fingerprint.is_empty() {
        return None;
    }
    let parent = root
        .join(format!("{}.md", id.0.trim_start_matches('/')))
        .parent()?
        .to_path_buf();
    let engine = crate::fingerprint::Engine::new(
        &parent,
        &crate::ports::fs::RealFs,
        &crate::ports::git::RealGit,
        None,
    );
    use crate::fingerprint::Fingerprinter;
    match engine.fingerprint(source) {
        Ok(current) => Some(
            if source
                .fingerprint
                .fields
                .iter()
                .all(|(k, v)| current.get(k) == Some(v))
            {
                "unchanged"
            } else {
                "changed"
            }
            .into(),
        ),
        Err(_) => Some("unavailable".into()),
    }
}
/// Resolve an ordinary document reference without clamping traversal to a bundle boundary.
pub fn ordinary_path(root: &Path, id: &str, resource: &str) -> Option<PathBuf> {
    if resource.trim().starts_with('#') {
        let path = root.join(format!("{}.md", id.trim_start_matches('/')));
        return Some(std::fs::canonicalize(&path).unwrap_or(path));
    }
    if classify(resource) == LinkKind::External {
        return None;
    }
    let clean = resource.split(['#', '?']).next().unwrap_or(resource);
    let mut path = if clean.starts_with('/') {
        root.join(clean.trim_start_matches('/'))
    } else {
        root.join(id.trim_start_matches('/')).parent()?.join(clean)
    };
    if path.extension().is_none() {
        path.set_extension("md");
    }
    let path = normalize(&path);
    Some(std::fs::canonicalize(&path).unwrap_or(path))
}
/// Identify available material by its effective root; unavailable lexical locations are
/// candidates only when exactly one registration contains the path.
pub fn entry_for_path<'a>(
    entries: &'a BTreeMap<String, CatalogEntry>,
    path: &Path,
) -> Option<&'a CatalogEntry> {
    let entry = if let Some(entry) = entries
        .values()
        .filter(|e| e.available && path.starts_with(&e.root))
        .max_by_key(|entry| entry.root.components().count())
    {
        entry
    } else {
        let candidates = entries
            .values()
            .filter(|e| !e.available && path.starts_with(&e.root))
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            return None;
        }
        candidates[0]
    };
    Some(entry)
}
fn target_for_path(
    entries: &BTreeMap<String, CatalogEntry>,
    path: &Path,
    bundles: &BTreeMap<String, (Bundle, String)>,
) -> Option<Node> {
    let entry = entry_for_path(entries, path)?;
    let rel = path.strip_prefix(&entry.root).ok()?;
    if rel.extension().and_then(|s| s.to_str()) != Some("md")
        || matches!(
            rel.file_name().and_then(|s| s.to_str()),
            Some("index.md" | "log.md")
        )
    {
        return None;
    }
    Some(Node {
        bundle: entry.id.clone(),
        id: ConceptId::from_relative(rel.to_string_lossy().strip_suffix(".md").unwrap()).0,
        version: bundles
            .get(&entry.id)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "working-tree".into()),
    })
}
struct ResolutionContext<'a> {
    entries: &'a BTreeMap<String, CatalogEntry>,
    scope: &'a BTreeSet<String>,
    bundles: &'a BTreeMap<String, (Bundle, String)>,
}
fn resolve_edge(
    context: ResolutionContext<'_>,
    source: &Node,
    root: &Path,
    resource: &str,
    location: &str,
    declared: Option<&BundleRef>,
) -> Edge {
    let ResolutionContext {
        entries,
        scope,
        bundles,
    } = context;
    let path = ordinary_path(root, &source.id, resource);
    let ordinary = path
        .as_ref()
        .and_then(|p| target_for_path(entries, p, bundles));
    let mut edge = Edge {
        source: source.clone(),
        resource: resource.into(),
        location: location.into(),
        target: ordinary.clone(),
        status: "missing-target".into(),
        evidence: "ordinary-path".into(),
        snapshot: None,
        fingerprint_status: None,
        relationship: None,
        reference_rule: None,
    };
    if let Some(declared) = declared {
        let Some(entry) = entries.get(&declared.id) else {
            edge.status = "unknown-bundle-id".into();
            return edge;
        };
        if Path::new(&declared.path).is_absolute()
            || Path::new(&declared.path)
                .components()
                .any(|p| matches!(p, std::path::Component::ParentDir))
        {
            edge.status = "reference-mismatch".into();
            edge.evidence = "bundle_ref.path must stay relative to target bundle root".into();
            return edge;
        }
        if !entry.available {
            edge.status = "unavailable-root".into();
            return edge;
        }
        let declared_path = normalize(&entry.root.join(&declared.path));
        let declared_path = std::fs::canonicalize(&declared_path).unwrap_or(declared_path);
        if !declared_path.starts_with(&entry.root) {
            edge.status = "reference-mismatch".into();
            edge.evidence = "bundle_ref target escapes registered root".into();
            return edge;
        }
        let declared_target = target_for_path(entries, &declared_path, bundles);
        if let Some(path) = &path {
            if path != &declared_path {
                edge.status = "reference-mismatch".into();
                edge.evidence =
                    "ordinary resource and bundle_ref address different local paths".into();
                return edge;
            }
            edge.evidence = "same-local-file".into();
        } else {
            edge.target = declared_target;
            edge.evidence = "declared-mapping; equivalence-unchecked".into();
        }
        if let Some(snapshot) = &declared.snapshot {
            // Never inspect snapshots outside explicitly authorized examination scope.
            if scope.contains(&declared.id) {
                edge.snapshot = Some(resolve_snapshot(
                    snapshot,
                    &entry.root,
                    &declared_path,
                    edge.target.as_ref(),
                    path.is_some(),
                ));
            } else {
                edge.snapshot = Some(SnapshotResult {
                    requested: snapshot.clone(),
                    status: "out-of-scope".into(),
                    evidence: "not-examined".into(),
                    resolved: None,
                    candidate: None,
                });
            }
        }
    }
    if let Some(target) = &edge.target {
        if entries
            .get(&target.bundle)
            .is_some_and(|entry| !entry.available)
        {
            edge.status = "unavailable-root".into();
            edge.evidence =
                "registered lexical location; root unavailable and canonical equivalence unchecked"
                    .into();
        } else if !scope.contains(&target.bundle) {
            edge.status = "out-of-scope".into();
        } else if let Some((bundle, _)) = bundles.get(&target.bundle) {
            if bundle.get(&target.id).is_some() {
                edge.status = if path.is_some() {
                    "resolved"
                } else {
                    "unchecked-equivalence"
                }
                .into();
            }
        } else {
            edge.status = "unavailable-root-or-revision".into();
        }
    } else if path.as_ref().is_some_and(|p| !p.starts_with(root)) {
        edge.status = "unregistered-target".into();
    }
    edge
}
fn resolve_snapshot(
    request: &SnapshotRequest,
    root: &Path,
    path: &Path,
    target: Option<&Node>,
    ordinary_local: bool,
) -> SnapshotResult {
    let current = match target.and_then(|n| n.version.strip_prefix("git:")) {
        Some(rev) => historical_bytes(root, path, rev).map(|(_, bytes)| bytes),
        None => std::fs::read(path).ok(),
    };
    let mut result = SnapshotResult {
        requested: request.clone(),
        status: "unverified".into(),
        evidence: "label-or-unspecified-expectation".into(),
        resolved: None,
        candidate: target.cloned().filter(|_| current.is_some()),
    };
    let digest = request
        .digest
        .as_deref()
        .map(|s| s.strip_prefix("sha256:").unwrap_or(s));
    if let Some(digest) = digest {
        if digest.len() != 64 || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
            result.status = "mismatched".into();
            result.evidence = "invalid SHA-256 file digest".into();
            return result;
        }
    }
    if let Some(requested) = request.commit.as_deref().or(request.ref_.as_deref()) {
        // A commit is an exact immutable object ID; ref names belong in the ref field.
        if request.commit.is_some()
            && (!(requested.len() == 40 || requested.len() == 64)
                || !requested.chars().all(|c| c.is_ascii_hexdigit()))
        {
            result.status = "mismatched".into();
            result.evidence = "commit must be a full immutable object ID".into();
            return result;
        }
        let Some((commit, bytes)) = historical_bytes(root, path, requested) else {
            result.status = "unavailable".into();
            result.evidence = "requested local Git object unavailable (no fetch)".into();
            return result;
        };
        if request
            .commit
            .as_ref()
            .is_some_and(|expected| expected.to_lowercase() != commit)
        {
            result.status = "mismatched".into();
            result.evidence =
                "exact commit identifier resolves to a different commit object".into();
            return result;
        }
        if let Some(digest) = digest {
            if sha256_hex(&bytes) != digest.to_lowercase() {
                result.status = "mismatched".into();
                result.evidence = "historical file digest differs".into();
                return result;
            }
        }
        result.resolved = target.cloned().map(|mut n| {
            n.version = format!("git:{commit}");
            n
        });
        if ordinary_local && current.as_ref().is_some_and(|current| current != &bytes) {
            result.status = "mismatched".into();
            result.evidence =
                "requested Git bytes found; ordinary live resource has different bytes".into();
        } else {
            result.status = "matched".into();
            result.evidence = if ordinary_local {
                "same-repository-commit-path-and-bytes"
            } else {
                "declared-mapping; equivalence-unchecked"
            }
            .into();
            result.candidate = None;
        }
    } else if let Some(digest) = digest {
        match current {
            Some(bytes) if sha256_hex(&bytes) == digest.to_lowercase() => {
                result.status = "matched".into();
                result.evidence = "raw-file-sha256".into();
                result.resolved = target.cloned().map(|mut node| {
                    node.version = format!("sha256:{}", digest.to_lowercase());
                    node
                });
                result.candidate = None;
            }
            Some(_) => {
                result.status = "mismatched".into();
                result.evidence = "raw-file-sha256 differs".into();
            }
            None => {
                result.status = "unavailable".into();
                result.evidence = "local file unavailable".into();
            }
        }
    }
    result
}
/// Resolve a standalone ordinary reference without loading content outside scope.
pub fn resolve_reference(
    context: &Context,
    graph: &CatalogGraph,
    from: &str,
    resource: &str,
) -> Edge {
    let entries = entries(context);
    let requested = graph.scope.requested.iter().cloned().collect();
    let bundles = graph
        .scope
        .examined
        .iter()
        .map(|e| {
            (
                e.id.clone(),
                (
                    Bundle {
                        root: e.root.clone(),
                        concepts: Vec::new(),
                    },
                    e.version.clone(),
                ),
            )
        })
        .collect();
    let source = graph.target(context, from);
    let source_root = entries
        .get(&source.bundle)
        .map(|entry| entry.root.as_path());
    let mut edge = resolve_edge(
        ResolutionContext {
            entries: &entries,
            scope: &requested,
            bundles: &bundles,
        },
        &source,
        source_root.unwrap_or(&context.primary.root),
        resource,
        "argument",
        None,
    );
    if source_root.is_none() {
        edge.status = "unknown-bundle-id".into();
        edge.evidence = "source bundle is not registered".into();
        return edge;
    }
    // Structural Markdown files are resolvable documents, but remain excluded from
    // the concept graph. Resolve them only for this explicit reference query.
    if let Some(path) = ordinary_path(source_root.unwrap(), &source.id, resource) {
        if matches!(
            path.file_name().and_then(|s| s.to_str()),
            Some("index.md" | "log.md")
        ) {
            if let Some(entry) = entry_for_path(&entries, &path) {
                let id = ConceptId::from_relative(
                    &path
                        .strip_prefix(&entry.root)
                        .unwrap()
                        .with_extension("")
                        .to_string_lossy(),
                )
                .0;
                let examined = graph.scope.examined.iter().find(|e| e.id == entry.id);
                let version = examined
                    .map(|e| e.version.clone())
                    .unwrap_or_else(|| "working-tree".into());
                let exists = entry.available
                    && graph.scope.requested.contains(&entry.id)
                    && examined.is_some()
                    && version
                        .strip_prefix("git:")
                        .map(|rev| historical_bytes(&entry.root, &path, rev).is_some())
                        .unwrap_or_else(|| path.is_file());
                edge.target = Some(Node {
                    bundle: entry.id.clone(),
                    id,
                    version,
                });
                edge.status = if !entry.available {
                    "unavailable-root"
                } else if !graph.scope.requested.contains(&entry.id) {
                    "out-of-scope"
                } else if examined.is_none() {
                    "unavailable-root-or-revision"
                } else if exists {
                    "resolved"
                } else {
                    "missing-target"
                }
                .into();
            }
        }
    }
    if edge
        .target
        .as_ref()
        .is_some_and(|n| graph.nodes.contains(n))
        && edge.status == "missing-target"
    {
        edge.status = "resolved".into();
    }
    edge
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for path in ["a/metrics", "b/policies"] {
            std::fs::create_dir_all(dir.path().join(path)).unwrap();
        }
        std::fs::write(
            dir.path().join("okf.toml"),
            "catalog='catalog.yaml'\ndefault_bundle={id='acme.a'}\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("catalog.yaml"),"catalog_version: 1\nbundles:\n  acme.a: {location: {type: directory, path: a}}\n  acme.b: {location: {type: directory, path: b}}\n").unwrap();
        std::fs::write(
            dir.path().join("b/policies/margin.md"),
            "---\ntype: Policy\n---\napproved\n",
        )
        .unwrap();
        dir
    }
    fn context(dir: &tempfile::TempDir) -> Context {
        crate::bundle::context::resolve_context(None, None, None, dir.path()).unwrap()
    }
    fn write_source(dir: &tempfile::TempDir, snapshot: &str, body: &str) {
        std::fs::write(dir.path().join("a/metrics/revenue.md"),format!("---\ntype: Metric\nsources:\n- resource: ../../b/policies/margin.md\n  kind: file\n  bundle_ref:\n    id: acme.b\n    path: policies/margin.md\n{snapshot}---\n{body}\n")).unwrap();
    }
    fn git_ok(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().into()
    }
    #[test]
    fn catalog_keeps_repository_relative_git_concept_sources_and_body_occurrences() {
        let dir = fixture();
        git_ok(dir.path(), &["init", "-q"]);
        write_source(&dir, "", "[Missing](/missing.md)");
        let path = dir.path().join("a/metrics/revenue.md");
        let text = std::fs::read_to_string(&path)
            .unwrap()
            .replace("../../b/policies/margin.md", "b/policies/margin.md")
            .replace("kind: file", "kind: git-path");
        std::fs::write(&path, text).unwrap();
        let graph = build_catalog_graph(&context(&dir), &[], true, None).unwrap();
        assert_eq!(graph.edges.len(), 2);
        assert_eq!(graph.edges[0].status, "resolved");
        assert_eq!(graph.edges[0].target.as_ref().unwrap().bundle, "acme.b");
        assert_eq!(
            graph.edges[0].target.as_ref().unwrap().id,
            "/policies/margin"
        );
        assert_eq!(graph.edges[1].status, "missing-target");
        assert_eq!(graph.edges[1].target.as_ref().unwrap().id, "/missing");
    }

    #[test]
    fn external_sources_need_explicit_mapping_and_fragments_preserve_resource_meaning() {
        let dir = fixture();
        std::fs::write(
            dir.path().join("a/metrics/revenue.md"),
            "---\ntype: Metric\nsources:\n- resource: https://example.invalid/policy.md\n---\n",
        )
        .unwrap();
        let graph = build_catalog_graph(&context(&dir), &[], true, None).unwrap();
        assert!(graph.edges.is_empty());
        write_source(&dir, "", "");
        let path = dir.path().join("a/metrics/revenue.md");
        let text = std::fs::read_to_string(&path)
            .unwrap()
            .replace("../../b/policies/margin.md", "\"#section\"");
        std::fs::write(&path, text).unwrap();
        let graph = build_catalog_graph(&context(&dir), &[], true, None).unwrap();
        assert_eq!(graph.edges[0].status, "reference-mismatch");
    }
    #[test]
    fn standalone_subdirectory_keeps_its_boundary_and_rejects_ambiguous_scope() {
        let dir = fixture();
        write_source(&dir, "", "[sibling](sibling.md)");
        std::fs::write(
            dir.path().join("a/metrics/sibling.md"),
            "---\ntype: Note\n---\n",
        )
        .unwrap();
        let ctx =
            crate::bundle::context::resolve_context(Some("a/metrics"), None, None, dir.path())
                .unwrap();
        let graph = build_catalog_graph(&ctx, &[], false, None).unwrap();
        let edge = graph
            .edges
            .iter()
            .find(|edge| edge.resource == "sibling.md")
            .unwrap();
        assert_eq!(edge.status, "resolved");
        assert_eq!(edge.target.as_ref().unwrap().bundle, edge.source.bundle);
        assert_eq!(edge.target.as_ref().unwrap().id, "/sibling");
        assert!(build_catalog_graph(&ctx, &[], true, None).is_err());
        assert!(resolve_scope(&ctx, &[], true).is_err());
    }
    #[test]
    fn cross_bundle_paths_are_qualified_without_above_root_clamping() {
        let dir = fixture();
        write_source(&dir, "", "[policy](../../b/policies/margin.md)");
        let ctx = context(&dir);
        let default = build_catalog_graph(&ctx, &[], false, None).unwrap();
        assert_eq!(default.scope.examined.len(), 1);
        assert!(default.edges.iter().all(|e| e.status == "out-of-scope"));
        assert_eq!(
            default.edges[0].target.as_ref().unwrap().id,
            "/policies/margin"
        );
        let graph = build_catalog_graph(&ctx, &[], true, None).unwrap();
        assert_eq!(graph.scope.examined.len(), 2);
        assert!(graph.edges.iter().all(|e| e.status == "resolved"));
        assert_eq!(graph.edges[0].target.as_ref().unwrap().bundle, "acme.b");
        let edge = resolve_reference(&ctx, &graph, "/metrics/revenue", "../../../outside.md");
        assert_eq!(edge.status, "unregistered-target");
        assert!(edge.target.is_none());
    }
    #[test]
    fn out_of_scope_content_is_not_loaded() {
        let dir = fixture();
        write_source(&dir, "", "[policy](../../b/policies/margin.md)");
        std::fs::write(
            dir.path().join("b/policies/margin.md"),
            "---\ntype: [\n---\n",
        )
        .unwrap();
        assert!(build_catalog_graph(&context(&dir), &[], false, None).is_ok());
        assert!(build_catalog_graph(&context(&dir), &[], true, None).is_err());
    }
    #[test]
    fn declaration_never_redirects_a_known_mismatch() {
        let dir = fixture();
        write_source(&dir, "", "");
        let p = dir.path().join("a/metrics/revenue.md");
        let text = std::fs::read_to_string(&p)
            .unwrap()
            .replace("path: policies/margin.md", "path: policies/other.md");
        std::fs::write(&p, text).unwrap();
        let graph = build_catalog_graph(&context(&dir), &[], true, None).unwrap();
        let edge = &graph.edges[0];
        assert_eq!(edge.status, "reference-mismatch");
        assert_eq!(edge.target.as_ref().unwrap().id, "/policies/margin");
    }
    #[test]
    fn snapshot_candidates_are_separate_and_body_links_do_not_inherit_expectations() {
        let dir = fixture();
        write_source(
            &dir,
            "    snapshot: {label: FY2026}\n",
            "[policy](../../b/policies/margin.md)",
        );
        let graph = build_catalog_graph(&context(&dir), &[], true, None).unwrap();
        let snapshot = graph.edges[0].snapshot.as_ref().unwrap();
        assert_eq!(snapshot.status, "unverified");
        assert!(snapshot.candidate.is_some());
        assert!(snapshot.resolved.is_none());
        assert!(graph.edges[1].snapshot.is_none());
    }
    #[test]
    fn raw_digest_matches_bytes_and_not_canonicalized_text() {
        let dir = fixture();
        let p = dir.path().join("b/policies/margin.md");
        let digest = sha256_hex(&std::fs::read(&p).unwrap());
        write_source(
            &dir,
            &format!("    snapshot: {{digest: 'sha256:{digest}'}}\n"),
            "",
        );
        let graph = build_catalog_graph(&context(&dir), &[], true, None).unwrap();
        assert_eq!(graph.edges[0].snapshot.as_ref().unwrap().status, "matched");
        std::fs::write(&p, "---\r\ntype: Policy\r\n---\r\napproved\r\n").unwrap();
        let graph = build_catalog_graph(&context(&dir), &[], true, None).unwrap();
        assert_eq!(
            graph.edges[0].snapshot.as_ref().unwrap().status,
            "mismatched"
        );
    }
    #[test]
    fn local_git_history_remains_separate_from_live_mismatch() {
        let dir = fixture();
        git_ok(dir.path(), &["init", "-q"]);
        git_ok(dir.path(), &["config", "user.name", "Test"]);
        git_ok(
            dir.path(),
            &["config", "user.email", "test@example.invalid"],
        );
        write_source(&dir, "", "");
        git_ok(dir.path(), &["add", "."]);
        git_ok(dir.path(), &["commit", "-qm", "initial"]);
        let commit = git_ok(dir.path(), &["rev-parse", "HEAD"]);
        write_source(&dir, &format!("    snapshot: {{commit: '{commit}'}}\n"), "");
        std::fs::write(
            dir.path().join("b/policies/margin.md"),
            "---\ntype: Policy\n---\nchanged\n",
        )
        .unwrap();
        let graph = build_catalog_graph(&context(&dir), &[], true, None).unwrap();
        let edge = &graph.edges[0];
        let snapshot = edge.snapshot.as_ref().unwrap();
        assert_eq!(edge.status, "resolved");
        assert_eq!(edge.target.as_ref().unwrap().version, "working-tree");
        assert_eq!(snapshot.status, "mismatched");
        assert_eq!(
            snapshot.resolved.as_ref().unwrap().version,
            format!("git:{commit}")
        );
        assert_eq!(snapshot.candidate.as_ref().unwrap().version, "working-tree");
        let historical = build_catalog_graph(&context(&dir), &[], true, Some(&commit)).unwrap();
        assert!(historical
            .nodes
            .iter()
            .all(|n| n.version == format!("git:{commit}")));
        assert!(
            build_catalog_graph(&context(&dir), &[], true, Some("missing-revision"))
                .unwrap()
                .scope
                .unavailable
                .len()
                == 2
        );
    }
    #[test]
    fn cross_bundle_cycles_are_bounded_and_report_only_examined_consumers() {
        let dir = fixture();
        write_source(&dir, "", "");
        std::fs::write(
            dir.path().join("b/policies/margin.md"),
            "---\ntype: Policy\n---\n[revenue](../../a/metrics/revenue.md)\n",
        )
        .unwrap();
        let ctx = context(&dir);
        let graph = build_catalog_graph(&ctx, &[], true, None).unwrap();
        let changed = graph.target(&ctx, "acme.b:/policies/margin");
        let affected = graph.affected(std::slice::from_ref(&changed), true, None);
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0].bundle, "acme.a");
        assert_eq!(graph.neighborhood(&changed, "both", None).len(), 2);
        let scoped = build_catalog_graph(&ctx, &[], false, None).unwrap();
        assert_eq!(scoped.scope.examined.len(), 1);
    }
    #[test]
    fn qualified_from_resolves_in_its_registered_bundle() {
        let dir = fixture();
        let ctx = context(&dir);
        let graph = build_catalog_graph(&ctx, &[], true, None).unwrap();
        let edge = resolve_reference(&ctx, &graph, "acme.b:/policies/context", "margin.md");
        assert_eq!(edge.status, "resolved");
        let target = edge.target.unwrap();
        assert_eq!(target.bundle, "acme.b");
        assert_eq!(target.id, "/policies/margin");
    }

    #[test]
    fn standalone_reference_resolves_structural_documents_without_graph_nodes() {
        let dir = fixture();
        std::fs::write(dir.path().join("a/index.md"), "# Bundle\n").unwrap();
        let ctx = context(&dir);
        let graph = build_catalog_graph(&ctx, &[], false, None).unwrap();
        let edge = resolve_reference(&ctx, &graph, "/context", "/index.md");
        assert_eq!(edge.status, "resolved");
        let target = edge.target.unwrap();
        assert_eq!(target.id, "/index");
        assert!(!graph.nodes.contains(&target));
        assert_eq!(
            resolve_reference(&ctx, &graph, "/context", "/log.md").status,
            "missing-target"
        );
    }
    #[test]
    fn structural_resolution_uses_examined_revision_and_respects_scope() {
        let dir = fixture();
        std::fs::write(dir.path().join("b/index.md"), "# Bundle\n").unwrap();
        git_ok(dir.path(), &["init", "-q"]);
        git_ok(dir.path(), &["config", "user.name", "Test"]);
        git_ok(
            dir.path(),
            &["config", "user.email", "test@example.invalid"],
        );
        git_ok(dir.path(), &["add", "."]);
        git_ok(dir.path(), &["commit", "-qm", "initial"]);
        std::fs::remove_file(dir.path().join("b/index.md")).unwrap();
        std::fs::write(dir.path().join("b/log.md"), "current log\n").unwrap();
        let ctx = context(&dir);
        let historical = build_catalog_graph(&ctx, &[], true, Some("HEAD")).unwrap();
        assert_eq!(
            resolve_reference(&ctx, &historical, "acme.b:/context", "/index.md").status,
            "resolved"
        );
        assert_eq!(
            resolve_reference(&ctx, &historical, "acme.b:/context", "/log.md").status,
            "missing-target"
        );
        let scoped = build_catalog_graph(&ctx, &[], false, None).unwrap();
        assert_eq!(
            resolve_reference(&ctx, &scoped, "acme.b:/context", "/log.md").status,
            "out-of-scope"
        );
        let unavailable = build_catalog_graph(&ctx, &[], true, Some("missing-revision")).unwrap();
        assert_eq!(
            resolve_reference(&ctx, &unavailable, "acme.b:/context", "/log.md").status,
            "unavailable-root-or-revision"
        );
    }
    #[test]
    fn resource_impact_seeds_consumers_and_preserves_depth_and_concept_graph() {
        let dir = fixture();
        std::fs::write(dir.path().join("a/data.csv"), "1\n").unwrap();
        std::fs::write(dir.path().join("a/metrics/consumer.md"),
            "---\ntype: Metric\nsources:\n- {resource: data.csv, kind: file}\n- {resource: 'data.csv#L1', kind: line-range}\n- {resource: 'data.csv#heading', kind: markdown-heading}\n- {resource: '../data.csv'}\n---\n").unwrap();
        std::fs::write(
            dir.path().join("b/policies/consumer.md"),
            "---\ntype: Policy\nsources:\n- {resource: '../a/data.csv', kind: file}\n---\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("a/metrics/dependent.md"),
            "---\ntype: Metric\n---\n[Consumer](consumer.md)\n",
        )
        .unwrap();
        let ctx = context(&dir);
        let graph = build_catalog_graph(&ctx, &[], true, None).unwrap();
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.nodes.len(), 4);
        let changed = vec!["acme.a:/data.csv".into()];
        let ids = |nodes: Vec<Node>| {
            nodes
                .into_iter()
                .map(|node| format!("{}:{}", node.bundle, node.id))
                .collect::<Vec<_>>()
        };
        let direct = vec!["acme.a:/metrics/consumer", "acme.b:/policies/consumer"];
        assert_eq!(
            ids(graph.affected_resources(&ctx, &changed, false, None)),
            direct
        );
        assert_eq!(
            ids(graph.affected_resources(&ctx, &changed, true, Some(1))),
            direct
        );
        assert!(graph
            .affected_resources(&ctx, &changed, true, Some(0))
            .is_empty());
        assert_eq!(
            ids(graph.affected_resources(&ctx, &changed, true, Some(2))),
            vec![
                "acme.a:/metrics/consumer",
                "acme.a:/metrics/dependent",
                "acme.b:/policies/consumer"
            ]
        );
        let mixed = vec![
            "acme.a:/data.csv".into(),
            "acme.a:/metrics/consumer.md".into(),
        ];
        assert_eq!(
            ids(graph.affected_resources(&ctx, &mixed, true, None)),
            vec!["acme.a:/metrics/dependent", "acme.b:/policies/consumer"]
        );
        assert!(serde_json::to_value(&graph)
            .unwrap()
            .get("resource_dependencies")
            .is_none());
    }
}
