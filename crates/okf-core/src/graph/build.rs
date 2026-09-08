//! Build forward + reverse adjacency from concepts.
//!
//! The graph is the deterministic substrate for `backlinks`, `affected`, `stats`, and the
//! render emitters. Edges may point at ids that do not exist in the bundle (broken links are
//! recorded, per spec), so the reverse map is keyed by *every* referenced target, not only by
//! loaded concepts.
use crate::bundle::loader::Bundle;
use crate::model::concept::ConceptId;
use crate::model::link::outbound_links;
use crate::ontology::schema::Ontology;
use indexmap::{IndexMap, IndexSet};

/// Forward + reverse adjacency over a bundle's concepts.
#[derive(Debug, Clone, Default)]
pub struct LinkGraph {
    /// concept id → outbound target ids (order-preserving, deduped). Keyed only by concepts
    /// that exist in the bundle.
    pub forward: IndexMap<String, Vec<ConceptId>>,
    /// target id → concepts that link to it (sorted). Keyed by every referenced target,
    /// including broken (non-existent) ones.
    pub reverse: IndexMap<String, Vec<ConceptId>>,
    /// Ids that actually exist as loaded concepts.
    pub existing: IndexSet<String>,
}

impl LinkGraph {
    /// Outbound targets of `id` (empty if `id` is unknown or a leaf).
    pub fn outbound(&self, id: &str) -> &[ConceptId] {
        self.forward.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Concepts linking to `id` (empty if nothing points at it).
    pub fn inbound(&self, id: &str) -> &[ConceptId] {
        self.reverse.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Whether `id` is a loaded concept (vs. a broken-link target).
    pub fn exists(&self, id: &str) -> bool {
        self.existing.contains(id)
    }
}

/// Construct the [`LinkGraph`] for a bundle. Concepts are visited in bundle order (already
/// sorted by id in the loader); reverse adjacency lists and keys are sorted for determinism.
pub fn build_graph(bundle: &Bundle, ontology: Option<&Ontology>) -> LinkGraph {
    let mut forward: IndexMap<String, Vec<ConceptId>> = IndexMap::new();
    let mut reverse: IndexMap<String, Vec<ConceptId>> = IndexMap::new();
    let mut existing: IndexSet<String> = IndexSet::new();

    for c in &bundle.concepts {
        existing.insert(c.id.0.clone());
    }

    for c in &bundle.concepts {
        let outs = outbound_links(c, ontology);
        for target in &outs {
            reverse
                .entry(target.0.clone())
                .or_default()
                .push(c.id.clone());
        }
        forward.insert(c.id.0.clone(), outs);
    }

    for sources in reverse.values_mut() {
        sources.sort_by(|a, b| a.0.cmp(&b.0));
        sources.dedup_by(|a, b| a.0 == b.0);
    }
    reverse.sort_keys();

    LinkGraph {
        forward,
        reverse,
        existing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::concept::Concept;
    use crate::model::frontmatter::Frontmatter;
    use indexmap::IndexMap as Map;
    use serde_yaml::Value;

    fn concept(id: &str, yaml: &str, body: &str) -> Concept {
        let map: Map<String, Value> = serde_yaml::from_str(yaml).unwrap();
        Concept {
            id: ConceptId::from_relative(id),
            frontmatter: Frontmatter::from_map(map),
            body: body.to_string(),
        }
    }

    fn bundle(concepts: Vec<Concept>) -> Bundle {
        let mut concepts = concepts;
        concepts.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        Bundle {
            root: std::path::PathBuf::from("."),
            concepts,
        }
    }

    #[test]
    fn forward_and_reverse_including_broken() {
        let ontology = crate::ontology::load::parse_ontology(
            "okf_ontology: '0.1'\nconcepts:\n  T:\n    references:\n      refs: {target: T, cardinality: 0..n}\n"
        ).unwrap();
        let b = bundle(vec![
            concept("a", "type: T\nrefs:\n- /b", "[ghost](/missing.md)"),
            concept("b", "type: X", ""),
        ]);
        let g = build_graph(&b, Some(&ontology));
        let out: Vec<&str> = g.outbound("/a").iter().map(|c| c.0.as_str()).collect();
        assert_eq!(out, vec!["/b", "/missing"]);
        let inb: Vec<&str> = g.inbound("/b").iter().map(|c| c.0.as_str()).collect();
        assert_eq!(inb, vec!["/a"]);
        // Broken target still recorded in reverse, but is not "existing".
        assert_eq!(g.inbound("/missing").len(), 1);
        assert!(!g.exists("/missing"));
        assert!(g.exists("/b"));
    }
}
