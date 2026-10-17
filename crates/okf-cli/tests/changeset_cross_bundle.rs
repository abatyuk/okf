use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use std::fs;

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for name in ["planning", "knowledge", "unregistered"] {
        fs::create_dir(dir.path().join(name)).unwrap();
    }
    fs::write(dir.path().join("okf.toml"), "catalog='catalog.yaml'\n").unwrap();
    fs::write(dir.path().join("catalog.yaml"), "catalog_version: 1\nbundles:\n  acme.planning: {location: {type: directory, path: planning}}\n  acme.knowledge: {location: {type: directory, path: knowledge}}\n").unwrap();
    fs::write(dir.path().join("planning/ontology.yaml"), "okf_ontology: '0.1'\nconcepts:\n  Task:\n    references:\n      concepts:\n        target: [Task, 'acme.knowledge:Component']\n        cardinality: 0..n\n").unwrap();
    fs::write(
        dir.path().join("knowledge/target.md"),
        "---\ntype: Component\n---\n# Target\n",
    )
    .unwrap();
    dir
}

fn task(reference: &str) -> String {
    format!("---\ntype: Task\nconcepts: ['{reference}']\n---\n[target]({reference})\n")
}

fn command(dir: &tempfile::TempDir, verb: &str, operations: serde_json::Value) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("okf"));
    command.current_dir(dir.path()).args([
        "changeset",
        verb,
        "--bundle-id",
        "acme.planning",
        "--json",
        "--from",
        &json!({"version": 1, "operations": operations}).to_string(),
    ]);
    command
}

#[test]
fn new_registered_sibling_reference_is_previewed_and_applied_without_sibling_writes() {
    let dir = fixture();
    let target = fs::read(dir.path().join("knowledge/target.md")).unwrap();
    let operations = json!([{"op":"create","path":"new","content":task("../knowledge/target.md")}]);
    command(&dir, "plan", operations.clone()).assert().success();
    assert!(!dir.path().join("planning/new.md").exists());
    command(&dir, "apply", operations).assert().success();
    assert!(fs::read_to_string(dir.path().join("planning/new.md"))
        .unwrap()
        .contains("../knowledge/target.md"));
    assert_eq!(
        fs::read(dir.path().join("knowledge/target.md")).unwrap(),
        target
    );
    assert_eq!(
        fs::read_dir(dir.path().join("knowledge")).unwrap().count(),
        1
    );
}

#[test]
fn move_rebases_registered_sibling_reference_without_aliasing_a_local_target() {
    let dir = fixture();
    fs::write(
        dir.path().join("planning/old.md"),
        task("../knowledge/target.md#section"),
    )
    .unwrap();
    fs::create_dir(dir.path().join("planning/knowledge")).unwrap();
    fs::write(
        dir.path().join("planning/knowledge/target.md"),
        "---\ntype: Task\n---\n# Local\n",
    )
    .unwrap();
    let operations = json!([{"op":"move","from":"old","to":"nested/new"}, {"op":"move","from":"knowledge/target","to":"local-renamed"}]);
    command(&dir, "apply", operations).assert().success();
    let moved = fs::read_to_string(dir.path().join("planning/nested/new.md")).unwrap();
    assert!(
        moved.contains("../../knowledge/target.md#section"),
        "{moved}"
    );
    assert!(!moved.contains("local-renamed"), "{moved}");
    assert!(!dir.path().join("planning/old.md").exists());
    assert!(dir.path().join("knowledge/target.md").exists());
}

#[test]
fn nonexistent_registered_target_is_rejected_without_publication() {
    let dir = fixture();
    command(
        &dir,
        "apply",
        json!([{"op":"create","path":"new","content":task("../knowledge/missing.md")}]),
    )
    .assert()
    .code(2)
    .stderr(predicate::str::contains("dangling/blocked"));
    assert!(!dir.path().join("planning/new.md").exists());
}

#[test]
fn unregistered_existing_target_and_standalone_escape_remain_blocked() {
    let dir = fixture();
    fs::write(
        dir.path().join("unregistered/target.md"),
        "---\ntype: Task\n---\n",
    )
    .unwrap();
    for reference in [
        "../unregistered/target.md",
        "../../outside.md",
        "C:/../../knowledge/target.md",
    ] {
        command(
            &dir,
            "apply",
            json!([{"op":"create","path":"new","content":task(reference)}]),
        )
        .assert()
        .code(2)
        .stderr(predicate::str::contains("dangling/blocked"));
    }
    fs::remove_file(dir.path().join("okf.toml")).unwrap();
    Command::new(assert_cmd::cargo::cargo_bin!("okf"))
        .current_dir(dir.path()).args(["changeset", "apply", "planning", "--from", &json!({"version":1,"operations":[{"op":"create","path":"new","content":task("../knowledge/target.md")}]}).to_string()])
        .assert().code(2).stderr(predicate::str::contains("dangling/blocked"));
    assert!(!dir.path().join("planning/new.md").exists());
}

#[cfg(unix)]
#[test]
fn registered_sibling_symlink_escape_remains_blocked() {
    let dir = fixture();
    fs::write(
        dir.path().join("unregistered/secret.md"),
        "---\ntype: Component\n---\n",
    )
    .unwrap();
    std::os::unix::fs::symlink(
        dir.path().join("unregistered/secret.md"),
        dir.path().join("knowledge/escape.md"),
    )
    .unwrap();
    command(
        &dir,
        "apply",
        json!([{"op":"create","path":"new","content":task("../knowledge/escape.md")}]),
    )
    .assert()
    .code(2)
    .stderr(predicate::str::contains("dangling/blocked"));
    assert!(!dir.path().join("planning/new.md").exists());
}

#[test]
fn registered_sibling_typed_file_source_is_checked_from_bundle_root() {
    let dir = fixture();
    fs::write(dir.path().join("knowledge/data.csv"), "a,b\n").unwrap();
    let content =
        "---\ntype: Task\nsources:\n- kind: file\n  resource: ../knowledge/data.csv\n---\n# New\n";
    command(
        &dir,
        "apply",
        json!([{"op":"create","path":"nested/new","content":content}]),
    )
    .assert()
    .success();
}

#[test]
fn sibling_registration_does_not_authorize_cross_bundle_writes() {
    let dir = fixture();
    let original = fs::read(dir.path().join("knowledge/target.md")).unwrap();
    for operation in [
        json!({"op":"remove", "path":"../knowledge/target.md"}),
        json!({"op":"create", "path":"../knowledge/new", "content":task("/new")}),
        json!({"op":"put-artifact", "path":"../knowledge/data.csv", "content":"changed"}),
    ] {
        command(
            &dir,
            "apply",
            json!([{"op":"create","path":"new","content":"---\ntype: Task\n---\n"}, operation]),
        )
        .assert()
        .code(2);
        assert!(!dir.path().join("planning/new.md").exists());
        assert_eq!(
            fs::read(dir.path().join("knowledge/target.md")).unwrap(),
            original
        );
        assert_eq!(
            fs::read_dir(dir.path().join("knowledge")).unwrap().count(),
            1
        );
    }
}

#[test]
fn declared_metadata_reference_is_checked_without_a_matching_body_link() {
    let dir = fixture();
    for (path, reference, succeeds) in [
        ("valid", "../knowledge/target.md", true),
        ("invalid", "../knowledge/missing.md", false),
    ] {
        let content = format!("---\ntype: Task\nconcepts: ['{reference}']\n---\n# Task\n");
        let assertion = command(
            &dir,
            "apply",
            json!([{"op":"create","path":path,"content":content}]),
        )
        .assert();
        if succeeds {
            assertion.success();
        } else {
            assertion
                .code(2)
                .stderr(predicate::str::contains("dangling/blocked"));
        }
        assert_eq!(
            dir.path().join(format!("planning/{path}.md")).exists(),
            succeeds
        );
    }
}
