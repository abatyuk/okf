//! Structured references participate in transactional moves.
use okf_core::mutate::mv::{mv, mv_with_ontology};
use okf_core::ontology::load::parse_ontology;
use std::fs;

const ONTOLOGY: &str = "okf_ontology: '0.1'\nconcepts:\n  Note:\n    references:\n      related:\n        selector: relations[].target\n        target: Note\n        cardinality: 0..n\n";

#[test]
fn move_rewrites_nested_inbound_references() {
    let root = tempfile::tempdir().unwrap();
    let note = "---\ntype: Note\n---\nOriginal\n";
    let referrer = "---\ntype: Note\nrelations: [{target: /old.md}]\n---\n[old](/old.md)\n";
    fs::write(root.path().join("ontology.yaml"), ONTOLOGY).unwrap();
    fs::write(root.path().join("old.md"), note).unwrap();
    fs::write(root.path().join("referrer.md"), referrer).unwrap();
    mv(root.path(), "old", "new").unwrap();
    assert!(!root.path().join("old.md").exists());
    let changed = fs::read_to_string(root.path().join("referrer.md")).unwrap();
    assert!(changed.contains("target: /new.md"));
    assert!(changed.contains("[old](/new.md)"));
    assert!(root.path().join("new.md").exists());
}

#[test]
fn sidecar_move_rebases_outbound_relative_reference() {
    let root = tempfile::tempdir().unwrap();
    let ontology = parse_ontology(ONTOLOGY).unwrap();
    let note = "---\ntype: Note\nrelations: [{target: peer}]\n---\nBody\n";
    fs::write(root.path().join("old.md"), note).unwrap();
    fs::write(root.path().join("peer.md"), "---\ntype: Note\n---\n").unwrap();
    mv_with_ontology(root.path(), "old", "nested/new", Some(&ontology)).unwrap();
    let changed = fs::read_to_string(root.path().join("nested/new.md")).unwrap();
    assert!(changed.contains("target: ../peer"));
    assert!(!root.path().join("old.md").exists());
}

#[test]
fn unaffected_structured_references_do_not_block_moves() {
    let root = tempfile::tempdir().unwrap();
    let ontology = parse_ontology(ONTOLOGY).unwrap();
    fs::write(
        root.path().join("old.md"),
        "---\ntype: Note\nrelations: [{target: /peer}]\n---\n",
    )
    .unwrap();
    fs::write(root.path().join("peer.md"), "---\ntype: Note\n---\n").unwrap();
    mv_with_ontology(root.path(), "old", "nested/new", Some(&ontology)).unwrap();
    assert!(!root.path().join("old.md").exists());
    assert!(fs::read_to_string(root.path().join("nested/new.md"))
        .unwrap()
        .contains("/peer"));
}

#[test]
fn sidecar_remove_sees_nested_backlinks_and_requires_force() {
    use okf_core::mutate::rm::rm_with_ontology;
    let root = tempfile::tempdir().unwrap();
    let ontology = parse_ontology(ONTOLOGY).unwrap();
    fs::write(root.path().join("old.md"), "---\ntype: Note\n---\n").unwrap();
    fs::write(
        root.path().join("referrer.md"),
        "---\ntype: Note\nrelations: [{target: /old}]\n---\n",
    )
    .unwrap();
    assert!(rm_with_ontology(root.path(), "old", false, Some(&ontology)).is_err());
    assert!(root.path().join("old.md").exists());
    let result = rm_with_ontology(root.path(), "old", true, Some(&ontology)).unwrap();
    assert_eq!(result.dangling_referrers.len(), 1);
    assert_eq!(result.dangling_referrers[0].0, "/referrer");
}
