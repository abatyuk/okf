//! Structured ontology declarations, coordinated changes, and safe previews.
use assert_cmd::prelude::*;
use std::process::{Command, Output};

fn run(root: &std::path::Path, args: &[&str]) -> Output {
    let mut cmd = Command::cargo_bin("okf").unwrap();
    cmd.current_dir(root)
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args(args);
    cmd.output().unwrap()
}
fn success(root: &std::path::Path, args: &[&str]) -> Output {
    let out = run(root, args);
    assert!(
        out.status.success(),
        "{:?}: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    out
}
fn ontology(root: &std::path::Path) -> serde_yaml::Value {
    serde_yaml::from_str(&std::fs::read_to_string(root.join("ontology.yaml")).unwrap()).unwrap()
}

#[test]
fn ontology_inspection_exposes_selectors_relationships_and_authored_properties() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    success(root, &["ontology", "apply", "--from", "{field_types: {Norm: {base: object, fields: {bearer: {type: uri}}}}, concepts: {Base: {}, Procedure: {extends: Base, trust: {min_tier: verified, local: retained}, local: authored, fields: {norms: {type: list, item: Norm, min: 1}}, references: {bearers: {selector: 'norms[].bearer', target: Team, cardinality: '1..n', local: retained}}, relationships: {obligations: {reference: bearers, kind: obliges, inverse: obligated-by}}}}}"]);
    let out = success(root, &["ontology", "show", "Procedure"]);
    let human = String::from_utf8(out.stdout).unwrap();
    for expected in [
        "selector: norms[].bearer",
        "relationships:",
        "obligations:",
        "inverse: obligated-by",
        "trust:",
        "min_tier: verified",
        "extends: Base",
        "local: authored",
    ] {
        assert!(human.contains(expected), "missing {expected:?}: {human}");
    }
    let out = success(root, &["ontology", "show", "Procedure", "--json"]);
    let record: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(record["trust"]["min_tier"], "verified");
    assert_eq!(record["extra"]["local"], "authored");
    assert_eq!(record["extends"], "Base");
    assert_eq!(record["fields"][0]["declaration"]["min"], 1);
    assert_eq!(record["references"][0]["declaration"]["local"], "retained");
    let declared: serde_yaml::Value =
        serde_yaml::from_str(&serde_json::to_string(&record["declaration"]).unwrap()).unwrap();
    assert_eq!(declared, ontology(root)["concepts"]["Procedure"]);
}

#[test]
fn reusable_field_type_inspection_shows_declarations_and_effective_constraints() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    success(root, &["ontology", "apply", "--from", "{field_types: {Text: {base: string, min: 1, local: retained}, Label: {extends: Text, max: 20}}}"]);
    let out = success(root, &["ontology", "field-type", "list", "--json"]);
    let records: Vec<serde_json::Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(records.len(), 2);
    assert_eq!(records[1]["name"], "Label");
    let out = success(root, &["ontology", "field-type", "show", "Label", "--json"]);
    let record: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(record["definition"]["extends"], "Text");
    assert_eq!(record["effective"]["base"], "string");
    assert_eq!(record["effective"]["constraints"]["min"], 1);
    assert_eq!(record["effective"]["constraints"]["max"], 20);
    assert_eq!(record["effective"]["constraints"]["local"], "retained");
    let human = success(root, &["ontology", "field-type", "show", "Label"]);
    assert!(String::from_utf8(human.stdout)
        .unwrap()
        .contains("extends: Text"));
    let missing = run(root, &["ontology", "field-type", "show", "Missing"]);
    assert_eq!(missing.status.code(), Some(2));
}

#[test]
fn unrelated_ontology_update_preserves_literal_scalar_in_preview_and_write() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let original = "okf_ontology: '0.1'\nconcepts:\n  Service:\n    description: |\n      # authored description\n      status: authored prose\n    fields:\n      status: {type: string}\n";
    std::fs::write(root.join("ontology.yaml"), original).unwrap();
    let before = ontology(root)["concepts"]["Service"]["description"].clone();
    let preview = success(
        root,
        &[
            "ontology",
            "update",
            "Service",
            "--field",
            "owner:string",
            "--dry-run",
            "--json",
        ],
    );
    assert_eq!(
        std::fs::read_to_string(root.join("ontology.yaml")).unwrap(),
        original
    );
    let record: serde_json::Value = serde_json::from_slice(&preview.stdout).unwrap();
    let generated = record["diff"]
        .as_str()
        .unwrap()
        .lines()
        .skip(2)
        .filter_map(|line| line.strip_prefix('+'))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let parsed: serde_yaml::Value = serde_yaml::from_str(&generated).unwrap();
    assert_eq!(parsed["concepts"]["Service"]["description"], before);
    success(
        root,
        &["ontology", "update", "Service", "--field", "owner:string"],
    );
    assert_eq!(ontology(root)["concepts"]["Service"]["description"], before);
    assert_eq!(
        std::fs::read_to_string(root.join("ontology.yaml")).unwrap(),
        generated
    );
}

#[test]
fn doctor_uses_configured_ontology_sidecar_for_warnings_and_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::create_dir(root.join("bundle")).unwrap();
    std::fs::write(
        root.join("okf.toml"),
        "bundle='bundle'\n[bundle_settings.default]\nontology='sidecar.yaml'\n",
    )
    .unwrap();
    std::fs::write(
        root.join("sidecar.yaml"),
        "okf_ontology: '0.1'\nconcepts:\n  Custom: {attested: true}\n",
    )
    .unwrap();
    let out = run(root, &["doctor", "--json"]);
    let records: Vec<serde_json::Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let warning = records
        .iter()
        .find(|record| record["rule"] == "custom-attested-ontology-type")
        .expect("configured ontology warning");
    assert!(warning["path"].as_str().unwrap().ends_with("sidecar.yaml"));
    std::fs::write(root.join("sidecar.yaml"), "broken: [\n").unwrap();
    let out = run(root, &["doctor", "--json"]);
    let records: Vec<serde_json::Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let warning = records
        .iter()
        .find(|record| record["rule"] == "ontology-unreadable")
        .expect("invalid configured ontology warning");
    assert!(warning["path"].as_str().unwrap().ends_with("sidecar.yaml"));
    std::fs::remove_file(root.join("sidecar.yaml")).unwrap();
    let out = run(root, &["doctor", "--json"]);
    let records: Vec<serde_json::Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let warning = records
        .iter()
        .find(|record| record["rule"] == "ontology-unreadable")
        .expect("missing configured ontology warning");
    assert!(warning["path"].as_str().unwrap().ends_with("sidecar.yaml"));
    assert!(warning["message"]
        .as_str()
        .unwrap()
        .contains("sidecar.yaml"));
}

#[test]
fn structured_declarations_author_relationships_and_replace_named_entries() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    success(
        root,
        &[
            "ontology",
            "field-type",
            "add",
            "Norm",
            "--from",
            "{base: object, fields: {bearer: {type: uri}, action: {type: string}}}",
        ],
    );
    success(root, &["ontology", "add", "Procedure", "--field-yaml", "norms={type: list, item: Norm, min: 1}", "--ref-yaml", "bearers={selector: 'norms[].bearer', target: Team, cardinality: '1..n'}", "--relationship-yaml", "obligations={reference: bearers, kind: obliges, inverse: obligated-by, attributes: {action: 'norms[].action'}}"]);
    let before = ontology(root);
    assert_eq!(
        before["concepts"]["Procedure"]["relationships"]["obligations"]["inverse"],
        "obligated-by"
    );
    assert_eq!(before["concepts"]["Procedure"]["fields"]["norms"]["min"], 1);
    success(
        root,
        &[
            "ontology",
            "update",
            "Procedure",
            "--field-yaml",
            "norms={type: list, item: Norm}",
        ],
    );
    let after = ontology(root);
    assert!(after["concepts"]["Procedure"]["fields"]["norms"]["min"].is_null());
    assert_eq!(
        after["concepts"]["Procedure"]["relationships"],
        before["concepts"]["Procedure"]["relationships"]
    );
    success(
        root,
        &[
            "ontology",
            "update",
            "Procedure",
            "--remove-relationship",
            "obligations",
            "--remove-ref",
            "bearers",
        ],
    );
    assert!(ontology(root)["concepts"]["Procedure"]["relationships"].is_null());
}

#[test]
fn definition_file_merge_preserves_omitted_properties_and_extensions() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    success(root, &["ontology", "add", "Service", "--from", "{description: First, fields: {details: {type: object, fields: {priority: {type: int}}}, owner: {type: string}}, local: {retained: yes}}"]);
    std::fs::write(root.join("update.yaml"), "description: Second\nfields:\n  details:\n    type: object\n    fields:\n      severity: {type: string}\n").unwrap();
    success(
        root,
        &[
            "ontology",
            "update",
            "Service",
            "--from",
            "update.yaml",
            "--field",
            "status:string",
        ],
    );
    let data = ontology(root);
    let ct = &data["concepts"]["Service"];
    assert_eq!(ct["description"], "Second");
    assert_eq!(ct["fields"]["owner"]["type"], "string");
    assert_eq!(ct["fields"]["status"]["type"], "string");
    assert!(ct["fields"]["details"]["fields"]["priority"].is_null());
    assert_eq!(ct["local"]["retained"], "yes");
    let bytes = std::fs::read(root.join("ontology.yaml")).unwrap();
    let out = run(
        root,
        &[
            "ontology",
            "update",
            "Service",
            "--from",
            "update.yaml",
            "--field",
            "details:string",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(bytes, std::fs::read(root.join("ontology.yaml")).unwrap());
}

#[test]
fn coordinated_apply_validates_final_state_and_supports_explicit_deletion() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    success(root, &["ontology", "apply", "--from", "{field_types: {Deadline: {base: object, fields: {within: {type: int, min: 1}}}}, concepts: {Procedure: {fields: {deadline: {type: Deadline}}}, Retired: {}}}"]);
    let bytes = std::fs::read(root.join("ontology.yaml")).unwrap();
    let out = run(root, &["ontology", "field-type", "remove", "Deadline"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(bytes, std::fs::read(root.join("ontology.yaml")).unwrap());
    let conflict = run(root, &["ontology", "apply", "--from", "{remove: {field_types: [Deadline], concepts: [Retired]}, concepts: {Procedure: {remove: {fields: [deadline]}, fields: {deadline: {type: int}}}}}"]);
    assert_eq!(conflict.status.code(), Some(2));
    assert_eq!(bytes, std::fs::read(root.join("ontology.yaml")).unwrap());
    success(root, &["ontology", "apply", "--from", "{remove: {field_types: [Deadline], concepts: [Retired]}, concepts: {Procedure: {fields: {deadline: {type: int}}}}}"]);
    let data = ontology(root);
    assert!(data["field_types"].is_null());
    assert!(data["concepts"]["Retired"].is_null());
    assert_eq!(
        data["concepts"]["Procedure"]["fields"]["deadline"]["type"],
        "int"
    );
    success(
        root,
        &[
            "ontology",
            "apply",
            "--from",
            "{concepts: {Procedure: {remove: {fields: [deadline]}}}}",
        ],
    );
    assert!(ontology(root)["concepts"]["Procedure"]["fields"].is_null());
}

#[test]
fn previews_preserve_comments_and_match_real_write_without_side_effects() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let original = "# rationale\nokf_ontology: '0.1'\nconcepts:\n  Service:\n    description: Old # retained\n";
    std::fs::write(root.join("ontology.yaml"), original).unwrap();
    let out = success(
        root,
        &[
            "ontology",
            "update",
            "Service",
            "--description",
            "New",
            "--dry-run",
            "--json",
        ],
    );
    assert_eq!(
        std::fs::read_to_string(root.join("ontology.yaml")).unwrap(),
        original
    );
    let record: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(record["dry_run"], true);
    let diff = record["diff"].as_str().unwrap();
    assert!(diff.contains("+# rationale"), "{diff}");
    let generated = diff
        .lines()
        .skip(2)
        .filter_map(|line| line.strip_prefix('+'))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    success(
        root,
        &["ontology", "update", "Service", "--description", "New"],
    );
    assert_eq!(
        std::fs::read_to_string(root.join("ontology.yaml")).unwrap(),
        generated
    );
    let fresh = tempfile::tempdir().unwrap();
    success(fresh.path(), &["ontology", "add", "Service", "--dry-run"]);
    assert!(!fresh.path().join("ontology.yaml").exists());
}

#[test]
fn invalid_structured_inputs_fail_before_writes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for args in [
        vec![
            "ontology",
            "add",
            "Service",
            "--field-yaml",
            "details={required: true}",
        ],
        vec![
            "ontology",
            "add",
            "Service",
            "--field-yaml",
            "details={type: string, type: int}",
        ],
        vec![
            "ontology",
            "add",
            "Service",
            "--ref-yaml",
            "owner={target: Team, cardinality: nonsense}",
        ],
        vec![
            "ontology",
            "add",
            "Service",
            "--relationship-yaml",
            "owner={reference: absent, kind: owns}",
        ],
        vec![
            "ontology",
            "add",
            "Service",
            "--field",
            "status:string",
            "--field-yaml",
            "status={type: int}",
        ],
        vec!["ontology", "add", "Service", "--from", "[]"],
        vec![
            "ontology",
            "field-type",
            "add",
            "int",
            "--from",
            "{base: string}",
        ],
        vec![
            "ontology",
            "apply",
            "--from",
            "{concepts: {Wrong: {attested: true}}}",
        ],
        vec![
            "ontology",
            "apply",
            "--from",
            "{concepts: {Service: {}}, remove: {concepts: [Service]}}",
        ],
        vec!["ontology", "apply", "--from", "{conceptz: {Service: {}}}"],
    ] {
        let out = run(root, &args);
        assert_eq!(
            out.status.code(),
            Some(2),
            "{:?}: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!root.join("ontology.yaml").exists());
    }
}

#[test]
fn reusable_field_type_updates_replace_and_inheritance_checks_removal() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    success(
        root,
        &[
            "ontology",
            "field-type",
            "add",
            "Positive",
            "--from",
            "{base: int, min: 1, local: retained}",
        ],
    );
    success(
        root,
        &[
            "ontology",
            "field-type",
            "update",
            "Positive",
            "--from",
            "{base: int, max: 100}",
        ],
    );
    let data = ontology(root);
    assert!(data["field_types"]["Positive"]["min"].is_null());
    assert!(data["field_types"]["Positive"]["local"].is_null());
    success(root, &["ontology", "apply", "--from", "{field_types: {Limited: {extends: Positive, min: 1}}, concepts: {Parent: {}, Child: {extends: Parent}}}"]);
    let bytes = std::fs::read(root.join("ontology.yaml")).unwrap();
    assert_eq!(
        run(root, &["ontology", "field-type", "remove", "Positive"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        run(root, &["ontology", "remove", "Parent"]).status.code(),
        Some(2)
    );
    assert_eq!(bytes, std::fs::read(root.join("ontology.yaml")).unwrap());
    success(
        root,
        &[
            "ontology",
            "apply",
            "--from",
            "{remove: {field_types: [Positive, Limited], concepts: [Parent, Child]}}",
        ],
    );
}

#[test]
fn stdin_and_file_inputs_share_strict_parsing_and_single_consumer_rule() {
    use std::io::Write;
    use std::process::Stdio;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let out = run(
        root,
        &[
            "ontology",
            "add",
            "Service",
            "--from",
            "-",
            "--field-yaml",
            "details=-",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("one structured input"));
    let mut child = Command::cargo_bin("okf")
        .unwrap()
        .current_dir(root)
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args(["ontology", "add", "Service", "--from", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"fields:\n  details: {type: object, fields: {priority: {type: int}}}\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::write(
        root.join("reference.yaml"),
        "target: Component\ncardinality: 0..n\n",
    )
    .unwrap();
    std::fs::write(
        root.join("relationship.yaml"),
        "reference: subjects\nkind: specifies\ninverse: specified-by\n",
    )
    .unwrap();
    success(
        root,
        &[
            "ontology",
            "update",
            "Service",
            "--ref-yaml",
            "subjects=@reference.yaml",
            "--relationship-yaml",
            "specification=@relationship.yaml",
        ],
    );
    let data = ontology(root);
    assert_eq!(
        data["concepts"]["Service"]["relationships"]["specification"]["inverse"],
        "specified-by"
    );
    success(
        root,
        &[
            "ontology",
            "update",
            "Service",
            "--from",
            "---\ndescription: updated\n",
        ],
    );
}
