use assert_cmd::prelude::*;
use serde_json::Value;
use std::process::Command;

#[test]
fn discovery_exposes_nested_commands_and_preview_conditions() {
    let output = Command::cargo_bin("okf")
        .unwrap()
        .arg("schema")
        .output()
        .unwrap();
    assert!(output.status.success());
    let records: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    for name in [
        "add",
        "edit",
        "ontology add",
        "ontology update",
        "ontology remove",
        "ontology apply",
        "ontology field-type add",
        "ontology field-type update",
        "ontology field-type remove",
    ] {
        let command = records
            .iter()
            .find(|r| r["name"] == name)
            .unwrap_or_else(|| panic!("missing command {name}"));
        assert_eq!(command["mutates"], true, "{name}");
        assert_eq!(command["mutates_when"]["dry-run"], false, "{name}");
        assert_eq!(command["output"]["stream"], "change", "{name}");
        assert!(
            command["args"]
                .as_array()
                .unwrap()
                .iter()
                .any(|a| a["name"] == "dry-run"),
            "{name}"
        );
    }
    assert!(!records.iter().any(|r| r["name"] == "ontology field-type"));
}

#[test]
fn catalog_selection_supports_new_ontology_commands_and_previews() {
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("knowledge");
    std::fs::create_dir(&bundle).unwrap();
    std::fs::write(
        root.path().join("okf.toml"),
        "catalog = 'okf-catalog.yaml'\n",
    )
    .unwrap();
    std::fs::write(root.path().join("okf-catalog.yaml"), "catalog_version: 1\nbundles:\n  acme.notes:\n    location: {type: directory, path: knowledge}\n").unwrap();
    let run = |args: &[&str]| {
        let output = Command::cargo_bin("okf")
            .unwrap()
            .current_dir(root.path())
            .env_remove("OKF_BUNDLE")
            .env_remove("OKF_CATALOG_OVERRIDES")
            .args(["--bundle-id", "acme.notes"])
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["ontology", "apply", "--from", "{field_types: {Label: {base: string}}, concepts: {Note: {fields: {label: {type: Label}}}}}", "--dry-run"]);
    assert!(!bundle.join("ontology.yaml").exists());
    run(&["ontology", "apply", "--from", "{field_types: {Label: {base: string}}, concepts: {Note: {fields: {label: {type: Label}}}}}"]);
    let baseline = std::fs::read(bundle.join("ontology.yaml")).unwrap();
    run(&[
        "ontology",
        "field-type",
        "add",
        "Counter",
        "--from",
        "{base: int}",
        "--dry-run",
    ]);
    assert_eq!(
        std::fs::read(bundle.join("ontology.yaml")).unwrap(),
        baseline
    );
    run(&[
        "ontology",
        "field-type",
        "add",
        "Counter",
        "--from",
        "{base: int}",
    ]);
    run(&[
        "ontology",
        "field-type",
        "update",
        "Counter",
        "--from",
        "{base: int, min: 0}",
    ]);
    run(&["ontology", "field-type", "remove", "Counter"]);
}
