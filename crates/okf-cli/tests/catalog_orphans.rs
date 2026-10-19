use assert_cmd::prelude::*;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn write(root: &Path, path: &str, content: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "okf.toml", "catalog='catalog.yaml'\n");
    write(
        root.path(),
        "catalog.yaml",
        "catalog_version: 1\nbundles:\n  acme.a: {location: {type: directory, path: a}}\n  acme.b: {location: {type: directory, path: b}}\n",
    );
    for bundle in ["a", "b"] {
        write(
            root.path(),
            &format!("{bundle}/ontology.yaml"),
            "okf_ontology: '0.1'\nconcepts:\n  Node:\n    references:\n      refs: {target: [Node, 'acme.b:Node'], cardinality: 0..n}\n",
        );
    }
    root
}

fn node(root: &Path, path: &str, metadata: &str, body: &str) {
    write(
        root,
        path,
        &format!(
            "---\ntype: Node\ntitle: {path}\ndescription: Catalog orphan regression.\n{metadata}---\n{body}\n"
        ),
    );
}

fn lint(root: &Path, fail_on: &str) -> Output {
    Command::cargo_bin("okf")
        .unwrap()
        .current_dir(root)
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args([
            "lint",
            "--bundle-id",
            "acme.a",
            "--catalog-scope",
            "--fail-on",
            fail_on,
            "--json",
        ])
        .output()
        .unwrap()
}

fn orphans(output: &Output) -> BTreeSet<(String, String, String)> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .filter(|row| row["rule"] == "orphan")
        .map(|row| {
            (
                row["bundle"].as_str().unwrap().into(),
                row["concept"].as_str().unwrap().into(),
                row["severity"].as_str().unwrap().into(),
            )
        })
        .collect()
}

#[test]
fn catalog_orphans_include_declared_local_and_cross_bundle_connections() {
    let root = fixture();
    node(
        root.path(),
        "a/local-source.md",
        "refs: [/local-target.md]\n",
        "",
    );
    node(root.path(), "a/local-target.md", "", "");
    node(
        root.path(),
        "a/cross-source.md",
        "refs: [../b/remote.md]\n",
        "",
    );
    node(root.path(), "b/remote.md", "", "");
    node(
        root.path(),
        "a/body-source.md",
        "",
        "[Remote](../b/body-target.md)",
    );
    node(root.path(), "b/body-target.md", "", "");
    // A single-root graph would invent inbound links to these local path collisions.
    node(root.path(), "a/b/remote.md", "", "");
    node(root.path(), "a/b/body-target.md", "", "");
    node(root.path(), "a/isolated.md", "", "");

    let output = lint(root.path(), "never");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        orphans(&output),
        BTreeSet::from([
            ("acme.a".into(), "/b/body-target".into(), "info".into()),
            ("acme.a".into(), "/b/remote".into(), "info".into()),
            ("acme.a".into(), "/isolated".into(), "info".into()),
        ]),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn catalog_orphans_keep_broken_concept_edges_but_exclude_self_and_artifacts() {
    let root = fixture();
    node(
        root.path(),
        "a/missing-local.md",
        "refs: [/missing.md]\n",
        "",
    );
    node(
        root.path(),
        "a/missing-remote.md",
        "refs: [../b/missing.md]\n",
        "",
    );
    node(root.path(), "a/self.md", "refs: [/self.md]\n", "");
    node(root.path(), "a/artifact.md", "", "[Artifact](data.json)");
    write(root.path(), "a/data.json", "{}");
    node(
        root.path(),
        "a/external.md",
        "",
        "[External](https://example.invalid)",
    );

    let output = lint(root.path(), "never");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        orphans(&output),
        BTreeSet::from([
            ("acme.a".into(), "/artifact".into(), "info".into()),
            ("acme.a".into(), "/external".into(), "info".into()),
            ("acme.a".into(), "/self".into(), "info".into()),
        ]),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn catalog_orphans_honor_each_bundles_severity_and_disabled_setting() {
    let root = fixture();
    node(root.path(), "a/isolated.md", "", "");
    node(root.path(), "b/isolated.md", "", "");
    for (a_severity, b_severity, expected_bundle, expected_severity) in [
        ("error", "off", "acme.a", "error"),
        ("off", "warn", "acme.b", "warn"),
    ] {
        write(
            root.path(),
            "okf.toml",
            &format!(
                "catalog='catalog.yaml'\n[bundle_settings.\"acme.a\".lint]\norphan='{a_severity}'\nindex_coverage='off'\n[bundle_settings.\"acme.b\".lint]\norphan='{b_severity}'\nindex_coverage='off'\n"
            ),
        );
        let output = lint(root.path(), "warn");
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert_eq!(
            orphans(&output),
            BTreeSet::from([(
                expected_bundle.into(),
                "/isolated".into(),
                expected_severity.into(),
            )]),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(lint(root.path(), "never").status.success());
    }
}
