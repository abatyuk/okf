//! Source health regressions: adding provenance must not silently bypass drift checks.
use std::path::Path;
use std::process::{Command, Output};

use assert_cmd::prelude::*;
use serde_json::Value;

fn run(root: &Path, args: &[&str]) -> Output {
    Command::cargo_bin("okf")
        .unwrap()
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn successful(root: &Path, args: &[&str]) -> Output {
    let output = run(root, args);
    assert!(
        output.status.success(),
        "{args:?}: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn bundle() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    successful(root.path(), &["init", "."]);
    std::fs::create_dir_all(root.path().join("contracts/x/references")).unwrap();
    std::fs::write(
        root.path().join("contracts/x/references/s.json"),
        "{\"a\":1}\n",
    )
    .unwrap();
    root
}

fn add(root: &Path, source: &str) {
    successful(
        root,
        &[
            "add",
            "contracts/x/c",
            "--type",
            "Concept",
            "--title",
            "T",
            "--description",
            "D",
            "--add-source-json",
            source,
        ],
    );
}

fn records(output: &Output) -> Vec<Value> {
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn unrecorded_source_is_reported_until_refresh_and_then_detects_changes() {
    let root = bundle();
    add(
        root.path(),
        r#"{"resource":"contracts/x/references/s.json","kind":"file"}"#,
    );
    successful(root.path(), &["validate"]);
    let stale = run(root.path(), &["stale", "--json", "--fail-on", "any"]);
    assert_eq!(stale.status.code(), Some(1));
    assert_eq!(records(&stale)[0]["sources"][0]["drift"], "unrecorded");

    let lint = run(root.path(), &["lint", "--json", "--fail-on", "warn"]);
    assert_eq!(lint.status.code(), Some(1));
    assert!(records(&lint)
        .iter()
        .any(|record| record["rule"] == "source-unrecorded" && record["severity"] == "warn"));
    assert!(!records(&lint)
        .iter()
        .any(|record| record["rule"] == "source-missing"));

    successful(root.path(), &["refresh", "/contracts/x/c"]);
    assert!(records(&successful(
        root.path(),
        &["stale", "--json", "--fail-on", "any"]
    ))
    .is_empty());
    assert!(!records(&successful(root.path(), &["lint", "--json"]))
        .iter()
        .any(|record| record["rule"] == "source-unrecorded"));

    std::fs::write(
        root.path().join("contracts/x/references/s.json"),
        "{\"a\":2}\n",
    )
    .unwrap();
    let stale = run(root.path(), &["stale", "--json", "--fail-on", "any"]);
    assert_eq!(stale.status.code(), Some(1));
    assert_eq!(records(&stale)[0]["sources"][0]["drift"], "drifted");
}

#[test]
fn missing_unrecorded_source_fails_health_checks_and_refresh_has_one_error_prefix() {
    let root = bundle();
    add(
        root.path(),
        r#"{"resource":"contracts/x/references/nope.json","kind":"file"}"#,
    );
    successful(root.path(), &["validate"]);
    let stale = run(root.path(), &["stale", "--json", "--fail-on", "any"]);
    assert_eq!(stale.status.code(), Some(1));
    assert_eq!(records(&stale)[0]["sources"][0]["drift"], "missing");

    let lint = run(root.path(), &["lint", "--json"]);
    assert_eq!(lint.status.code(), Some(1));
    assert!(records(&lint)
        .iter()
        .any(|record| record["rule"] == "source-missing" && record["severity"] == "error"));

    let refresh = run(
        root.path(),
        &["refresh", "/contracts/x/c", "--fail-on", "skipped"],
    );
    assert_eq!(refresh.status.code(), Some(1));
    let message = format!(
        "{}{}",
        String::from_utf8_lossy(&refresh.stdout),
        String::from_utf8_lossy(&refresh.stderr)
    );
    assert!(message.contains("nope.json"));
    assert_eq!(message.matches("I/O error:").count(), 1, "{message}");
}

#[test]
fn source_added_by_edit_is_also_unrecorded() {
    let root = bundle();
    add(
        root.path(),
        r#"{"resource":"https://example.invalid/provenance"}"#,
    );
    successful(root.path(), &["stale", "--fail-on", "any"]);
    successful(
        root.path(),
        &[
            "edit",
            "/contracts/x/c",
            "--add-source-json",
            r#"{"resource":"contracts/x/references/s.json","kind":"file"}"#,
        ],
    );
    let stale = run(root.path(), &["stale", "--json", "--fail-on", "any"]);
    assert_eq!(stale.status.code(), Some(1));
    let sources = records(&stale)[0]["sources"].as_array().unwrap().clone();
    assert_eq!(sources.len(), 1, "kind-less provenance remains valid");
    assert_eq!(sources[0]["drift"], "unrecorded");
}

#[test]
fn deleted_recorded_source_is_also_a_lint_error() {
    let root = bundle();
    add(
        root.path(),
        r#"{"resource":"contracts/x/references/s.json","kind":"file"}"#,
    );
    successful(root.path(), &["refresh", "/contracts/x/c"]);
    std::fs::remove_file(root.path().join("contracts/x/references/s.json")).unwrap();
    let lint = run(root.path(), &["lint", "--json"]);
    assert_eq!(lint.status.code(), Some(1));
    assert!(records(&lint)
        .iter()
        .any(|record| record["rule"] == "source-missing"));
    assert!(!records(&lint)
        .iter()
        .any(|record| record["rule"] == "source-unrecorded"));
}
