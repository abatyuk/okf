use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn catalog_graph_artifacts_keep_scope_and_reference_diagnostics_on_stderr() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("knowledge")).unwrap();
    std::fs::write(
        root.path().join("okf.toml"),
        "catalog = 'okf-catalog.yaml'\ndefault_bundle = {id = 'acme.notes'}\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("okf-catalog.yaml"),
        "catalog_version: 1\nbundles:\n  acme.notes:\n    location: {type: directory, path: knowledge}\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("knowledge/note.md"),
        "---\ntype: Note\n---\n[Missing](/missing.md)\n",
    )
    .unwrap();
    for (format, prefix, suffix) in [
        ("graphml", "<?xml version=", "</graph></graphml>"),
        ("dot", "digraph okf {", "}"),
        ("mermaid", "graph TD", "n1"),
    ] {
        let result = Command::cargo_bin("okf")
            .unwrap()
            .current_dir(root.path())
            .env_remove("OKF_BUNDLE")
            .env_remove("OKF_CATALOG_OVERRIDES")
            .args(["graph", "--format", format])
            .output()
            .unwrap();
        assert!(result.status.success(), "{:?}", result);
        let stdout = String::from_utf8(result.stdout).unwrap();
        let stderr = String::from_utf8(result.stderr).unwrap();
        assert!(stdout.starts_with(prefix), "{stdout}");
        if format != "mermaid" {
            assert!(stdout.trim_end().ends_with(suffix), "{stdout}");
        }
        assert!(!stdout.contains("scope:"), "{stdout}");
        assert!(!stdout.contains("ordinary-path"), "{stdout}");
        assert!(stderr.contains("scope:"), "{stderr}");
        assert!(stderr.contains("missing-target"), "{stderr}");
    }
    let json = Command::cargo_bin("okf")
        .unwrap()
        .current_dir(root.path())
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args(["graph", "--json", "--format", "graphml"])
        .output()
        .unwrap();
    assert!(json.status.success(), "{:?}", json);
    let records = String::from_utf8(json.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert!(records.iter().any(|record| record["kind"] == "scope"));
    assert!(records
        .iter()
        .any(|record| { record["kind"] == "bundle-edge" && record["status"] == "missing-target" }));
}
