//! Catalog and structured-query contracts must agree about scope and scan completeness.
use serde_json::Value;
use std::fs;
use std::process::{Command, Output};

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for bundle in ["alpha", "beta"] {
        fs::create_dir(root.path().join(bundle)).unwrap();
    }
    fs::write(root.path().join("okf.toml"), "catalog = 'okf-catalog.yaml'\ndefault_bundle = { id = 'acme.alpha' }\n[bundle_settings.\"acme.alpha\".query]\nscan_limit = 2\n[bundle_settings.\"acme.beta\".query]\nscan_limit = 9\n").unwrap();
    fs::write(root.path().join("okf-catalog.yaml"), "catalog_version: 1\nbundles:\n  acme.alpha:\n    location: {type: directory, path: alpha}\n  acme.beta:\n    location: {type: directory, path: beta}\n").unwrap();
    for (bundle, id, tags) in [
        ("alpha", "one", "[security, security]"),
        ("alpha", "two", "[production]"),
        ("beta", "three", "[security]"),
    ] {
        fs::write(
            root.path().join(bundle).join(format!("{id}.md")),
            format!("---\ntype: Note\ntitle: {id}\ntags: {tags}\n---\n"),
        )
        .unwrap();
    }
    root
}
fn run(root: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_okf"))
        .current_dir(root)
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args(args)
        .output()
        .unwrap()
}
fn records(output: &Output) -> Vec<Value> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
#[test]
fn catalog_registration_does_not_expand_default_query_scope() {
    let root = fixture();
    let rows = records(&run(root.path(), &["list", "--json", "--full-scan"]));
    let concepts: Vec<_> = rows
        .iter()
        .filter(|row| row.get("trust_tier").is_some())
        .collect();
    assert_eq!(concepts.len(), 2, "{rows:?}");
    assert!(concepts.iter().all(|row| row["title"] != "three"));
}
#[test]
fn primary_scan_budget_is_query_wide_and_full_scan_removes_it() {
    let root = fixture();
    let partial = records(&run(
        root.path(),
        &[
            "search",
            "--json",
            "--catalog-scope",
            "--offset",
            "0",
            "--limit",
            "10",
        ],
    ));
    let summary = partial
        .iter()
        .find(|row| row["kind"] == "query-summary")
        .expect("query summary");
    assert!(summary["total_matches"].is_null(), "{summary}");
    assert!(summary["next_offset"].is_null(), "{summary}");
    assert!(
        partial.iter().any(|row| row["kind"] == "warning"),
        "{partial:?}"
    );
    let complete = records(&run(
        root.path(),
        &[
            "search",
            "--json",
            "--catalog-scope",
            "--full-scan",
            "--offset",
            "0",
            "--limit",
            "10",
        ],
    ));
    let summary = complete
        .iter()
        .find(|row| row["kind"] == "query-summary")
        .unwrap();
    assert_eq!(summary["total_matches"], 3, "{summary}");
    assert_eq!(summary["returned"], 3, "{summary}");
}
#[test]
fn facets_count_scoped_matches_before_pagination_and_deduplicate_values() {
    let root = fixture();
    let rows = records(&run(
        root.path(),
        &[
            "search",
            "--json",
            "--catalog-scope",
            "--full-scan",
            "--facets",
            "--facet-filter",
            "tags[] in [\"security\"]",
            "--offset",
            "1",
            "--limit",
            "1",
        ],
    ));
    let summary = rows
        .iter()
        .find(|row| row["kind"] == "query-summary")
        .unwrap();
    assert_eq!(summary["total_matches"], 2, "{summary}");
    assert_eq!(summary["returned"], 1, "{summary}");
    let facet = rows
        .iter()
        .find(|row| row["kind"] == "facet" && row["field"] == "tags[]")
        .expect("tags facet");
    let bucket = facet["values"]
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["value"] == "security")
        .unwrap();
    assert_eq!(bucket["count"], 2, "{facet}");
}
#[test]
fn current_directory_selects_registered_bundle_before_configured_default() {
    let root = fixture();
    let rows = records(&run(
        &root.path().join("beta"),
        &["list", "--json", "--full-scan"],
    ));
    let concepts: Vec<_> = rows
        .iter()
        .filter(|row| row.get("trust_tier").is_some())
        .collect();
    assert_eq!(concepts.len(), 1, "{rows:?}");
    assert_eq!(concepts[0]["title"], "three");
}

#[test]
fn scan_budget_does_not_parse_documents_beyond_the_examined_prefix() {
    let root = fixture();
    fs::write(
        root.path().join("beta/zzz-unexamined.md"),
        "---\ntype: [\n---\n",
    )
    .unwrap();
    let rows = records(&run(
        root.path(),
        &[
            "search",
            "--json",
            "--catalog-scope",
            "--scan-limit",
            "2",
            "--offset",
            "0",
            "--limit",
            "10",
        ],
    ));
    let summary = rows
        .iter()
        .find(|row| row["kind"] == "query-summary")
        .unwrap();
    assert!(summary["total_matches"].is_null());
    assert!(rows.iter().any(|row| row["kind"] == "warning"));
}

#[test]
fn schema_describes_global_scope_query_controls_and_extension_records() {
    let root = fixture();
    let rows = records(&run(root.path(), &["schema"]));
    let header = &rows[0];
    for flag in ["bundle-id", "scope-bundle", "catalog-scope", "revision"] {
        assert!(
            header["global_args"]
                .as_array()
                .unwrap()
                .iter()
                .any(|arg| arg["name"] == flag),
            "{flag}"
        );
    }
    for record in [
        "query-summary",
        "projection",
        "facet",
        "relationship",
        "related-concept",
        "expansion-summary",
        "scope",
        "bundle-edge",
        "warning",
    ] {
        assert!(header["output_records"].get(record).is_some(), "{record}");
    }
    for name in ["list", "search"] {
        let command = rows
            .iter()
            .find(|row| row["kind"] == "command" && row["name"] == name)
            .unwrap();
        for flag in [
            "scan-limit",
            "offset",
            "expansion-edges",
            "expansion-targets",
            "expansion-bytes",
        ] {
            let arg = command["args"]
                .as_array()
                .unwrap()
                .iter()
                .find(|arg| arg["name"] == flag)
                .unwrap();
            assert_eq!(arg["type"], "int", "{name} {flag}");
        }
    }
}

#[test]
fn query_controls_reject_conflicting_or_ambiguous_requests() {
    let root = fixture();
    for args in [
        vec!["search", "--full-scan", "--scan-limit", "2"],
        vec!["search", "--facets"],
        vec!["search", "--offset", "1"],
        vec!["list", "alpha", "--bundle-id", "acme.beta"],
        vec!["edit", "one", "--catalog-scope", "--set", "title=Changed"],
    ] {
        let output = run(root.path(), &args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn selected_primary_settings_win_even_when_its_id_sorts_last() {
    let root = fixture();
    let rows = records(&run(
        root.path(),
        &[
            "search",
            "--json",
            "--bundle-id",
            "acme.beta",
            "--catalog-scope",
            "--offset",
            "0",
            "--limit",
            "10",
        ],
    ));
    let summary = rows
        .iter()
        .find(|row| row["kind"] == "query-summary")
        .unwrap();
    assert_eq!(summary["scan_limit"], 9, "{summary}");
    assert_eq!(summary["total_matches"], 3, "{summary}");
}

#[test]
fn scoped_lint_does_not_call_registered_cross_bundle_links_broken() {
    let root = fixture();
    fs::write(
        root.path().join("alpha/one.md"),
        "---\ntype: Note\ntitle: One\n---\n[Three](../beta/three.md)\n",
    )
    .unwrap();
    for scope in [
        vec!["lint", "--json"],
        vec!["lint", "--json", "--catalog-scope"],
    ] {
        let rows = records(&run(root.path(), &scope));
        assert!(
            !rows
                .iter()
                .any(|row| row["rule"] == "broken-link" && row["concept"] == "/one"),
            "{rows:?}"
        );
    }
}

fn cross_relationship(root: &std::path::Path, target_type: &str) {
    fs::write(root.join("alpha/ontology.yaml"), format!("okf_ontology: '0.1'\nconcepts:\n  Note:\n    references:\n      dependency:\n        selector: relations[].target\n        target: '{target_type}'\n        cardinality: 0..n\n    relationships:\n      dependencies:\n        reference: dependency\n        kind: depends-on\n        inverse: dependency-of\n")).unwrap();
    fs::write(
        root.join("alpha/one.md"),
        "---\ntype: Note\ntitle: One\nrelations: [{target: ../beta/three.md}]\n---\n",
    )
    .unwrap();
}

#[test]
fn expansion_resolves_scoped_target_outside_primary_scan_prefix() {
    let root = fixture();
    cross_relationship(root.path(), "acme.beta:Note");
    let rows = records(&run(
        root.path(),
        &[
            "search",
            "--json",
            "--catalog-scope",
            "--scan-limit",
            "1",
            "--limit",
            "1",
            "--expand",
            "dependencies",
        ],
    ));
    let relation = rows
        .iter()
        .find(|row| row["kind"] == "relationship")
        .expect("relationship");
    assert_eq!(relation["edge"]["status"], "resolved", "{rows:?}");
    let related = rows
        .iter()
        .find(|row| row["kind"] == "related-concept")
        .expect("related metadata");
    assert_eq!(related["bundle"], "acme.beta");
    assert_eq!(related["id"], "/three");
    let summary = rows
        .iter()
        .find(|row| row["kind"] == "query-summary")
        .unwrap();
    assert_eq!(summary["examined_documents"], 1);
    assert!(summary["total_matches"].is_null());
}

#[test]
fn equal_type_spelling_does_not_imply_cross_bundle_type_equivalence() {
    let root = fixture();
    cross_relationship(root.path(), "Note");
    let rows = records(&run(
        root.path(),
        &["lint", "--json", "--catalog-scope", "--fail-on", "never"],
    ));
    assert!(
        rows.iter().any(|row| row["code"] == "wrong-target-type"
            || row["code"] == "metadata-reference-target"
            || row["code"] == "metadata-reference-target-type"),
        "{rows:?}"
    );
    cross_relationship(root.path(), "acme.beta:Note");
    let rows = records(&run(
        root.path(),
        &["lint", "--json", "--catalog-scope", "--fail-on", "never"],
    ));
    assert!(
        !rows.iter().any(|row| row["code"] == "wrong-target-type"
            || row["code"] == "metadata-reference-target"
            || row["code"] == "metadata-reference-target-type"
            || row["code"] == "metadata-reference-missing"),
        "{rows:?}"
    );
}

#[test]
fn catalog_lint_preserves_existing_internal_body_link_error_severity() {
    let root = fixture();
    fs::write(
        root.path().join("alpha/one.md"),
        "---\ntype: Note\ntitle: One\n---\n[Missing](/ghost.md)\n",
    )
    .unwrap();
    let output = run(root.path(), &["lint", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let rows: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        rows.iter()
            .any(|row| row["rule"] == "broken-link" && row["severity"] == "error"),
        "{rows:?}"
    );
}

#[test]
fn historical_query_uses_committed_concepts_and_interpretation_settings() {
    let root = fixture();
    cross_relationship(root.path(), "acme.beta:Note");
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .current_dir(root.path())
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init", "-q"]);
    git(&["add", "--", "."]);
    git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "commit",
        "-qm",
        "Fixture",
    ]);
    fs::write(root.path().join("alpha/ontology.yaml"), "not valid: [").unwrap();
    fs::write(
        root.path().join("alpha/one.md"),
        "---\ntype: Note\ntitle: Changed after commit\n---\n",
    )
    .unwrap();
    fs::write(root.path().join("okf.toml"), "catalog = 'okf-catalog.yaml'\ndefault_bundle = {id = 'acme.alpha'}\n[bundle_settings.\"acme.alpha\".query]\nscan_limit = 1\n").unwrap();
    let rows = records(&run(
        root.path(),
        &[
            "search",
            "--json",
            "--catalog-scope",
            "--revision",
            "HEAD",
            "--offset",
            "0",
            "--limit",
            "10",
        ],
    ));
    let summary = rows
        .iter()
        .find(|row| row["kind"] == "query-summary")
        .unwrap();
    assert_eq!(summary["scan_limit"], 2, "{summary}");
    assert_eq!(summary["examined_documents"], 2);
    assert!(summary["scope"]
        .as_array()
        .unwrap()
        .iter()
        .all(|member| member["version"].as_str().unwrap().starts_with("git:")));
    assert!(!rows
        .iter()
        .any(|row| row["title"] == "Changed after commit"));
}
