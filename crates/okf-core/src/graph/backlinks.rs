//! Concepts linking to a given concept.
use crate::bundle::loader::Bundle;
use crate::graph::build::{build_graph, LinkGraph};
use crate::model::concept::ConceptId;

/// Concepts that link *to* `id` (accepts an id with or without a leading slash), sorted by id.
/// Returns references into the graph's reverse adjacency.
pub fn backlinks<'a>(graph: &'a LinkGraph, id: &str) -> Vec<&'a ConceptId> {
    let key = ConceptId::from_relative(id).0;
    graph.inbound(&key).iter().collect()
}

/// Convenience: build the graph for `bundle` and return the backlink ids of `id` (owned).
pub fn backlinks_of(bundle: &Bundle, id: &str) -> Vec<ConceptId> {
    let graph = build_graph(bundle);
    backlinks(&graph, id).into_iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn returns_linkers_sorted() {
        let bundle = Bundle {
            root: std::path::PathBuf::from("."),
            concepts: vec![
                concept("policies/travel", "refs:\n- /tables/customers"),
                concept("computations/mileage", "inputs:\n- /tables/customers"),
                concept("tables/customers", "type: X"),
            ],
        };
        let g = build_graph(&bundle);
        let bl: Vec<&str> = backlinks(&g, "tables/customers")
            .iter()
            .map(|c| c.0.as_str())
            .collect();
        assert_eq!(bl, vec!["/computations/mileage", "/policies/travel"]);
        // leading slash accepted too
        assert_eq!(backlinks(&g, "/tables/customers").len(), 2);
        // nothing links to travel
        assert!(backlinks(&g, "policies/travel").is_empty());
    }
}
