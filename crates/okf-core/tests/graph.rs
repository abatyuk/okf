//! Integration tests for the link graph + query(resolve, stats) layer over a fixture bundle
//! with frontmatter reference fields, body markdown links, a broken link, a cycle, and an
//! orphan.
use std::path::PathBuf;

use okf_core::bundle::loader::load_bundle;
use okf_core::graph::affected::{affected, AffectedOptions};
use okf_core::graph::backlinks::backlinks_of;
use okf_core::graph::build::build_graph;
use okf_core::graph::render::{render, RenderFormat};
use okf_core::query::resolve::resolve;
use okf_core::query::stats::stats;

fn linked_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/linked-bundle")
}

fn ids(v: Vec<okf_core::model::concept::ConceptId>) -> Vec<String> {
    v.into_iter().map(|c| c.0).collect()
}

#[test]
fn outbound_edges_from_frontmatter_and_body_including_broken() {
    let bundle = load_bundle(&linked_root()).unwrap();
    let g = build_graph(&bundle);

    // Policy pulls its ref from frontmatter (computations) AND body links; dedups the
    // repeated mileage link, keeps the broken ghost, drops the external URL.
    let out: Vec<&str> = g
        .outbound("/policies/travel")
        .iter()
        .map(|c| c.0.as_str())
        .collect();
    assert_eq!(
        out,
        vec!["/computations/mileage", "/tables/customers", "/tables/ghost"]
    );

    // Broken target exists in reverse but is not a loaded concept.
    assert!(!g.exists("/tables/ghost"));
    assert_eq!(g.inbound("/tables/ghost").len(), 1);
}

#[test]
fn backlinks_are_sorted_linkers() {
    let bundle = load_bundle(&linked_root()).unwrap();
    assert_eq!(
        ids(backlinks_of(&bundle, "tables/customers")),
        vec![
            "/computations/mileage".to_string(),
            "/policies/travel".to_string(),
        ]
    );
}

#[test]
fn affected_direct_transitive_and_depth_capped() {
    let bundle = load_bundle(&linked_root()).unwrap();
    let g = build_graph(&bundle);
    let changed = vec!["/tables/customers".to_string()];

    // Direct dependents only.
    assert_eq!(
        ids(affected(&g, &changed, &AffectedOptions::default())),
        vec![
            "/computations/mileage".to_string(),
            "/policies/travel".to_string(),
        ]
    );

    // Transitive walk terminates despite the customers→revenue→mileage→customers cycle.
    assert_eq!(
        ids(affected(
            &g,
            &changed,
            &AffectedOptions { transitive: true, depth: None }
        )),
        vec![
            "/computations/mileage".to_string(),
            "/metrics/revenue".to_string(),
            "/policies/travel".to_string(),
        ]
    );

    // Depth cap of 1 == direct.
    assert_eq!(
        ids(affected(
            &g,
            &changed,
            &AffectedOptions { transitive: true, depth: Some(1) }
        )),
        vec![
            "/computations/mileage".to_string(),
            "/policies/travel".to_string(),
        ]
    );
}

#[test]
fn resolve_maps_id_to_path_and_flags_broken() {
    let bundle = load_bundle(&linked_root()).unwrap();

    let ok = resolve(&bundle, None, "/tables/customers.md");
    assert_eq!(ok.path, PathBuf::from("tables/customers.md"));
    assert!(ok.exists);

    let broken = resolve(&bundle, None, "/tables/ghost");
    assert_eq!(broken.path, PathBuf::from("tables/ghost.md"));
    assert!(!broken.exists);
}

#[test]
fn stats_summarize_bundle() {
    let bundle = load_bundle(&linked_root()).unwrap();
    let s = stats(&bundle);
    assert_eq!(s.total, 5);
    assert_eq!(s.by_type.get("BigQuery Table"), Some(&1));
    assert_eq!(s.by_type.get("Policy"), Some(&1));
    assert_eq!(s.trust.human_reviewed, 1); // customers
    assert_eq!(s.trust.machine_confirmed, 1); // revenue
    assert_eq!(s.trust.unverified, 3); // mileage, orphan, travel
    assert_eq!(s.orphans, 1); // notes/orphan
    assert_eq!(s.stale, None);
}

#[test]
fn render_formats_are_stable() {
    let bundle = load_bundle(&linked_root()).unwrap();
    let g = build_graph(&bundle);

    // Subtree forward-reachable from customers is the whole cycle
    // {customers, revenue, mileage} (mileage → customers closes it).
    let mermaid = render(&g, RenderFormat::Mermaid, Some("tables/customers"));
    assert_eq!(
        mermaid,
        "graph LR\n    \
         n0[\"/computations/mileage\"]\n    \
         n1[\"/metrics/revenue\"]\n    \
         n2[\"/tables/customers\"]\n    \
         n0 --> n2\n    \
         n1 --> n0\n    \
         n2 --> n1\n"
    );

    // Full-graph DOT/GraphML render without panicking and contain expected fragments.
    let dot = render(&g, RenderFormat::Dot, None);
    assert!(dot.starts_with("digraph okf {\n"));
    assert!(dot.trim_end().ends_with('}'));

    let graphml = render(&g, RenderFormat::Graphml, None);
    assert!(graphml.contains("<graph id=\"okf\" edgedefault=\"directed\">"));
}
