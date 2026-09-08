//! Concepts linking to a given concept.
use crate::bundle::loader::Bundle;
use crate::graph::build::{build_graph, LinkGraph};
use crate::model::concept::ConceptId;
use crate::ontology::schema::Ontology;

/// Concepts that link *to* `id` (accepts an id with or without a leading slash), sorted by id.
/// Returns references into the graph's reverse adjacency.
pub fn backlinks<'a>(graph: &'a LinkGraph, id: &str) -> Vec<&'a ConceptId> {
    let key = ConceptId::from_relative(id).0;
    graph.inbound(&key).iter().collect()
}

/// Convenience: build the graph for `bundle` and return the backlink ids of `id` (owned).
pub fn backlinks_of(bundle: &Bundle, ontology: Option<&Ontology>, id: &str) -> Vec<ConceptId> {
    let graph = build_graph(bundle, ontology);
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
        let ontology = crate::ontology::load::parse_ontology(
            "okf_ontology: '0.1'\nconcepts:\n  T:\n    references:\n      refs: {target: T, cardinality: 0..n}\n      inputs: {target: T, cardinality: 0..n}\n  X: {}\n"
        ).unwrap();
        let bundle = Bundle {
            root: std::path::PathBuf::from("."),
            concepts: vec![
                concept("policies/travel", "type: T\nrefs:\n- /tables/customers"),
                concept(
                    "computations/mileage",
                    "type: T\ninputs:\n- /tables/customers",
                ),
                concept("tables/customers", "type: X"),
            ],
        };
        let g = build_graph(&bundle, Some(&ontology));
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
