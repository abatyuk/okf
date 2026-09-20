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
fn list_is_exactly_unfiltered_search_in_json_mode() {
    let bundle = fixture("sample-bundle");
    let list = okf()
        .args(["list", bundle.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let search = okf()
        .args(["search", bundle.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(list.status.success());
    assert!(search.status.success());
    assert_eq!(list.stdout, search.stdout);
}

#[test]
fn search_and_init_honor_environment_and_toml_bundle_resolution() {
    let project = tempfile::tempdir().unwrap();
    let configured = project.path().join("configured");
    std::fs::create_dir(&configured).unwrap();
    std::fs::write(
        configured.join("configured.md"),
        "---\ntype: Note\ntitle: Configured\n---\n",
    )
    .unwrap();
    std::fs::write(project.path().join("okf.toml"), "bundle = \"configured\"\n").unwrap();

    let config_search = okf()
        .args(["search", "--text", "Configured", "--json"])
        .current_dir(project.path())
        .env_remove("OKF_BUNDLE")
        .output()
        .unwrap();
    assert!(config_search.status.success());
    assert_eq!(ndjson(&config_search.stdout)[0]["id"], "/configured");

    let config_graph = okf()
        .args(["graph", "--root", "configured", "--json"])
        .current_dir(project.path())
        .env_remove("OKF_BUNDLE")
        .output()
        .unwrap();
    assert!(config_graph.status.success());
    assert_eq!(ndjson(&config_graph.stdout)[0]["root"], "configured");

    let env_bundle = project.path().join("environment");
    std::fs::create_dir(&env_bundle).unwrap();
    std::fs::write(
        env_bundle.join("environment.md"),
        "---\ntype: Note\ntitle: Environment\n---\n",
    )
    .unwrap();
    let env_search = okf()
        .args(["search", "--text", "Environment", "--json"])
        .current_dir(project.path())
        .env("OKF_BUNDLE", &env_bundle)
        .output()
        .unwrap();
    assert!(env_search.status.success());
    assert_eq!(ndjson(&env_search.stdout)[0]["id"], "/environment");

    std::fs::write(project.path().join("okf.toml"), "bundle = \"new/bundle\"\n").unwrap();
    let initialized = okf()
        .arg("init")
        .current_dir(project.path())
        .env_remove("OKF_BUNDLE")
        .output()
        .unwrap();
    assert!(initialized.status.success());
    assert!(project.path().join("new/bundle/index.md").is_file());
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
fn skill_reading_workflow_distinguishes_metadata_from_body_and_preserves_curated_index() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().to_str().unwrap();
    let index = "---\nokf_version: \"0.2\"\n---\n# Curated overview\n\nAuthored context.\n";
    std::fs::write(dir.path().join("index.md"), index).unwrap();
    std::fs::write(
        dir.path().join("policy.md"),
        "---\ntype: Note\ntitle: Policy\n---\n# Policy\n\nA material claim to review.\n",
    )
    .unwrap();

    let metadata = okf()
        .args(["show", "policy", bundle, "--json"])
        .output()
        .unwrap();
    assert!(metadata.status.success());
    let records = ndjson(&metadata.stdout);
    assert_eq!(records[0]["title"], "Policy");
    assert!(!String::from_utf8_lossy(&metadata.stdout).contains("A material claim"));

    let body = okf().args(["show", "policy", bundle]).output().unwrap();
    assert!(body.status.success());
    assert!(String::from_utf8_lossy(&body.stdout).contains("A material claim"));
    let slice = okf()
        .args(["show", "policy", bundle, "--lines", "5:7", "--json"])
        .output()
        .unwrap();
    assert!(slice.status.success());
    let records = ndjson(&slice.stdout);
    assert_eq!(
        records[0]["lines"][2]["text"],
        "A material claim to review."
    );

    // Read-only navigation preserves curated text; index generation explicitly replaces it.
    okf().args(["browse", bundle]).assert().success();
    assert_eq!(
        std::fs::read_to_string(dir.path().join("index.md")).unwrap(),
        index
    );
    okf()
        .args(["docs", bundle, "--format", "index"])
        .assert()
        .success();
    let generated = std::fs::read_to_string(dir.path().join("index.md")).unwrap();
    assert!(!generated.contains("Authored context."));
    assert!(generated.contains("policy.md"));
    assert!(generated.contains("okf_version"));
    okf().args(["validate", bundle]).assert().success();
}

#[test]
fn skill_search_workflow_narrows_fields_and_handles_no_matches() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().to_str().unwrap();
    std::fs::write(
        dir.path().join("policy.md"),
        "---\ntype: Note\ntitle: Policy\ncustom: secret-token\n---\n# Policy\n\nTravel reimbursement.\n",
    )
    .unwrap();
    let query = |term: &str, fields: Option<&str>| {
        let mut cmd = okf();
        cmd.args(["search", bundle, "--text", term, "--json"]);
        if let Some(fields) = fields {
            cmd.args(["--in", fields]);
        }
        let out = cmd.output().unwrap();
        assert!(out.status.success());
        ndjson(&out.stdout)
    };
    assert_eq!(query("reimbursement", None).len(), 1);
    assert!(query("reimbursement", Some("title,description")).is_empty());
    assert!(query("secret-token", None).is_empty());
    assert_eq!(query("secret-token", Some("frontmatter")).len(), 1);
}

#[test]
fn show_loads_only_the_requested_concept() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("wanted.md"),
        "---\ntype: Note\ntitle: Wanted\n---\nbody\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("broken.md"),
        "---\ntype: [unterminated\n---\n",
    )
    .unwrap();

    okf()
        .args(["show", "wanted", root.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("Wanted"));
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
fn search_text_matches_markdown_wrapping_but_not_paragraph_boundaries() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("wrapped.md"),
        "---\ntype: Note\ntitle: Wrapped prose\n---\nalpha\nbeta\n\ngamma\ndelta\n",
    )
    .unwrap();

    let search = |phrase: &str| {
        let out = okf()
            .args([
                "search",
                root.path().to_str().unwrap(),
                "--text",
                phrase,
                "--json",
            ])
            .output()
            .unwrap();
        assert!(out.status.success());
        ndjson(&out.stdout)
    };

    assert_eq!(search("alpha beta")[0]["id"], "/wrapped");
    assert_eq!(search("alpha   beta")[0]["id"], "/wrapped");
    assert!(search("beta gamma").is_empty());
}

#[test]
fn search_supports_modes_scopes_evidence_ranking_and_limits() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("a-body.md"),
        "---\ntype: Note\ntitle: Body result\nowner: Jane Doe\nsearch: authored\n---\n# Intro\nNeedle **search** includes [gamma](https://example.test).\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("z-title.md"),
        "---\ntype: Note\ntitle: Needle search\n---\nUnrelated body.\n",
    )
    .unwrap();

    let run = |args: &[&str]| {
        let mut command = okf();
        command
            .arg("search")
            .arg(root.path())
            .args(args)
            .arg("--json");
        let out = command.output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        ndjson(&out.stdout)
    };

    // Reader-visible Markdown and repeated phrases (AND).
    let markdown = run(&["--text", "needle search", "--text", "includes gamma"]);
    assert_eq!(markdown.len(), 1);
    assert_eq!(markdown[0]["id"], "/a-body");
    assert_eq!(markdown[0]["search"]["matches"][0]["field"], "body");
    assert_eq!(markdown[0]["search"]["matches"][0]["line"], 8);
    assert!(markdown[0]["search"]["matches"][0]["snippet"]
        .as_str()
        .unwrap()
        .contains("Needle search"));
    assert_eq!(markdown[0]["frontmatter_conflicts"]["search"], "authored");
    let shown = okf()
        .args([
            "show",
            "a-body",
            root.path().to_str().unwrap(),
            "--lines",
            "8",
        ])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(shown.stdout).unwrap(),
        "8: Needle **search** includes [gamma](https://example.test).\n"
    );

    assert!(run(&["--text", "needle search", "--match", "literal"])
        .iter()
        .all(|record| record["id"] != "/a-body"));
    assert_eq!(
        run(&["--text", "needle **search**", "--match", "literal"])[0]["id"],
        "/a-body"
    );
    assert_eq!(
        run(&["--text", "needle missing", "--match", "any", "--in", "body",])[0]["id"],
        "/a-body"
    );
    assert_eq!(
        run(&["--text", "needle gamma", "--match", "all"])[0]["id"],
        "/a-body"
    );

    // Scope can include arbitrary frontmatter values.
    assert_eq!(
        run(&["--text", "Jane Doe", "--in", "frontmatter"])[0]["search"]["matches"][0]["field"],
        "frontmatter.owner"
    );
    assert_eq!(
        run(&["--text", "needle search", "--in", "title"])[0]["id"],
        "/z-title"
    );

    // Relevance puts the exact title first; id sorting and limits are explicit.
    let relevant = run(&["--text", "needle search"]);
    assert_eq!(relevant[0]["id"], "/z-title");
    assert_eq!(relevant[1]["id"], "/a-body");
    let by_id = run(&["--text", "needle search", "--sort", "id", "--limit", "1"]);
    assert_eq!(by_id.len(), 1);
    assert_eq!(by_id[0]["id"], "/a-body");
}

#[test]
fn validate_conformant_exits_zero() {
    okf()
        .args(["validate", fixture("sample-bundle").to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn scan_supports_the_documented_fail_on_contract() {
    okf()
        .args([
            "scan",
            fixture("sample-bundle").to_str().unwrap(),
            "--fail-on",
            "never",
        ])
        .assert()
        .success();
    okf()
        .args([
            "scan",
            fixture("sample-bundle").to_str().unwrap(),
            "--fail-on",
            "any",
        ])
        .assert()
        .code(1);
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
fn graph_and_docs_honor_json_output() {
    let graph = okf()
        .args([
            "graph",
            fixture("linked-bundle").to_str().unwrap(),
            "--root",
            "policies/travel",
            "--format",
            "mermaid",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(graph.status.success());
    let graph = ndjson(&graph.stdout);
    assert_eq!(graph[0]["kind"], "graph");
    assert_eq!(graph[0]["root"], "policies/travel");
    assert!(graph[0]["content"]
        .as_str()
        .unwrap()
        .starts_with("graph LR\n"));

    let docs = okf()
        .args([
            "docs",
            fixture("sample-bundle").to_str().unwrap(),
            "--format",
            "md",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(docs.status.success());
    let docs = ndjson(&docs.stdout);
    assert_eq!(docs[0]["kind"], "docs");
    assert_eq!(docs[0]["format"], "md");
    assert!(!docs[0]["content"].as_str().unwrap().is_empty());
}

#[test]
fn finite_choice_flags_are_validated_by_clap() {
    for args in [
        vec!["graph", "--format", "unknown"],
        vec!["docs", "--format", "unknown"],
        vec!["scan", "--fail-on", "unknown"],
        vec!["refresh", "missing", "--fail-on", "unknown"],
    ] {
        okf().args(args).assert().code(2);
    }
}

#[test]
fn links_lists_direct_normalized_targets_and_missing_state() {
    let out = okf()
        .args([
            "links",
            "policies/travel",
            fixture("linked-bundle").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let records = ndjson(&out.stdout);
    assert_eq!(records.len(), 3);
    assert!(records.iter().all(|r| r["kind"] == "link"));
    assert!(records.iter().all(|r| r["source"] == "/policies/travel"));
    assert_eq!(records[0]["target"], "/computations/mileage");
    assert_eq!(records[1]["target"], "/tables/customers");
    assert_eq!(records[2]["target"], "/tables/ghost");
    assert_eq!(records[2]["exists"], false);
}

#[test]
fn graph_can_bound_outgoing_neighborhood_depth() {
    let out = okf()
        .args([
            "graph",
            fixture("linked-bundle").to_str().unwrap(),
            "--root",
            "policies/travel",
            "--direction",
            "outgoing",
            "--depth",
            "1",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("/policies/travel"), "{text}");
    assert!(text.contains("/tables/customers"), "{text}");
    assert!(text.contains("/tables/ghost"), "{text}");
    assert!(!text.contains("/metrics/revenue"), "{text}");
}

#[test]
fn graph_can_walk_incoming_neighborhood() {
    let out = okf()
        .args([
            "graph",
            fixture("linked-bundle").to_str().unwrap(),
            "--root",
            "metrics/revenue",
            "--direction",
            "incoming",
            "--depth",
            "1",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("/metrics/revenue"), "{text}");
    assert!(text.contains("/tables/customers"), "{text}");
    assert!(!text.contains("/policies/travel"), "{text}");
}

#[test]
fn schema_is_valid_ndjson_with_all_commands() {
    let out = okf().arg("schema").output().unwrap();
    assert!(out.status.success());
    let records = ndjson(&out.stdout);
    assert_eq!(records[0]["kind"], "schema");
    assert_eq!(records[0]["tool"], "okf");
    assert_eq!(records[0]["ndjson_schema"], "2");

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
        "links",
        "graph",
        "resolve",
        "artifact list",
        "artifact resolve",
        "artifact show",
        "computation check",
        "scan",
        "validate",
        "lint",
        "stale",
        "affected",
        "diff",
        "stats",
        "doctor",
        "source-scan",
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
        .find(|a| a["name"] == "set-section")
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

    let search = records
        .iter()
        .find(|record| record["name"] == "search")
        .expect("search command");
    let search_arg = |name: &str| {
        search["args"]
            .as_array()
            .unwrap()
            .iter()
            .find(|argument| argument["name"] == name)
            .unwrap()
    };
    assert_eq!(search_arg("limit")["type"], "int");
    assert_eq!(
        search_arg("bundle")["resolution"],
        serde_json::json!(["explicit", "env:OKF_BUNDLE", "config:okf.toml", "cwd"])
    );
    assert!(search_arg("bundle")["default"].is_null());
    assert_eq!(
        search_arg("match")["possible_values"],
        serde_json::json!(["phrase", "all", "any", "literal"])
    );
    assert_eq!(search_arg("text")["default"], serde_json::json!([]));
    assert_eq!(
        search_arg("in")["default"],
        serde_json::json!(["id", "title", "description", "body"])
    );

    let graph = records
        .iter()
        .find(|record| record["name"] == "graph")
        .expect("graph command");
    assert!(graph["args"]
        .as_array()
        .unwrap()
        .iter()
        .any(|argument| argument["name"] == "root"));
    let docs = records
        .iter()
        .find(|record| record["name"] == "docs")
        .expect("docs command");
    assert_eq!(docs["mutates"], true);
    assert_eq!(docs["output"]["stream"], "docs,change");
    assert_eq!(docs["output"]["stream_when"]["format=index"], "change");

    let lint = records
        .iter()
        .find(|record| record["name"] == "lint")
        .expect("lint command");
    let lint_args = lint["args"].as_array().unwrap();
    assert!(lint_args
        .iter()
        .any(|argument| argument["name"] == "fail-on" && argument["default"] == "error"));
    assert!(!lint_args
        .iter()
        .any(|argument| argument["name"] == "fail_on"));
}

#[test]
fn artifact_commands_resolve_and_show_reference_file() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("references")).unwrap();
    std::fs::write(root.path().join("references/query.sql"), "select 1;\n").unwrap();

    let out = okf()
        .args([
            "artifact",
            "resolve",
            "references/query.sql",
            root.path().to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let records = ndjson(&out.stdout);
    assert_eq!(records[0]["artifact_kind"], "artifact");

    okf()
        .args([
            "artifact",
            "show",
            "references/query.sql",
            root.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout("select 1;\n");
}

#[test]
fn doctor_reports_nonconformance_and_always_emits_summary() {
    let out = okf()
        .args([
            "doctor",
            fixture("nonconformant-bundle").to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let records = ndjson(&out.stdout);
    assert!(records.iter().any(|r| r["kind"] == "doctor-finding"));
    assert_eq!(records.last().unwrap()["kind"], "doctor-summary");
    assert_eq!(records.last().unwrap()["ready"], false);
}

#[test]
fn doctor_safe_fix_repairs_empty_index_only_when_confirmed() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("index.md"), "").unwrap();
    let dry = okf()
        .args([
            "doctor",
            root.path().to_str().unwrap(),
            "--fix-safe",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(dry.status.code(), Some(1));
    assert_eq!(
        std::fs::read_to_string(root.path().join("index.md")).unwrap(),
        ""
    );

    okf()
        .args([
            "doctor",
            root.path().to_str().unwrap(),
            "--fix-safe",
            "--yes",
        ])
        .assert()
        .success();
    assert!(std::fs::read_to_string(root.path().join("index.md"))
        .unwrap()
        .starts_with("# Concepts"));
}

#[test]
fn attested_add_requires_runtime_and_uses_exact_type() {
    let root = tempfile::tempdir().unwrap();
    let missing = okf()
        .args([
            "add",
            "computations/x",
            root.path().to_str().unwrap(),
            "--attested",
        ])
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(2));

    okf()
        .args([
            "add",
            "computations/x",
            root.path().to_str().unwrap(),
            "--attested",
            "--runtime",
            "python",
            "--inline-computation",
            "print(1)",
        ])
        .assert()
        .success();
    let text = std::fs::read_to_string(root.path().join("computations/x.md")).unwrap();
    assert!(text.contains("type: Attested Computation"));
    assert!(text.contains("runtime: python"));
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
            "--add-source-json",
            r#"{"resource":"https://example.com/spec","id":"spec","title":"Specification"}"#,
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
    assert_eq!(record["sources"][1]["id"], "spec");
    assert!(record["sources"][1].get("kind").is_none());

    okf()
        .args(["refresh", "notes/x", root, "--fail-on", "any"])
        .assert()
        .code(1)
        .stderr(predicates::str::contains("skipped missing.txt"));
}

#[test]
fn structured_sources_can_be_removed_by_resource_or_kind() {
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
            "--add-source",
            "resource=shared,kind=file",
            "--add-source",
            "resource=shared,kind=url",
            "--add-source",
            "resource=other,kind=file",
        ])
        .assert()
        .success();

    let edited = okf()
        .args([
            "edit",
            "notes/x",
            root,
            "--remove-source",
            "resource=shared,kind=file",
            "--remove-source",
            "other",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(edited.status.success());
    let changes = &ndjson(&edited.stdout)[0]["changes"];
    assert_eq!(changes[0]["removed"], 1);
    assert_eq!(changes[1]["removed"], 1);

    let shown = okf()
        .args(["show", "notes/x", root, "--json"])
        .output()
        .unwrap();
    let sources = ndjson(&shown.stdout)[0]["sources"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0]["resource"], "shared");
    assert_eq!(sources[0]["kind"], "url");
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
