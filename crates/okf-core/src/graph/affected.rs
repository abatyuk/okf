//! Impact query: which concepts need review given a set of changed ids/links.
//!
//! Walks the *reverse* link graph out from the changed set — a concept is "affected" if it
//! (transitively) links to something that changed. Direct dependents only by default;
//! `transitive` follows the cascade, optionally capped at `depth` hops. The changed set itself
//! is never reported (those are the things that changed, not their reviewers), and the walk is
//! cycle-safe via a visited set.
use crate::bundle::loader::Bundle;
use crate::graph::build::LinkGraph;
use crate::model::concept::ConceptId;
use crate::model::link::{classify, resolve_link, LinkKind};
use serde_yaml::Value;
use std::collections::{HashMap, HashSet};

/// Options for [`affected`].
#[derive(Debug, Clone, Default)]
pub struct AffectedOptions {
    /// Follow the cascade past direct dependents.
    pub transitive: bool,
    /// Cap the number of hops when `transitive`. `None` = unbounded. Ignored when
    /// `transitive` is false (which is always a single hop).
    pub depth: Option<usize>,
}

/// Concepts needing review given `changed` (ids or links, leading slash optional), sorted by
/// id. Walks reverse edges: direct dependents by default; the full cascade when
/// `opts.transitive`, capped at `opts.depth` hops when set.
pub fn affected(graph: &LinkGraph, changed: &[String], opts: &AffectedOptions) -> Vec<ConceptId> {
    let seeds: Vec<String> = changed
        .iter()
        // Inputs may be concept IDs or ordinary bundle-relative links, including .md,
        // fragments and query suffixes. Normalize them like the graph's target edges.
        .map(|c| crate::model::link::resolve_link(&ConceptId("/".into()), c).0)
        .collect();

    let max_hops = if opts.transitive {
        opts.depth.unwrap_or(usize::MAX)
    } else {
        1
    };

    // Seeds are visited (so cycles back to them terminate) but never emitted.
    let mut visited: HashSet<String> = seeds.iter().cloned().collect();
    let mut frontier: Vec<String> = seeds.clone();
    frontier.sort();
    frontier.dedup();

    let mut result: Vec<ConceptId> = Vec::new();
    let mut hop = 0;
    while hop < max_hops && !frontier.is_empty() {
        hop += 1;
        let mut next: Vec<String> = Vec::new();
        for node in &frontier {
            for dep in graph.inbound(node) {
                if visited.insert(dep.0.clone()) {
                    result.push(dep.clone());
                    next.push(dep.0.clone());
                }
            }
        }
        next.sort();
        frontier = next;
    }

    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// Include local source-resource dependencies in an impact query. These edges are private to
/// this traversal: opaque files must not become concept edges for stats, orphans, or backlinks.
/// File/text fingerprint kinds use bundle-root paths; other local sources use document paths.
pub fn affected_with_sources(
    bundle: &Bundle,
    graph: &LinkGraph,
    changed: &[String],
    opts: &AffectedOptions,
) -> Vec<ConceptId> {
    let mut impact = graph.clone();
    let root = std::fs::canonicalize(&bundle.root).unwrap_or_else(|_| {
        crate::bundle::catalog::absolute(&bundle.root).unwrap_or_else(|_| bundle.root.clone())
    });
    let mut resource_dependents = HashMap::<std::path::PathBuf, Vec<ConceptId>>::new();
    for concept in &bundle.concepts {
        let Some(Value::Sequence(sources)) = concept.frontmatter.get("sources") else {
            continue;
        };
        for source in sources {
            let Some(resource) = source.get("resource").and_then(Value::as_str) else {
                continue;
            };
            let Some(path) = crate::graph::catalog::source_dependency_path(
                &root,
                &concept.id.0,
                resource,
                source.get("kind").and_then(Value::as_str),
            ) else {
                continue;
            };
            let dependents = resource_dependents.entry(path).or_default();
            if !dependents.contains(&concept.id) {
                dependents.push(concept.id.clone());
            }
        }
    }
    for raw in changed {
        if classify(raw) == LinkKind::External {
            continue;
        }
        let clean = raw.split(['#', '?']).next().unwrap_or(raw);
        let path = crate::bundle::catalog::normalize(&root.join(clean.trim_start_matches('/')));
        let mut paths = vec![std::fs::canonicalize(&path).unwrap_or(path.clone())];
        if path.extension().is_none() {
            let document = path.with_extension("md");
            paths.push(std::fs::canonicalize(&document).unwrap_or(document));
        }
        let seed = resolve_link(&ConceptId("/".into()), raw);
        for path in paths {
            if let Some(dependents) = resource_dependents.get(&path) {
                let inbound = impact.reverse.entry(seed.0.clone()).or_default();
                for dependent in dependents {
                    if !inbound.contains(dependent) {
                        inbound.push(dependent.clone());
                    }
                }
            }
        }
    }
    affected(&impact, changed, opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::loader::Bundle;
    use crate::graph::build::build_graph;
    use crate::model::concept::Concept;
    use crate::model::frontmatter::Frontmatter;
    use indexmap::IndexMap;
    use serde_yaml::Value;

    fn concept(id: &str, yaml: &str) -> Concept {
        let map: IndexMap<String, Value> = serde_yaml::from_str(yaml).unwrap();
        Concept {
            id: ConceptId::from_relative(id),
            frontmatter: Frontmatter::from_map(map),
            body: String::new(),
        }
    }

    /// Cycle: customers → revenue → mileage → customers (via the fields below).
    fn cyclic_graph() -> LinkGraph {
        let ontology = crate::ontology::load::parse_ontology(
            "okf_ontology: '0.1'\nconcepts:\n  T:\n    references:\n      refs: {target: T, cardinality: 0..n}\n  Note: {}\n"
        ).unwrap();
        let bundle = Bundle {
            root: std::path::PathBuf::from("."),
            concepts: vec![
                concept("tables/customers", "type: T\nrefs:\n- /metrics/revenue"),
                concept("metrics/revenue", "type: T\nrefs:\n- /computations/mileage"),
                concept(
                    "computations/mileage",
                    "type: T\nrefs:\n- /tables/customers",
                ),
                concept("notes/orphan", "type: Note"),
            ],
        };
        build_graph(&bundle, Some(&ontology))
    }

    fn ids(v: Vec<ConceptId>) -> Vec<String> {
        v.into_iter().map(|c| c.0).collect()
    }

    #[test]
    fn direct_dependents_only_by_default() {
        let g = cyclic_graph();
        // who links to customers? mileage.
        let got = affected(
            &g,
            &["/tables/customers".into()],
            &AffectedOptions::default(),
        );
        assert_eq!(ids(got), vec!["/computations/mileage".to_string()]);
    }

    #[test]
    fn transitive_follows_cascade_and_terminates_on_cycle() {
        let g = cyclic_graph();
        let opts = AffectedOptions {
            transitive: true,
            depth: None,
        };
        let got = affected(&g, &["/tables/customers".into()], &opts);
        // mileage (direct) → revenue (links mileage) → customers (seed, skipped).
        assert_eq!(
            ids(got),
            vec![
                "/computations/mileage".to_string(),
                "/metrics/revenue".to_string(),
            ]
        );
    }

    #[test]
    fn depth_caps_hops() {
        let g = cyclic_graph();
        let opts = AffectedOptions {
            transitive: true,
            depth: Some(1),
        };
        let got = affected(&g, &["/tables/customers".into()], &opts);
        assert_eq!(ids(got), vec!["/computations/mileage".to_string()]);
    }

    #[test]
    fn resource_dependencies_use_kind_paths_and_preserve_depth_without_concept_edges() {
        let bundle = Bundle {
            root: ".".into(),
            concepts: vec![
                concept("notes/direct", "type: Note\nsources:\n- {kind: file, resource: data.csv}\n- {kind: line-range, resource: 'data.csv#L1-2'}\n- {resource: '../data.csv'}\n- {resource: 'https://example.com/data.csv'}"),
                concept("notes/second", "type: Note\nsources:\n- {resource: direct.md}"),
                concept("notes/third", "type: Note\nsources:\n- {resource: second.md}"),
                concept("notes/unrelated", "type: Note\nsources:\n- {resource: data.csv}"),
            ],
        };
        let graph = build_graph(&bundle, None);
        let before = graph.forward.clone();
        assert!(graph.inbound("/data.csv").is_empty());
        let changed = vec!["./data.csv#L2".into()];
        assert_eq!(
            ids(affected_with_sources(
                &bundle,
                &graph,
                &changed,
                &AffectedOptions::default()
            )),
            vec!["/notes/direct"]
        );
        assert_eq!(
            ids(affected_with_sources(
                &bundle,
                &graph,
                &changed,
                &AffectedOptions {
                    transitive: true,
                    depth: Some(2)
                }
            )),
            vec!["/notes/direct", "/notes/second"]
        );
        assert_eq!(
            ids(affected_with_sources(
                &bundle,
                &graph,
                &changed,
                &AffectedOptions {
                    transitive: true,
                    depth: None
                }
            )),
            vec!["/notes/direct", "/notes/second", "/notes/third"]
        );
        assert!(affected_with_sources(
            &bundle,
            &graph,
            &changed,
            &AffectedOptions {
                transitive: true,
                depth: Some(0)
            }
        )
        .is_empty());
        assert_eq!(graph.forward, before);
        assert!(graph.inbound("/data.csv").is_empty());
    }
}
