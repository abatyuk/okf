use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("note.md"), "---\ntype: Note\n---\n").unwrap();
    fs::write(
        root.path().join("catalog.yaml"),
        "catalog_version: 1\nbundles:\n  acme.notes: {location: {type: directory, path: .}}\n",
    )
    .unwrap();
    fs::write(
        root.path().join("overrides.yaml"),
        "bundles: {acme.notes: .}\n",
    )
    .unwrap();
    fs::write(root.path().join("okf.toml"), "catalog='catalog.yaml'\ncatalog_overrides='overrides.yaml'\ndefault_bundle={id='acme.notes'}\n").unwrap();
    root
}

#[test]
fn active_catalog_and_override_files_cannot_be_removed_or_replaced() {
    let root = fixture();
    for path in ["catalog.yaml", "overrides.yaml"] {
        let before = fs::read(root.path().join(path)).unwrap();
        for operation in [
            format!("- op: remove\n  path: {path}\n"),
            format!(
                "- op: put-artifact\n  path: {path}\n  content: overwritten\n  replace: true\n"
            ),
        ] {
            let plan = format!("version: 1\noperations:\n{operation}");
            Command::new(assert_cmd::cargo::cargo_bin!("okf"))
                .current_dir(root.path())
                .env_remove("OKF_BUNDLE")
                .env_remove("OKF_CATALOG_OVERRIDES")
                .args(["changeset", "apply", "--from", &plan])
                .assert()
                .code(2)
                .stderr(predicate::str::contains("active tool configuration"));
            assert_eq!(fs::read(root.path().join(path)).unwrap(), before);
        }
    }
}

#[test]
fn environment_override_inside_bundle_is_protected() {
    let root = fixture();
    fs::write(
        root.path().join("machine.yaml"),
        "bundles: {acme.notes: .}\n",
    )
    .unwrap();
    let plan = "version: 1\noperations:\n- op: remove\n  path: machine.yaml\n";
    Command::new(assert_cmd::cargo::cargo_bin!("okf"))
        .current_dir(root.path())
        .env_remove("OKF_BUNDLE")
        .env("OKF_CATALOG_OVERRIDES", "machine.yaml")
        .args(["changeset", "plan", "--from", plan])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "active tool configuration: machine.yaml",
        ));
    assert!(root.path().join("machine.yaml").exists());
}
