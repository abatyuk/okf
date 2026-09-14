//! End-to-end tests for the foundational slice: load, list, show, search, round-trip.
use std::path::PathBuf;

use okf_core::bundle::loader::load_bundle;
use okf_core::model::trust::TrustTier;
use okf_core::output::record::concept_record;
use okf_core::parse::{parse_concept, writer};
use okf_core::query::search::{search, SearchFilter};
use okf_core::query::show::show;

fn bundle_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample-bundle")
}

#[test]
fn loads_concepts_and_skips_reserved() {
    let bundle = load_bundle(&bundle_root()).unwrap();
    let ids: Vec<&str> = bundle.concepts.iter().map(|c| c.id.0.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "/metrics/revenue",
            "/notes/roundtrip",
            "/policies/travel",
            "/tables/customers",
        ],
        "reserved index.md must be skipped and ids sorted"
    );
}

#[test]
fn trust_tiers_are_derived() {
    let bundle = load_bundle(&bundle_root()).unwrap();
    let tier = |id: &str| show(&bundle, id).unwrap().trust_tier();
    assert_eq!(tier("tables/customers"), TrustTier::HumanReviewed);
    assert_eq!(tier("metrics/revenue"), TrustTier::MachineConfirmed);
    assert_eq!(tier("policies/travel"), TrustTier::Unverified);
}

#[test]
fn show_accepts_leading_slash_or_not() {
    let bundle = load_bundle(&bundle_root()).unwrap();
    assert!(show(&bundle, "tables/customers").is_some());
    assert!(show(&bundle, "/tables/customers").is_some());
    assert!(show(&bundle, "does/not/exist").is_none());
}

#[test]
fn search_filters_by_type_tag_text() {
    let bundle = load_bundle(&bundle_root()).unwrap();

    let by_type = search(
        &bundle,
        &SearchFilter {
            type_: Some("Policy".to_string()),
            ..Default::default()
        },
    );
    assert_eq!(by_type.len(), 1);
    assert_eq!(by_type[0].id.0, "/policies/travel");

    let by_tag = search(
        &bundle,
        &SearchFilter {
            tag: Some("finance".to_string()),
            ..Default::default()
        },
    );
    let mut tagged: Vec<&str> = by_tag.iter().map(|c| c.id.0.as_str()).collect();
    tagged.sort();
    assert_eq!(tagged, vec!["/metrics/revenue", "/policies/travel"]);

    let by_text = search(
        &bundle,
        &SearchFilter {
            text: vec!["REIMBURSEMENT".to_string()], // case-insensitive
            ..Default::default()
        },
    );
    assert_eq!(by_text.len(), 1);
    assert_eq!(by_text[0].id.0, "/policies/travel");

    // Empty filter (== list) returns everything.
    assert_eq!(search(&bundle, &SearchFilter::default()).len(), 4);
}

#[test]
fn concept_record_mirrors_frontmatter_plus_computed() {
    let bundle = load_bundle(&bundle_root()).unwrap();
    let c = show(&bundle, "tables/customers").unwrap();
    let rec = concept_record(c);
    let obj = rec.as_object().unwrap();

    // Computed fields present.
    assert_eq!(obj.get("id").unwrap(), "/tables/customers");
    assert_eq!(obj.get("trust_tier").unwrap(), "human-reviewed");
    // Frontmatter mirrored verbatim.
    assert_eq!(obj.get("type").unwrap(), "BigQuery Table");
    assert_eq!(
        obj.get("tags").unwrap(),
        &serde_json::json!(["sales", "core"])
    );
    // id comes first; frontmatter order is preserved before computed semantic views.
    let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    assert_eq!(keys.first(), Some(&"id"));
    assert!(keys.contains(&"effective_status"));
    assert!(keys.contains(&"verification_current"));
    let type_pos = keys.iter().position(|k| *k == "type").unwrap();
    let title_pos = keys.iter().position(|k| *k == "title").unwrap();
    assert!(type_pos < title_pos, "frontmatter order preserved");
}

#[test]
fn lossless_round_trip_is_byte_stable() {
    let path = bundle_root().join("notes/roundtrip.md");
    let original = std::fs::read_to_string(&path).unwrap();

    let id = okf_core::model::concept::ConceptId::from_relative("notes/roundtrip");
    let c1 = parse_concept(id.clone(), &original).unwrap();

    // Unknown keys preserved, in original order.
    let keys: Vec<&str> = c1.frontmatter.map.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        vec![
            "type",
            "zeta_unknown",
            "title",
            "custom_order",
            "tags",
            "nested_unknown",
        ]
    );

    let s1 = writer::write_concept(&c1).unwrap();
    // Byte-stable versus the on-disk file (modulo a trailing newline).
    assert_eq!(
        s1.trim_end_matches('\n'),
        original.trim_end_matches('\n'),
        "round-trip must be byte-stable\n--- got ---\n{s1}\n--- want ---\n{original}"
    );

    // And idempotent on a second pass.
    let c2 = parse_concept(id, &s1).unwrap();
    let s2 = writer::write_concept(&c2).unwrap();
    assert_eq!(s1, s2);
    assert_eq!(c1.frontmatter, c2.frontmatter);
    assert_eq!(c1.body, c2.body);
}
