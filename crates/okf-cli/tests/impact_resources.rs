//! Resource impact and scoped ontology-severity regressions.
use assert_cmd::prelude::*;
use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};

fn write(root: &Path, path: &str, content: &str) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::cargo_bin("okf")
        .unwrap()
        .current_dir(root)
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args(args)
        .output()
        .unwrap()
}

fn records(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn fixture(root: &Path, prefix: &str) {
    write(root, &format!("{prefix}data.csv"), "name,value\nx,1\n");
    write(root, &format!("{prefix}notes/direct.md"), "---\ntype: Note\nsources:\n- {resource: data.csv, kind: file}\n- {resource: 'data.csv#L1-2', kind: line-range}\n- {resource: ../data.csv}\n---\n");
    write(
        root,
        &format!("{prefix}notes/second.md"),
        "---\ntype: Note\nsources:\n- {resource: direct.md}\n---\n",
    );
    write(
        root,
        &format!("{prefix}notes/third.md"),
        "---\ntype: Note\nsources:\n- {resource: second.md}\n---\n",
    );
    write(
        root,
        &format!("{prefix}notes/unrelated.md"),
        "---\ntype: Note\nsources:\n- {resource: data.csv}\n---\n",
    );
}

#[test]
fn single_bundle_changed_resources_seed_direct_and_transitive_review() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "");
    for (extra, expected) in [
        (vec![], vec!["/notes/direct"]),
        (
            vec!["--transitive", "--depth", "2"],
            vec!["/notes/direct", "/notes/second"],
        ),
        (
            vec!["--transitive"],
            vec!["/notes/direct", "/notes/second", "/notes/third"],
        ),
    ] {
        let mut args = vec!["affected", "--changed", "./data.csv#L2", "--json"];
        args.extend(extra);
        let output = run(root, &args);
        assert!(output.status.success(), "{output:?}");
        let actual: Vec<_> = records(&output)
            .into_iter()
            .filter(|row| row["kind"] == "affected")
            .map(|row| row["concept"].as_str().unwrap().to_owned())
            .collect();
        assert_eq!(actual, expected);
    }
    // Deleted artifacts retain their source lineage and remain useful impact seeds.
    std::fs::remove_file(root.join("data.csv")).unwrap();
    let output = run(root, &["affected", "--changed", "data.csv", "--json"]);
    assert_eq!(records(&output)[0]["concept"], "/notes/direct");
}

#[test]
fn catalog_changed_resources_accept_qualified_seeds_and_preserve_depth() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "a/");
    write(
        root,
        "b/consumer.md",
        "---\ntype: Note\nsources:\n- {resource: '../a/notes/second.md'}\n---\n",
    );
    write(
        root,
        "okf.toml",
        "catalog='catalog.yaml'\ndefault_bundle={id='acme.a'}\n",
    );
    write(root, "catalog.yaml", "catalog_version: 1\nbundles:\n  acme.a: {location: {type: directory, path: a}}\n  acme.b: {location: {type: directory, path: b}}\n");
    for (extra, expected) in [
        (vec![], vec![("acme.a", "/notes/direct")]),
        (
            vec!["--transitive", "--depth", "2"],
            vec![("acme.a", "/notes/direct"), ("acme.a", "/notes/second")],
        ),
        (
            vec!["--transitive"],
            vec![
                ("acme.a", "/notes/direct"),
                ("acme.a", "/notes/second"),
                ("acme.a", "/notes/third"),
                ("acme.b", "/consumer"),
            ],
        ),
    ] {
        let mut args = vec![
            "affected",
            "--changed",
            "acme.a:/data.csv",
            "--catalog-scope",
            "--json",
        ];
        args.extend(extra);
        let output = run(root, &args);
        assert!(output.status.success(), "{output:?}");
        let actual: Vec<_> = records(&output)
            .into_iter()
            .filter(|row| row["kind"] == "bundle-affected")
            .map(|row| {
                (
                    row["bundle"].as_str().unwrap().to_owned(),
                    row["id"].as_str().unwrap().to_owned(),
                )
            })
            .collect();
        assert_eq!(
            actual,
            expected
                .into_iter()
                .map(|(bundle, id)| (bundle.to_owned(), id.to_owned()))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn catalog_local_missing_declared_references_obey_ontology_severity_and_off() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "a/note.md",
        "---\ntype: Note\nrefs: /missing.md\n---\n",
    );
    write(root, "a/ontology.yaml", "okf_ontology: '0.1'\nconcepts:\n  Note:\n    references:\n      refs: {target: Note, cardinality: 0..n}\n");
    write(
        root,
        "catalog.yaml",
        "catalog_version: 1\nbundles:\n  acme.a: {location: {type: directory, path: a}}\n",
    );
    for severity in ["off", "info", "warn", "error"] {
        write(root, "okf.toml", &format!("catalog='catalog.yaml'\ndefault_bundle={{id='acme.a'}}\n[bundle_settings.\"acme.a\".lint]\nmissing_title='off'\nmissing_description='off'\norphan='off'\nindex_coverage='off'\nbroken_link='off'\nontology_violation='{severity}'\n"));
        let output = run(
            root,
            &["lint", "--catalog-scope", "--json", "--fail-on", "warn"],
        );
        assert_eq!(
            output.status.code(),
            Some(if matches!(severity, "warn" | "error") {
                1
            } else {
                0
            }),
            "{severity}: {output:?}"
        );
        let findings: Vec<_> = records(&output)
            .into_iter()
            .filter(|row| row["kind"] == "finding")
            .collect();
        if severity == "off" {
            assert!(findings.is_empty(), "{findings:?}");
        } else {
            assert_eq!(findings.len(), 1, "{findings:?}");
            assert_eq!(findings[0]["rule"], "ontology-violation");
            assert_eq!(findings[0]["code"], "metadata-reference-missing");
            assert_eq!(findings[0]["severity"], severity);
        }
    }
}
