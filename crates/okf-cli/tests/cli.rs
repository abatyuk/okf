//! Integration tests for the `okf` binary: output shapes, exit codes, round-trips.
use std::path::PathBuf;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::PredicateBooleanExt;
use serde_json::Value;

/// Path to a shared fixture bundle (they live in the `okf-core` crate).
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../okf-core/tests/fixtures")
        .join(name)
}

fn okf() -> Command {
    Command::cargo_bin("okf").unwrap()
}

/// Parse stdout as NDJSON (one JSON object per line).
fn ndjson(bytes: &[u8]) -> Vec<Value> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<Value>(l).expect("valid JSON line"))
        .collect()
}

#[test]
fn list_json_mirrors_frontmatter_with_id_and_trust() {
    let out = okf()
        .args(["list", fixture("sample-bundle").to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let records = ndjson(&out.stdout);
    assert!(!records.is_empty());
    let revenue = records
        .iter()
        .find(|r| r["id"] == "/metrics/revenue")
        .expect("revenue concept present");
    // Verbatim frontmatter fields + computed id/trust_tier.
    assert_eq!(revenue["type"], "Metric");
    assert_eq!(revenue["title"], "Revenue");
    assert_eq!(revenue["trust_tier"], "machine-confirmed");
}

#[test]
fn show_json_is_single_record() {
    let out = okf()
        .args([
            "show",
            "tables/customers",
            fixture("sample-bundle").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let records = ndjson(&out.stdout);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["id"], "/tables/customers");
    assert_eq!(records[0]["type"], "BigQuery Table");
    assert_eq!(records[0]["trust_tier"], "human-reviewed");
}

#[test]
fn browse_reads_existing_index_and_synthesizes_missing_index() {
    let root = fixture("sample-bundle");
    okf()
        .args(["browse", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("reserved `index.md`"));

    let out = okf()
        .args([
            "browse",
            root.to_str().unwrap(),
            "--directory",
            "tables",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let records = ndjson(&out.stdout);
    assert_eq!(records[0]["kind"], "index");
    assert_eq!(records[0]["directory"], "/tables");
    assert_eq!(records[0]["source"], "synthesized");
    assert!(records[0]["content"]
        .as_str()
        .unwrap()
        .contains("[Customers](customers.md)"));
}

#[test]
fn resolve_reports_structural_index_as_existing() {
    okf()
        .args([
            "resolve",
            "index.md",
            fixture("sample-bundle").to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("index.md\texists"));
}

#[test]
fn show_outline_reports_headings_with_document_line_numbers() {
    let out = okf()
        .args([
            "show",
            "tables/customers",
            fixture("sample-bundle").to_str().unwrap(),
            "--outline",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("13: # Customers"), "{text}");
    assert!(!text.contains("The canonical customers table"), "{text}");
}

#[test]
fn show_lines_returns_only_requested_numbered_slice() {
    let out = okf()
        .args([
            "show",
            "tables/customers",
            fixture("sample-bundle").to_str().unwrap(),
            "--lines",
            "13:14",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "13: # Customers\n14: \n"
    );
}

#[test]
fn show_outline_json_is_structured() {
    let out = okf()
        .args([
            "show",
            "tables/customers",
            fixture("sample-bundle").to_str().unwrap(),
            "--outline",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let records = ndjson(&out.stdout);
    assert_eq!(records[0]["kind"], "outline");
    assert_eq!(records[0]["headings"][0]["line"], 13);
    assert_eq!(records[0]["headings"][0]["text"], "Customers");
}

#[test]
fn search_json_filters_by_type() {
    let out = okf()
        .args([
            "search",
            fixture("sample-bundle").to_str().unwrap(),
            "--type",
            "Metric",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let records = ndjson(&out.stdout);
    assert!(!records.is_empty());
    assert!(records.iter().all(|r| r["type"] == "Metric"));
}

#[test]
fn validate_conformant_exits_zero() {
    okf()
        .args(["validate", fixture("sample-bundle").to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn validate_nonconformant_exits_one() {
    let out = okf()
        .args([
            "validate",
            fixture("nonconformant-bundle").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn validate_json_emits_violation_records() {
    let out = okf()
        .args([
            "validate",
            fixture("nonconformant-bundle").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let records = ndjson(&out.stdout);
    assert!(records.iter().all(|r| r["kind"] == "violation"));
    assert!(records.iter().any(|r| r["rule"] == "missing-type"));
}

#[test]
fn lint_fail_on_never_exits_zero_but_default_may_fail() {
    // linked-bundle has broken links (error severity) → default fail-on error → exit 1.
    let default = okf()
        .args(["lint", fixture("linked-bundle").to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(default.status.code(), Some(1));

    // --fail-on never never fails, whatever the findings.
    okf()
        .args([
            "lint",
            fixture("linked-bundle").to_str().unwrap(),
            "--fail-on",
            "never",
        ])
        .assert()
        .success();
}

#[test]
fn graph_mermaid_renders() {
    let out = okf()
        .args([
            "graph",
            fixture("linked-bundle").to_str().unwrap(),
            "--format",
            "mermaid",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.starts_with("graph LR\n"));
    assert!(text.contains("-->"));
}

#[test]
fn schema_is_valid_ndjson_with_all_commands() {
    let out = okf().arg("schema").output().unwrap();
    assert!(out.status.success());
    let records = ndjson(&out.stdout);
    assert_eq!(records[0]["kind"], "schema");
    assert_eq!(records[0]["tool"], "okf");

    let commands: Vec<&str> = records
        .iter()
        .filter(|r| r["kind"] == "command")
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    for expected in [
        "schema",
        "version",
        "list",
        "search",
        "show",
        "browse",
        "backlinks",
        "graph",
        "resolve",
        "scan",
        "validate",
        "lint",
        "stale",
        "affected",
        "diff",
        "stats",
        "init",
        "add",
        "edit",
        "mv",
        "rm",
        "verify",
        "refresh",
        "docs",
        "ontology list",
        "ontology show",
        "ontology add",
        "ontology update",
        "ontology remove",
    ] {
        assert!(
            commands.contains(&expected),
            "missing command in schema: {expected}"
        );
    }
    // Every command declares a group and mutates bool.
    for r in records.iter().filter(|r| r["kind"] == "command") {
        assert!(r["group"].is_string());
        assert!(r["mutates"].is_boolean());
        assert!(r["output"]["stream"].is_string());
    }

    // Args carry help text (so consumers document them without a `--help` round-trip) and
    // multi-value args advertise their value names.
    let edit = records
        .iter()
        .find(|r| r["name"] == "edit")
        .expect("edit command");
    let set = edit["args"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["name"] == "set")
        .expect("--set arg");
    assert!(set["help"].as_str().is_some_and(|h| !h.is_empty()));
    let set_section = edit["args"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["name"] == "set_section")
        .expect("--set-section arg");
    assert_eq!(set_section["value_names"].as_array().unwrap().len(), 2);

    // Schema names describe the public CLI spelling, not the backing Rust field name.
    let ontology_add = records
        .iter()
        .find(|r| r["name"] == "ontology add")
        .expect("ontology add command");
    let ontology_arg_names: Vec<&str> = ontology_add["args"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a["name"].as_str())
        .collect();
    assert!(ontology_arg_names.contains(&"ref"));
    assert!(!ontology_arg_names.contains(&"reference"));
}

#[test]
fn add_then_show_round_trips_in_temp_bundle() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_str().unwrap();

    okf()
        .args(["init", root, "--no-ontology"])
        .assert()
        .success();
    okf()
        .args([
            "add",
            "policies/travel",
            root,
            "--type",
            "Policy",
            "--title",
            "Travel policy",
        ])
        .assert()
        .success();

    let out = okf()
        .args(["show", "policies/travel", root, "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let records = ndjson(&out.stdout);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["id"], "/policies/travel");
    assert_eq!(records[0]["type"], "Policy");
    assert_eq!(records[0]["title"], "Travel policy");
}

#[test]
fn mv_rewrites_inbound_links_via_cli() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_str().unwrap();

    okf().args(["init", root]).assert().success();
    okf()
        .args(["ontology", "add", "Computation", root])
        .assert()
        .success();
    okf()
        .args([
            "ontology",
            "add",
            "Policy",
            root,
            "--ref",
            "computations:Computation:0..1",
        ])
        .assert()
        .success();
    okf()
        .args([
            "add",
            "computations/mileage",
            root,
            "--type",
            "Computation",
            "--title",
            "M",
        ])
        .assert()
        .success();
    okf()
        .args([
            "add",
            "policies/travel",
            root,
            "--type",
            "Policy",
            "--title",
            "T",
        ])
        .assert()
        .success();
    // Point travel at mileage via a frontmatter link.
    okf()
        .args([
            "edit",
            "policies/travel",
            root,
            "--set",
            "computations=/computations/mileage",
        ])
        .assert()
        .success();

    // Move mileage; the inbound link in travel must be rewritten.
    okf()
        .args([
            "mv",
            "computations/mileage",
            "computations/mileage_v2",
            root,
        ])
        .assert()
        .success();

    let out = okf()
        .args(["backlinks", "computations/mileage_v2", root, "--json"])
        .output()
        .unwrap();
    let records = ndjson(&out.stdout);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["id"], "/policies/travel");
}

#[test]
fn structured_sources_can_be_added_and_refresh_can_fail_on_skips() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_str().unwrap();
    okf()
        .args(["init", root, "--no-ontology"])
        .assert()
        .success();
    okf()
        .args([
            "add",
            "notes/x",
            root,
            "--type",
            "Note",
            "--title",
            "X",
            "--set",
            "status=active",
            "--add-source",
            "resource=missing.txt,kind=file",
        ])
        .assert()
        .success();
    let shown = okf()
        .args(["show", "notes/x", root, "--json"])
        .output()
        .unwrap();
    let record = &ndjson(&shown.stdout)[0];
    assert_eq!(record["status"], "active");
    assert_eq!(record["sources"][0]["resource"], "missing.txt");
    assert_eq!(record["sources"][0]["kind"], "file");

    okf()
        .args(["refresh", "notes/x", root, "--fail-on", "any"])
        .assert()
        .code(1)
        .stderr(predicates::str::contains("skipped missing.txt"));
}

#[test]
fn ontology_show_lists_enum_values_and_update_can_remove_members() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_str().unwrap();
    okf().args(["init", root]).assert().success();
    okf()
        .args([
            "ontology",
            "add",
            "Contract",
            root,
            "--field",
            "status:enum:required:draft|active",
            "--field",
            "schema:string",
            "--ref",
            "depends_on:Contract:0..1",
        ])
        .assert()
        .success();
    okf()
        .args(["ontology", "show", "Contract", root])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "status: enum [draft|active] (required)",
        ));
    okf()
        .args([
            "ontology",
            "update",
            "Contract",
            root,
            "--remove-field",
            "schema",
            "--remove-ref",
            "depends_on",
        ])
        .assert()
        .success();
    okf()
        .args(["ontology", "show", "Contract", root])
        .assert()
        .success()
        .stdout(predicates::str::contains("schema").not())
        .stdout(predicates::str::contains("depends_on").not());
}
