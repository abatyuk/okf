//! Impact query: which concepts need review given a set of changed ids/links.
//!
//! Walks the *reverse* link graph out from the changed set — a concept is "affected" if it
//! (transitively) links to something that changed. Direct dependents only by default;
//! `transitive` follows the cascade, optionally capped at `depth` hops. The changed set itself
//! is never reported (those are the things that changed, not their reviewers), and the walk is
//! cycle-safe via a visited set.
use crate::graph::build::LinkGraph;
use crate::model::concept::ConceptId;
use std::collections::HashSet;

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
        .map(|c| ConceptId::from_relative(c).0)
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
        let bundle = Bundle {
            root: std::path::PathBuf::from("."),
            concepts: vec![
                concept("tables/customers", "refs:\n- /metrics/revenue"),
                concept("metrics/revenue", "refs:\n- /computations/mileage"),
                concept("computations/mileage", "refs:\n- /tables/customers"),
                concept("notes/orphan", "type: Note"),
            ],
        };
        build_graph(&bundle)
    }

    fn ids(v: Vec<ConceptId>) -> Vec<String> {
        v.into_iter().map(|c| c.0).collect()
    }

    #[test]
    fn direct_dependents_only_by_default() {
        let g = cyclic_graph();
        // who links to customers? mileage.
        let got = affected(&g, &["/tables/customers".into()], &AffectedOptions::default());
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
}
