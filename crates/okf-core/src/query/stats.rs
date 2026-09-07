//! Bundle summary: counts by type, trust-tier distribution, stale, orphans.
use crate::bundle::loader::Bundle;
use crate::graph::build::build_graph;
use crate::model::trust::TrustTier;
use std::collections::BTreeMap;

/// Type string used when a concept carries no (or an empty) `type`.
const UNTYPED: &str = "(untyped)";

/// Trust-tier counts across the bundle.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustDistribution {
    pub unverified: usize,
    pub machine_confirmed: usize,
    pub human_reviewed: usize,
}

/// A deterministic summary of a bundle (the `okf stats` payload).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stats {
    /// Total concepts loaded.
    pub total: usize,
    /// Concept counts keyed by `type` (untyped concepts bucket under `(untyped)`). Sorted.
    pub by_type: BTreeMap<String, usize>,
    /// Trust-tier distribution.
    pub trust: TrustDistribution,
    /// Stale (drifted) concept count. `None` until the fingerprint module lands.
    // TODO(fingerprint-wave): compute via `okf stale` once fingerprinting exists.
    pub stale: Option<usize>,
    /// Concepts with neither inbound nor outbound links.
    pub orphans: usize,
}

/// Compute the [`Stats`] summary for a bundle.
pub fn stats(bundle: &Bundle) -> Stats {
    let graph = build_graph(bundle);

    let mut by_type: BTreeMap<String, usize> = BTreeMap::new();
    let mut trust = TrustDistribution::default();
    let mut orphans = 0;

    for c in &bundle.concepts {
        let ty = match c.concept_type() {
            Some(t) if !t.is_empty() => t.to_string(),
            _ => UNTYPED.to_string(),
        };
        *by_type.entry(ty).or_insert(0) += 1;

        match c.trust_tier() {
            TrustTier::Unverified => trust.unverified += 1,
            TrustTier::MachineConfirmed => trust.machine_confirmed += 1,
            TrustTier::HumanReviewed => trust.human_reviewed += 1,
        }

        if graph.outbound(&c.id.0).is_empty() && graph.inbound(&c.id.0).is_empty() {
            orphans += 1;
        }
    }

    Stats {
        total: bundle.concepts.len(),
        by_type,
        trust,
        stale: None,
        orphans,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::concept::{Concept, ConceptId};
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
    fn summarizes_types_trust_and_orphans() {
        let bundle = Bundle {
            root: std::path::PathBuf::from("."),
            concepts: vec![
                concept("a", "type: Policy\nrefs:\n- /b"),
                concept("b", "type: Metric\nverified:\n- by: human:x"),
                concept("c", "type: Metric"), // orphan: no links either way
            ],
        };
        let s = stats(&bundle);
        assert_eq!(s.total, 3);
        assert_eq!(s.by_type.get("Policy"), Some(&1));
        assert_eq!(s.by_type.get("Metric"), Some(&2));
        assert_eq!(s.trust.unverified, 2);
        assert_eq!(s.trust.human_reviewed, 1);
        assert_eq!(s.orphans, 1); // c
        assert_eq!(s.stale, None);
    }

    #[test]
    fn untyped_bucket() {
        let bundle = Bundle {
            root: std::path::PathBuf::from("."),
            concepts: vec![concept("a", "title: no type")],
        };
        assert_eq!(stats(&bundle).by_type.get(UNTYPED), Some(&1));
    }
}
