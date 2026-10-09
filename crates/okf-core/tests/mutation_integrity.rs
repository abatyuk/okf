use okf_core::mutate::{add, body, edit, mv, rm};
use std::fs;
fn note(body: &str) -> String {
    format!("---\ntype: Note\n---\n{body}")
}

#[test]
fn failed_move_preserves_all_documents() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("old.md"), note("body\n")).unwrap();
    let reference = note("[old](old.md)\n");
    fs::write(root.path().join("ref.md"), &reference).unwrap();
    fs::write(root.path().join("blocked"), "file").unwrap();
    assert!(mv::mv(root.path(), "old", "blocked/new").is_err());
    assert_eq!(
        fs::read_to_string(root.path().join("ref.md")).unwrap(),
        reference
    );
    assert!(root.path().join("old.md").exists());
}

#[test]
fn moves_preserve_self_references_navigation_and_source_namespace() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("old.md"), "---\ntype: Note\nsources:\n- kind: file\n  resource: evidence/data.txt\n---\n[self](old.md)\n").unwrap();
    fs::write(
        root.path().join("index.md"),
        "[old][ref]\n\n[ref]: old.md \"Title\"\n",
    )
    .unwrap();
    fs::write(root.path().join("log.md"), "[old](old.md)\n").unwrap();
    assert!(rm::rm(root.path(), "old", false).is_err());
    mv::mv(root.path(), "old", "nested/new").unwrap();
    let moved = fs::read_to_string(root.path().join("nested/new.md")).unwrap();
    assert!(moved.contains("[self](new.md)"), "{moved}");
    assert!(moved.contains("resource: evidence/data.txt"), "{moved}");
    assert!(fs::read_to_string(root.path().join("index.md"))
        .unwrap()
        .contains("[ref]: nested/new.md"));
    assert!(fs::read_to_string(root.path().join("log.md"))
        .unwrap()
        .contains("(nested/new.md)"));
}

#[test]
fn section_edits_preserve_unrelated_fenced_whitespace() {
    let suffix = "## Other\n\n```\na\n\n\nb\n```\n\n\n";
    let doc = format!("## Target\n\nold\n\n{suffix}");
    for edited in [
        body::set_section(&doc, "Target", "new").unwrap(),
        body::append_section(&doc, "Target", "more").unwrap(),
        body::remove_section(&doc, "Target").unwrap(),
    ] {
        assert!(edited.ends_with(suffix));
    }
}

#[cfg(unix)]
#[test]
fn planted_legacy_temp_symlink_is_not_followed() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::NamedTempFile::new().unwrap();
    fs::write(outside.path(), "untouched").unwrap();
    fs::write(root.path().join("note.md"), note("body\n")).unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join("note.md.okf-tmp")).unwrap();
    edit::edit(
        root.path(),
        "note",
        &edit::EditSpec {
            set_body: Some("changed".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(fs::read_to_string(outside.path()).unwrap(), "untouched");
    assert!(!fs::symlink_metadata(root.path().join("note.md"))
        .unwrap()
        .file_type()
        .is_symlink());
}

#[test]
fn creation_rejects_discarded_arguments() {
    let root = tempfile::tempdir().unwrap();
    let opts = add::AddOptions {
        concept_type: Some("Note".into()),
        runtime: Some("python".into()),
        ..Default::default()
    };
    assert!(add::add(root.path(), "note", None, &opts).is_err());
    assert!(!root.path().join("note.md").exists());
}

#[test]
fn bare_declared_references_and_bundle_root_sources_follow_moves() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("ontology.yaml"), "okf_ontology: '0.1'\nconcepts:\n  Note:\n    references:\n      depends_on: {target: Note, cardinality: 0..1}\n").unwrap();
    fs::write(root.path().join("old.md"), note("body\n")).unwrap();
    fs::write(
        root.path().join("ref.md"),
        "---\ntype: Note\ndepends_on: old\n---\n",
    )
    .unwrap();
    fs::create_dir(root.path().join("nested")).unwrap();
    fs::write(
        root.path().join("nested/source.md"),
        "---\ntype: Note\nsources:\n- kind: file\n  resource: old.md\n---\n",
    )
    .unwrap();
    mv::mv(root.path(), "old", "nested/new").unwrap();
    assert!(fs::read_to_string(root.path().join("ref.md"))
        .unwrap()
        .contains("depends_on: nested/new"));
    assert!(fs::read_to_string(root.path().join("nested/source.md"))
        .unwrap()
        .contains("resource: nested/new.md"));
}
