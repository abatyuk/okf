//! Regression coverage for creation-time bodies and focused body edits.
use assert_cmd::prelude::*;
use std::process::Command;

fn okf() -> Command {
    Command::cargo_bin("okf").unwrap()
}

#[test]
fn add_body_and_edit_replace_and_rename_section() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().to_str().unwrap();
    let body_path = dir.path().join("body.md");
    let authored = "# Doc\n\nold phrase\n\n## Rates\n\nold phrase\n";
    std::fs::write(&body_path, authored).unwrap();

    okf()
        .args([
            "add",
            "notes/doc",
            bundle,
            "--type",
            "Note",
            "--title",
            "Doc",
            "--description",
            "Test",
            "--body",
            &format!("@{}", body_path.display()),
        ])
        .assert()
        .success();

    let path = dir.path().join("notes/doc.md");
    let before = std::fs::read_to_string(&path).unwrap();
    assert!(before.ends_with(authored));

    // Default behavior is guarded: ambiguous replacements fail without modifying the file.
    okf()
        .args([
            "edit",
            "notes/doc",
            bundle,
            "--replace",
            "old phrase",
            "new phrase",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("use --all"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);

    okf()
        .args([
            "edit",
            "notes/doc",
            bundle,
            "--replace",
            "old phrase",
            "new phrase",
            "--all",
            "--rename-section",
            "Rates",
            "Pricing",
        ])
        .assert()
        .success();

    let after = std::fs::read_to_string(path).unwrap();
    assert_eq!(after.matches("new phrase").count(), 2);
    assert!(!after.contains("old phrase"));
    assert!(after.contains("## Pricing\n\nnew phrase"));
    assert!(!after.contains("## Rates"));
}

#[test]
fn replace_requires_a_match_and_exactly_one_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().to_str().unwrap();
    std::fs::write(
        dir.path().join("note.md"),
        "---\ntype: Note\ntitle: Note\n---\n# Note\n\nunique\n",
    )
    .unwrap();
    let before = std::fs::read_to_string(dir.path().join("note.md")).unwrap();

    okf()
        .args(["edit", "note", bundle, "--replace", "absent", "x"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("text not found"));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("note.md")).unwrap(),
        before
    );

    okf()
        .args(["edit", "note", bundle, "--replace", "unique", "changed"])
        .assert()
        .success();
    assert!(std::fs::read_to_string(dir.path().join("note.md"))
        .unwrap()
        .contains("changed"));
}

#[test]
fn missing_add_body_file_does_not_leave_a_skeleton() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().to_str().unwrap();
    let missing = dir.path().join("missing.md");
    okf()
        .args([
            "add",
            "notes/unfinished",
            bundle,
            "--type",
            "Note",
            "--title",
            "Unfinished",
            "--description",
            "Test",
            "--body",
            &format!("@{}", missing.display()),
        ])
        .assert()
        .failure();
    assert!(!dir.path().join("notes/unfinished.md").exists());
}

#[test]
fn failed_replacement_preserves_verified_history() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().to_str().unwrap();
    let path = dir.path().join("note.md");
    let original = "---\ntype: Note\ntitle: Note\nverified:\n- by: human:reviewer\n  at: 2026-09-27T12:00:00Z\n---\n# Note\n\nunique\n";
    std::fs::write(&path, original).unwrap();

    okf()
        .args(["edit", "note", bundle, "--replace", "absent", "changed"])
        .assert()
        .failure();
    assert_eq!(std::fs::read_to_string(path).unwrap(), original);
}
