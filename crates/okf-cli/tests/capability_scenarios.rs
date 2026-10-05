//! Black-box scenarios derived from docs/{multi-bundle,structured-metadata}-capability.md.
//! These supplement the core contract and earlier CLI integration tests.
use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn write(root: &Path, name: &str, contents: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}
fn command(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_okf"));
    c.current_dir(root)
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args(args);
    c
}
fn run(root: &Path, args: &[&str]) -> Output {
    command(root, args).output().unwrap()
}
fn rows(output: Output) -> Vec<Value> {
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
fn query(root: &Path, args: &[&str]) -> Vec<Value> {
    rows(run(root, args))
}
fn record<'a>(rows: &'a [Value], kind: &str) -> &'a Value {
    rows.iter()
        .find(|r| r["kind"] == kind)
        .unwrap_or_else(|| panic!("missing {kind}: {rows:?}"))
}
fn reject(root: &Path, args: &[&str]) {
    let result = run(root, args);
    assert!(
        !result.status.success(),
        "unexpected acceptance: {args:?}: {}",
        String::from_utf8_lossy(&result.stdout)
    );
}
fn catalog() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    write(
        d.path(),
        "okf.toml",
        "catalog='config/catalog.yaml'\ndefault_bundle={id='acme.a'}\n",
    );
    write(d.path(),"config/catalog.yaml","catalog_version: 1\nbundles:\n  acme.a: {location: {type: directory, path: ../a}}\n  acme.b: {location: {type: directory, path: ../b}}\n");
    write(
        d.path(),
        "a/one.md",
        "---\ntype: Note\ntitle: Alpha\n---\n[Beta](../b/one.md)\n",
    );
    write(d.path(), "b/one.md", "---\ntype: Note\ntitle: Beta\n---\n");
    d
}
fn git(root: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).trim().into()
}
fn commit(root: &Path) -> String {
    git(root, &["init", "-q"]);
    git(root, &["add", "."]);
    git(
        root,
        &[
            "-c",
            "user.name=Scenario",
            "-c",
            "user.email=scenario@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    );
    git(root, &["rev-parse", "HEAD"])
}
fn structured() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let doc = include_str!("../../../docs/structured-metadata-capability.md");
    let ontology = doc
        .split("```yaml\n# tooling/product-ontology.yaml\n")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    let concept = doc
        .split("```markdown\n---\ntype: Procedure")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    write(d.path(), "ontology.yaml", ontology);
    write(
        d.path(),
        "procedures/onboarding.md",
        &format!("---\ntype: Procedure{concept}"),
    );
    write(
        d.path(),
        "policies/access-control.md",
        "---\ntype: Policy\ntitle: Access control\n---\nPolicy.\n",
    );
    write(
        d.path(),
        "teams/platform.md",
        "---\ntype: Team\ntitle: Platform\n---\nTeam.\n",
    );
    d
}

#[test]
fn selection_precedence_and_catalog_relative_paths() {
    let d = catalog();
    let root = d.path();
    let title = |o: Output| {
        rows(o)
            .iter()
            .find_map(|r| r["title"].as_str().map(str::to_owned))
            .unwrap()
    };
    assert_eq!(title(run(root, &["list", "--json"])), "Alpha");
    assert_eq!(title(run(&root.join("b"), &["list", "--json"])), "Beta");
    assert_eq!(
        title(
            command(&root.join("b"), &["list", "--json"])
                .env("OKF_BUNDLE", "../a")
                .output()
                .unwrap()
        ),
        "Alpha"
    );
    assert_eq!(
        title(
            command(root, &["list", "--bundle-id", "acme.b", "--json"])
                .env("OKF_BUNDLE", "a")
                .output()
                .unwrap()
        ),
        "Beta"
    );
    assert_eq!(
        title(
            command(root, &["list", "b", "--json"])
                .env("OKF_BUNDLE", "a")
                .output()
                .unwrap()
        ),
        "Beta"
    );
    write(root, "okf.toml", "catalog='config/catalog.yaml'\n");
    reject(root, &["list", "--json"]);
    write(
        root,
        "config/catalog.yaml",
        "catalog_version: 1\nbundles:\n  acme.b: {location: {type: directory, path: ../b}}\n",
    );
    assert_eq!(title(run(root, &["list", "--json"])), "Beta");
    reject(root, &["list", "--bundle-id", "b"]);
    reject(root, &["list", "b", "--bundle-id", "acme.b"]);
}
#[test]
fn override_file_and_environment_precedence_are_reported() {
    let d = catalog();
    let root = d.path();
    write(
        root,
        "c/one.md",
        "---\ntype: Note\ntitle: Override C\n---\n",
    );
    write(
        root,
        "d/one.md",
        "---\ntype: Note\ntitle: Override D\n---\n",
    );
    write(
        root,
        "okf.toml",
        "catalog='config/catalog.yaml'\ncatalog_overrides='local/override.yaml'\n",
    );
    write(root, "local/override.yaml", "bundles:\n  acme.b: ../c\n");
    write(root, "env.yaml", "bundles:\n  acme.b: d\n");
    for (override_env, suffix) in [(false, "c"), (true, "d")] {
        let mut c = command(root, &["catalog", "--json"]);
        if override_env {
            c.env("OKF_CATALOG_OVERRIDES", "env.yaml");
        }
        let records = rows(c.output().unwrap());
        let b = records
            .iter()
            .find(|r| r["kind"] == "bundle-registration" && r["id"] == "acme.b")
            .unwrap();
        assert_eq!(b["overridden"], true);
        assert!(b["root"].as_str().unwrap().ends_with(suffix));
        assert!(b["configured_root"].as_str().unwrap().ends_with("b"));
    }
}
#[test]
fn invalid_catalog_is_separate_from_standalone_conformance() {
    let d = catalog();
    write(
        d.path(),
        "config/catalog.yaml",
        "catalog_version: 1\nbundles:\n  plain: {location: {type: directory, path: ../a}}\n",
    );
    reject(d.path(), &["catalog", "--json"]);
    rows(run(d.path(), &["validate", "a", "--json"]));
    assert!(query(d.path(), &["list", "a", "--json"])
        .iter()
        .any(|r| r["title"] == "Alpha"));
}
#[test]
fn catalog_rejects_selection_scope_and_revision_flags() {
    let d = catalog();
    for extra in [
        vec!["--bundle-id", "acme.a"],
        vec!["--scope-bundle", "acme.b"],
        vec!["--catalog-scope"],
        vec!["--revision", "HEAD"],
    ] {
        let mut args = vec!["catalog", "--json"];
        args.extend(extra);
        reject(d.path(), &args);
    }
    for args in [
        vec!["show", "one", "--scope-bundle", "acme.b"],
        vec!["validate", "--catalog-scope"],
        vec!["lint", "--revision", "HEAD"],
        vec!["show", "one", "--revision", "HEAD"],
    ] {
        reject(d.path(), &args);
    }
}
#[test]
fn unavailable_scope_members_and_unknown_ids_are_distinct() {
    let d = catalog();
    let path = d.path().join("config/catalog.yaml");
    fs::write(
        &path,
        format!(
            "{}  acme.missing: {{location: {{type: directory, path: ../missing}}}}\n",
            fs::read_to_string(&path).unwrap()
        ),
    )
    .unwrap();
    let records = query(
        d.path(),
        &["list", "--catalog-scope", "--full-scan", "--json"],
    );
    assert!(record(&records, "scope")["unavailable"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "acme.missing"));
    assert_eq!(record(&records, "query-summary")["total_matches"], 2);
    reject(d.path(), &["list", "--scope-bundle", "acme.unknown"]);
}
#[test]
fn explicit_settings_do_not_leak_to_unregistered_paths() {
    let d = catalog();
    let root = d.path();
    write(root, "outside/x.md", "---\ntype: Note\n---\n");
    write(root,"okf.toml","catalog='config/catalog.yaml'\ndefault_bundle={id='acme.a'}\n[bundle_settings.\"acme.a\"]\nontology='missing.yaml'\n[bundle_settings.default.query]\nscan_limit=1\n");
    reject(root, &["list", "--bundle-id", "acme.a"]);
    let records = query(
        root,
        &[
            "list", "outside", "--offset", "0", "--limit", "10", "--json",
        ],
    );
    assert_eq!(record(&records, "query-summary")["scan_limit"], 1000);
}
#[test]
fn query_and_lint_configuration_reject_invalid_limits_and_selectors() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "a.md", "---\ntype: Note\n---\n");
    for config in [
        "[bundle_settings.default.query]\nscan_limit=0",
        "[bundle_settings.default.query]\nfull_scan=true",
        "[bundle_settings.default.lint]\nfinding_budget=0",
        "[bundle_settings.default.facets]\nmax_distinct_values=0",
        "[bundle_settings.default.lint]\nindex_coverage='silent'",
        "[bundle_settings.default.views.bad]\ncolumns=['$.title']",
    ] {
        write(d.path(), "okf.toml", config);
        reject(d.path(), &["list", "--json"]);
    }
}
#[test]
fn snapshots_cover_digest_label_commit_ref_missing_and_mismatch() {
    let d = catalog();
    let root = d.path();
    let commit = commit(root);
    let bytes = fs::read(root.join("b/one.md")).unwrap();
    let digest = okf_core::fingerprint::canonicalize::sha256_hex(&bytes);
    let cases = [
        (json!({"label":"Approved"}), "unverified"),
        (json!({"digest":digest}), "matched"),
        (json!({"digest":"0".repeat(64)}), "mismatched"),
        (json!({"commit":commit}), "matched"),
        (json!({"ref":"HEAD"}), "matched"),
        (json!({"ref":"missing-ref"}), "unavailable"),
        (json!({"commit":"HEAD"}), "mismatched"),
        (json!({"digest":"bad"}), "mismatched"),
        (
            json!({"commit":commit,"digest":"0".repeat(64)}),
            "mismatched",
        ),
    ];
    for (request, status) in cases {
        write(root,"a/one.md",&format!("---\ntype: Note\nsources:\n- id: policy\n  resource: ../b/one.md\n  bundle_ref:\n    id: acme.b\n    path: one.md\n    snapshot: {request}\n---\n[Beta](../b/one.md)\n"));
        let before = fs::read(root.join("a/one.md")).unwrap();
        let records = query(
            root,
            &["links", "one", "--scope-bundle", "acme.b", "--json"],
        );
        let source = records
            .iter()
            .find(|r| r["location"] == "sources[0].resource")
            .unwrap();
        assert_eq!(
            source["snapshot"]["status"], status,
            "{request}: {records:?}"
        );
        assert_eq!(source["target"]["version"], "working-tree");
        assert_eq!(
            source["snapshot"]["candidate"].is_null(),
            status == "matched"
        );
        assert!(records
            .iter()
            .filter(|r| r["location"]
                .as_str()
                .is_some_and(|s| s.starts_with("body:")))
            .all(|r| r["snapshot"].is_null()));
        assert_eq!(fs::read(root.join("a/one.md")).unwrap(), before);
    }
}
#[test]
fn declared_url_mapping_is_unchecked_and_never_fetched() {
    let d = catalog();
    write(d.path(),"a/one.md","---\ntype: Note\nsources:\n- resource: https://does-not-exist.invalid/policy.md\n  bundle_ref: {id: acme.b, path: one.md}\n---\n");
    let records = query(
        d.path(),
        &["links", "one", "--scope-bundle", "acme.b", "--json"],
    );
    let edge = record(&records, "bundle-edge");
    assert_eq!(edge["status"], "unchecked-equivalence");
    assert!(edge["evidence"].as_str().unwrap().contains("unchecked"));
}
#[test]
fn historical_external_ontology_reports_current_digest_without_replacing_internal_history() {
    let d = catalog();
    let external = tempfile::tempdir().unwrap();
    write(
        external.path(),
        "ontology.yaml",
        "okf_ontology: '0.1'\nconcepts: {Note: {}}\n",
    );
    write(d.path(),"okf.toml",&format!("catalog='config/catalog.yaml'\ndefault_bundle={{id='acme.a'}}\n[bundle_settings.\"acme.a\"]\nontology='{}'\n",external.path().join("ontology.yaml").display()));
    let revision = commit(d.path());
    write(
        external.path(),
        "ontology.yaml",
        "okf_ontology: '0.1'\nconcepts: {Note: {description: Current}}\n",
    );
    let records = query(
        d.path(),
        &["list", "--revision", &revision, "--full-scan", "--json"],
    );
    let interpretation = &record(&records, "query-summary")["interpretation"][0];
    assert_eq!(
        interpretation["ontology_digest"],
        okf_core::fingerprint::canonicalize::sha256_hex(
            &fs::read(external.path().join("ontology.yaml")).unwrap()
        )
    );
    assert!(record(&records, "query-summary")["scope"][0]["version"]
        .as_str()
        .unwrap()
        .starts_with("git:"));
}
#[test]
fn published_structured_example_lints_and_expands_paired_obligations() {
    let d = structured();
    let root = d.path();
    let findings = query(root, &["lint", "--fail-on", "never", "--json"]);
    assert!(
        !findings.iter().any(|r| r["rule"] == "ontology-violation"),
        "{findings:?}"
    );
    let records = query(
        root,
        &[
            "search",
            "--type",
            "Procedure",
            "--expand",
            "policy-relations",
            "--expand",
            "obligations",
            "--json",
        ],
    );
    assert_eq!(
        records
            .iter()
            .filter(|r| r["kind"] == "relationship")
            .count(),
        2
    );
    let obligation = records
        .iter()
        .find(|r| r["edge"]["rule"] == "obligations")
        .unwrap();
    assert_eq!(obligation["edge"]["attributes"]["deadline"]["within"], 24);
    assert_eq!(
        obligation["edge"]["attributes"]["trigger"],
        "account-created"
    );
    assert_eq!(obligation["edge"]["inverse"], "obligated-by");
}
#[test]
fn projections_distinguish_missing_null_literal_keys_and_authored_identity() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(),"a.md","---\ntype: Note\nbundle: authored\nversion: authored\npolicy.status: literal\nnullable: null\ntags: []\n---\n");
    let records = query(
        d.path(),
        &[
            "show",
            "a",
            "--project",
            "$id",
            "--project",
            "bundle",
            "--project",
            "version",
            "--project",
            "[\"policy.status\"]",
            "--project",
            "nullable",
            "--project",
            "missing",
            "--project",
            "tags[]",
            "--json",
        ],
    );
    let fields = record(&records, "projection")["fields"].as_array().unwrap();
    assert_eq!(fields[0]["value"], "/a");
    assert_eq!(fields[1]["occurrences"][0]["value"], "authored");
    assert_eq!(fields[3]["occurrences"][0]["value"], "literal");
    assert_eq!(fields[4]["present"], true);
    assert!(fields[4]["occurrences"][0]["value"].is_null());
    assert_eq!(fields[5]["present"], false);
    assert_eq!(fields[6]["empty_lists"], 1);
}
#[test]
fn views_preserve_legacy_json_until_explicit_machine_opt_in() {
    let d = structured();
    let root = d.path();
    write(root,"okf.toml","[bundle_settings.default.views.compact]\ncolumns=['$id','title']\nexpand=['obligations']\n[bundle_settings.default.views.extended]\ncolumns=['$id','title']\nexpand=['obligations']\nextended_output=true\n");
    let ordinary = query(
        root,
        &[
            "search",
            "--type",
            "Procedure",
            "--view",
            "compact",
            "--json",
        ],
    );
    assert!(ordinary.iter().any(|r| r["title"] == "Account onboarding"));
    assert!(!ordinary
        .iter()
        .any(|r| r["kind"] == "projection" || r["kind"] == "relationship"));
    let extended = query(
        root,
        &[
            "search",
            "--type",
            "Procedure",
            "--view",
            "extended",
            "--json",
        ],
    );
    record(&extended, "projection");
    record(&extended, "relationship");
    let human = run(
        root,
        &["search", "--type", "Procedure", "--view", "compact"],
    );
    assert!(human.status.success());
    assert!(String::from_utf8_lossy(&human.stdout).contains("obliges"));
    let disabled = run(
        root,
        &[
            "search",
            "--type",
            "Procedure",
            "--view",
            "compact",
            "--no-expand",
        ],
    );
    assert!(!String::from_utf8_lossy(&disabled.stdout).contains("obliges"));
}
#[test]
fn typed_filter_errors_and_literal_key_compatibility() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(),"a.md","---\ntype: Note\nx: 2\ntags: [security, production]\npolicy.status: live\nobj: {key: value}\n---\n");
    for filter in [
        "x=\"2\"",
        "x=null",
        "tags[] in []",
        "tags[] in [[\"security\"]]",
        "obj=\"value\"",
        "tags[*]=\"security\"",
        "$.x=2",
    ] {
        reject(d.path(), &["search", "--facet-filter", filter, "--json"]);
    }
    for args in [
        vec!["search", "--field", "policy.status=live", "--json"],
        vec![
            "search",
            "--facet-filter",
            "tags[]=\"security\"",
            "--facet-filter",
            "tags[]=\"production\"",
            "--json",
        ],
        vec![
            "search",
            "--facet-filter",
            "tags[] in [\"absent\",\"security\"]",
            "--json",
        ],
    ] {
        assert!(query(d.path(), &args).iter().any(|r| r["id"] == "/a"));
    }
    let none = query(
        d.path(),
        &[
            "search",
            "--facet-filter",
            "tags[] not in [\"production\"]",
            "--json",
        ],
    );
    assert_eq!(record(&none, "query-summary")["total_matches"], 0);
}
#[test]
fn facets_threshold_output_cap_and_explicit_inclusion_are_independent() {
    let d = tempfile::tempdir().unwrap();
    let root = d.path();
    for i in 0..1001 {
        write(
            root,
            &format!("{i:04}.md"),
            &format!("---\ntype: Note\nvalue: {i}\n---\n"),
        );
    }
    let hundred = query(
        root,
        &[
            "search",
            "--scan-limit",
            "100",
            "--facet",
            "value",
            "--limit",
            "1",
            "--json",
        ],
    );
    assert_eq!(
        record(&hundred, "facet")["values"]
            .as_array()
            .unwrap()
            .len(),
        100
    );
    assert_eq!(record(&hundred, "facet")["complete"], false);
    let excluded = query(
        root,
        &[
            "search",
            "--scan-limit",
            "101",
            "--facet",
            "value",
            "--limit",
            "1",
            "--json",
        ],
    );
    assert_eq!(
        record(&excluded, "facet-excluded")["reason"],
        "high-cardinality"
    );
    write(
        root,
        "okf.toml",
        "[bundle_settings.default.facets]\ninclude_high_cardinality=['value']\n",
    );
    let full = query(
        root,
        &[
            "search",
            "--full-scan",
            "--facet",
            "value",
            "--limit",
            "1",
            "--json",
        ],
    );
    let facet = record(&full, "facet");
    assert_eq!(facet["complete"], true);
    assert_eq!(facet["truncated"], true);
    assert_eq!(facet["omitted_values"], 1);
    assert_eq!(facet["values"].as_array().unwrap().len(), 1000);
    assert!(full
        .iter()
        .any(|r| r["kind"] == "warning" && r["reason"] == "facet-output-limit"));
}
#[test]
fn facets_ignore_missing_null_and_empty_but_keep_empty_string_and_types() {
    let d = tempfile::tempdir().unwrap();
    for (i, meta) in [
        "value: null",
        "other: missing",
        "value: ''",
        "value: false",
        "value: 2",
        "value: '2'",
        "value: 2",
    ]
    .iter()
    .enumerate()
    {
        write(
            d.path(),
            &format!("{i}.md"),
            &format!("---\ntype: Note\n{meta}\n---\n"),
        );
    }
    let records = query(d.path(), &["search", "--facet", "value", "--json"]);
    let values = record(&records, "facet")["values"].as_array().unwrap();
    assert_eq!(values.len(), 4);
    for v in [json!(""), json!(false), json!(2), json!("2")] {
        let expected = if v == json!(2) { 2 } else { 1 };
        assert!(values
            .iter()
            .any(|r| r["value"] == v && r["count"] == expected));
    }
    assert_eq!(values[0]["value"], 2);
    write(
        d.path(),
        "okf.toml",
        "[bundle_settings.default.facets]\nfields=['value','type']\n",
    );
    let ordered = query(d.path(), &["search", "--facets", "--json"]);
    let fields: Vec<_> = ordered
        .iter()
        .filter(|r| r["kind"] == "facet")
        .map(|r| r["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, vec!["value", "type"]);
    write(d.path(), "object.md", "---\ntype: Note\nvalue: {}\n---\n");
    let invalid = query(d.path(), &["search", "--facet", "value", "--json"]);
    assert_eq!(record(&invalid, "facet")["complete"], false);
    assert!(!record(&invalid, "facet")["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
}
#[test]
fn pagination_is_globally_ordered_and_beyond_end_has_no_continuation() {
    let d = catalog();
    let root = d.path();
    let page0 = query(
        root,
        &[
            "search",
            "--bundle-id",
            "acme.b",
            "--catalog-scope",
            "--full-scan",
            "--offset",
            "0",
            "--limit",
            "1",
            "--sort",
            "id",
            "--json",
        ],
    );
    assert_eq!(record(&page0, "concept-identity")["bundle"], "acme.a");
    assert_eq!(record(&page0, "query-summary")["next_offset"], 1);
    let page1 = query(
        root,
        &[
            "search",
            "--bundle-id",
            "acme.b",
            "--catalog-scope",
            "--full-scan",
            "--offset",
            "1",
            "--limit",
            "1",
            "--sort",
            "id",
            "--json",
        ],
    );
    assert_eq!(record(&page1, "concept-identity")["bundle"], "acme.b");
    assert_eq!(record(&page1, "query-summary")["has_more"], false);
    let beyond = query(
        root,
        &[
            "search",
            "--catalog-scope",
            "--full-scan",
            "--offset",
            "5",
            "--limit",
            "1",
            "--json",
        ],
    );
    assert_eq!(record(&beyond, "query-summary")["returned"], 0);
    assert!(record(&beyond, "query-summary")["next_offset"].is_null());
    for extra in [
        vec!["--offset", "-1", "--limit", "1"],
        vec!["--offset", "0", "--limit", "0"],
        vec!["--scan-limit", "0"],
        vec!["--full-scan", "--scan-limit", "1"],
    ] {
        let mut args = vec!["search"];
        args.extend(extra);
        reject(root, &args);
    }
}
#[test]
fn human_cells_escape_delimiters_and_mark_truncation_and_missing() {
    let d = tempfile::tempdir().unwrap();
    write(
        d.path(),
        "a.md",
        &format!(
            "---\ntype: Note\nvalue: \"a\\tb\\nc\"\nlong: '{}'\n---\n",
            "x".repeat(200)
        ),
    );
    let o = run(d.path(), &["list", "--columns", "value,long,missing"]);
    assert!(o.status.success());
    let s = String::from_utf8(o.stdout).unwrap();
    assert!(s.contains("a\\tb\\nc"));
    assert!(s.contains('…'));
    assert!(s.contains('—'));
}

#[test]
fn human_expansion_accepts_computed_target_fields() {
    let d = structured();
    let o = run(
        d.path(),
        &[
            "search",
            "--type",
            "Procedure",
            "--expand",
            "obligations",
            "--target-field",
            "$id",
        ],
    );
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stdout).contains("/teams/platform"));
}
#[test]
fn expansion_rule_qualification_and_occurrence_deduplication() {
    let d = catalog();
    let root = d.path();
    for bundle in ["a", "b"] {
        write(root,&format!("{bundle}/ontology.yaml"),"okf_ontology: '0.1'\nconcepts:\n  Note:\n    references:\n      peers: {selector: 'relations[].target', target: Note, cardinality: 0..n}\n    relationships:\n      deps: {reference: peers, kind: depends-on, inverse: dependency-of}\n");
        write(
            root,
            &format!("{bundle}/one.md"),
            "---\ntype: Note\nrelations: [{target: /one.md}, {target: /one.md}]\n---\n",
        );
    }
    let short = query(
        root,
        &["search", "--catalog-scope", "--expand", "deps", "--json"],
    );
    assert_eq!(
        short.iter().filter(|r| r["kind"] == "relationship").count(),
        2
    );
    assert!(short
        .iter()
        .filter(|r| r["kind"] == "relationship")
        .all(|r| r["bundle"] == "acme.a"));
    let qualified = query(
        root,
        &[
            "search",
            "--catalog-scope",
            "--expand",
            "acme.a:deps",
            "--expand",
            "acme.b:deps",
            "--json",
        ],
    );
    assert_eq!(
        qualified
            .iter()
            .filter(|r| r["kind"] == "relationship")
            .count(),
        4
    );
    assert_eq!(
        qualified
            .iter()
            .filter(|r| r["kind"] == "related-concept")
            .count(),
        2
    );
    let limited = query(
        root,
        &[
            "search",
            "--catalog-scope",
            "--expand",
            "acme.a:deps",
            "--expand",
            "acme.b:deps",
            "--expansion-edges",
            "1",
            "--json",
        ],
    );
    assert_eq!(record(&limited, "expansion-summary")["emitted_edges"], 2);
    assert_eq!(record(&limited, "expansion-summary")["total_edges"], 4);
    assert!(limited.iter().any(|r| r["reason"] == "edge-limit"));
}
#[test]
fn expansion_reports_missing_wrong_type_unknown_kind_and_byte_bounds() {
    let d = structured();
    let root = d.path();
    let path = root.join("procedures/onboarding.md");
    let original = fs::read_to_string(&path).unwrap();
    for (text, status) in [
        (
            original.replace("modality: obliges", "modality: unknown"),
            "unknown-kind",
        ),
        (
            original.replace("bearer: /teams/platform.md", "bearer: /absent.md"),
            "missing",
        ),
        (
            original.replace(
                "bearer: /teams/platform.md",
                "bearer: /policies/access-control.md",
            ),
            "wrong-type",
        ),
    ] {
        fs::write(&path, text).unwrap();
        let records = query(
            root,
            &[
                "search",
                "--type",
                "Procedure",
                "--expand",
                "obligations",
                "--json",
            ],
        );
        assert_eq!(record(&records, "relationship")["edge"]["status"], status);
    }
    fs::write(&path, original).unwrap();
    let records = query(
        root,
        &[
            "search",
            "--type",
            "Procedure",
            "--expand",
            "obligations",
            "--expansion-bytes",
            "1",
            "--json",
        ],
    );
    let summary = record(&records, "expansion-summary");
    assert_eq!(summary["truncated"], true);
    assert_eq!(summary["emitted_bytes"], 0);
    assert!(summary["total_edges"].is_null());
    for (flag, bad) in [
        ("--expansion-edges", "101"),
        ("--expansion-targets", "1001"),
        ("--expansion-bytes", "1048577"),
        ("--expansion-edges", "0"),
    ] {
        reject(root, &["search", flag, bad]);
    }
}
#[test]
fn nested_targets_participate_in_affected_analysis_without_mutation() {
    let d = structured();
    let root = d.path();
    let before = fs::read(root.join("procedures/onboarding.md")).unwrap();
    let records = query(
        root,
        &["affected", "--changed", "/teams/platform.md", "--json"],
    );
    assert!(
        records
            .iter()
            .any(|r| r.to_string().contains("/procedures/onboarding")),
        "{records:?}"
    );
    assert_eq!(
        fs::read(root.join("procedures/onboarding.md")).unwrap(),
        before
    );
}
#[test]
fn lint_severity_and_finding_budget_preserve_conformance_separation() {
    let d = structured();
    let root = d.path();
    let path = root.join("procedures/onboarding.md");
    fs::write(
        &path,
        fs::read_to_string(&path)
            .unwrap()
            .replace("within: 24", "within: 0"),
    )
    .unwrap();
    query(root, &["validate", "--json"]);
    write(root,"okf.toml","[bundle_settings.default.lint]\nontology_violation='error'\nfinding_budget=1\nindex_coverage='info'\n");
    let records = query(root, &["lint", "--fail-on", "never", "--json"]);
    assert!(
        records
            .iter()
            .any(|r| r["code"] == "metadata-range" && r["severity"] == "error"),
        "{records:?}"
    );
    reject(root, &["lint", "--fail-on", "error", "--json"]);
    let bytes = fs::read(&path).unwrap();
    query(root, &["lint", "--fail-on", "never", "--json"]);
    assert_eq!(fs::read(path).unwrap(), bytes);
}
#[test]
fn index_coverage_uses_real_immediate_links_not_mentions_or_images() {
    let d = tempfile::tempdir().unwrap();
    let root = d.path();
    for path in ["a.md", "space name.md", "sub/deep.md"] {
        write(root, path, "---\ntype: Note\n---\n");
    }
    write(root, "artifacts/file.txt", "opaque");
    write(root, ".git/ignored.md", "---\ntype: Note\n---\n");
    let good="# Index\n[A][a]\n[a]: a#section\n[Space](space%20name.md)\n[Sub](sub/index.md)\n[A again](a.md)\n";
    write(root, "index.md", good);
    write(root, "sub/index.md", "# Sub\n[Deep](deep.md)\n");
    let before = fs::read(root.join("index.md")).unwrap();
    let records = query(root, &["lint", "--fail-on", "never", "--json"]);
    assert!(
        !records.iter().any(|r| r["rule"] == "index-coverage"),
        "{records:?}"
    );
    assert_eq!(fs::read(root.join("index.md")).unwrap(), before);
    write(root,"index.md","# Index\na.md\n![A](a.md)\n`[Space](space%20name.md)`\n[Deep](sub/deep.md)\n[Remote](https://example.invalid/a.md)\n");
    let bad = query(root, &["lint", "--fail-on", "never", "--json"]);
    assert_eq!(
        bad.iter()
            .filter(|r| r["code"] == "index-missing-entry")
            .count(),
        3,
        "{bad:?}"
    );
    write(
        root,
        "okf.toml",
        "[bundle_settings.default.lint]\nindex_exclude=['sub/**','space name.md']\n",
    );
    let excluded = query(root, &["lint", "--fail-on", "never", "--json"]);
    assert_eq!(
        excluded
            .iter()
            .filter(|r| r["code"] == "index-missing-entry")
            .count(),
        1,
        "{excluded:?}"
    );
    assert_eq!(
        query(root, &["list", "--full-scan", "--json"])
            .iter()
            .filter(|r| r.get("trust_tier").is_some())
            .count(),
        3
    );
}
#[test]
fn scan_exact_boundary_is_complete_and_limits_never_hide_empty_scan_warning() {
    let d = tempfile::tempdir().unwrap();
    let root = d.path();
    for name in ["a", "b"] {
        write(root, &format!("{name}.md"), "---\ntype: Note\n---\n");
    }
    let exact = query(
        root,
        &["list", "--scan-limit", "2", "--limit", "1", "--json"],
    );
    assert_eq!(record(&exact, "query-summary")["scan_complete"], true);
    assert_eq!(record(&exact, "query-summary")["total_matches"], 2);
    assert!(exact.iter().any(|r| r["reason"] == "result-limit"));
    let empty = query(
        root,
        &["search", "--type", "Absent", "--scan-limit", "1", "--json"],
    );
    assert_eq!(record(&empty, "query-summary")["observed_matches"], 0);
    assert!(record(&empty, "query-summary")["total_matches"].is_null());
    assert!(empty
        .iter()
        .any(|r| r["kind"] == "warning" && r["reason"] == "scan-limit"));
    let human = run(root, &["search", "--type", "Absent", "--scan-limit", "1"]);
    assert!(String::from_utf8_lossy(&human.stdout).contains("scan-limit"));
}
#[cfg(unix)]
#[test]
fn historical_git_calls_disable_lazy_fetch_and_never_invoke_fetch() {
    use std::os::unix::fs::PermissionsExt;
    let d = catalog();
    let root = d.path();
    let revision = commit(root);
    let which = Command::new("sh")
        .args(["-c", "command -v git"])
        .output()
        .unwrap();
    let real = String::from_utf8(which.stdout).unwrap();
    let shim = tempfile::tempdir().unwrap();
    write(shim.path(),"git",&format!("#!/bin/sh\nprintf '%s|%s\\n' \"$GIT_NO_LAZY_FETCH\" \"$*\" >> \"$OKF_GIT_AUDIT\"\nexec '{}' \"$@\"\n",real.trim().replace('\'',"'\\''")));
    fs::set_permissions(shim.path().join("git"), fs::Permissions::from_mode(0o755)).unwrap();
    let log = root.join("git-audit.log");
    let mut c = command(
        root,
        &[
            "list",
            "--catalog-scope",
            "--revision",
            &revision,
            "--full-scan",
            "--json",
        ],
    );
    c.env(
        "PATH",
        format!(
            "{}:{}",
            shim.path().display(),
            std::env::var("PATH").unwrap()
        ),
    )
    .env("OKF_GIT_AUDIT", &log);
    rows(c.output().unwrap());
    let calls = fs::read_to_string(log).unwrap();
    assert!(!calls.is_empty());
    for line in calls.lines() {
        assert!(line.starts_with("1|"), "lazy fetch enabled: {line}");
        assert!(!line
            .split_whitespace()
            .any(|word| matches!(word, "fetch" | "pull" | "clone")));
    }
}

#[test]
fn configured_sidecar_has_same_effect_on_affected_stats_and_rendering_as_root_ontology() {
    let d = structured();
    let root = d.path();
    let commands = [
        vec!["affected", "--changed", "/teams/platform", "--json"],
        vec!["stats", "--json"],
        vec!["docs", "--format", "graphml"],
    ];
    let before: Vec<_> = commands
        .iter()
        .map(|args| {
            let o = run(root, args);
            assert!(o.status.success());
            o.stdout
        })
        .collect();
    fs::rename(root.join("ontology.yaml"), root.join("sidecar.yaml")).unwrap();
    write(
        root,
        "okf.toml",
        "[bundle_settings.default]\nontology='sidecar.yaml'\n",
    );
    for (args, expected) in commands.iter().zip(before) {
        let o = run(root, args);
        assert!(o.status.success());
        assert_eq!(o.stdout, expected, "configured sidecar ignored: {args:?}");
    }
}

#[test]
fn every_revision_capable_command_reads_available_local_commits() {
    let d = catalog();
    let root = d.path();
    let rev = commit(root);
    let commands = [
        vec!["graph"],
        vec!["links", "one"],
        vec!["backlinks", "acme.b:/one"],
        vec!["resolve", "../b/one.md", "--from", "one"],
        vec!["affected", "--changed", "acme.b:/one"],
        vec!["search"],
        vec!["list"],
    ];
    for mut args in commands {
        args.extend(["--catalog-scope", "--revision", &rev, "--json"]);
        let r = query(root, &args);
        let scope = record(&r, "scope");
        assert_eq!(scope["examined"].as_array().unwrap().len(), 2);
        assert!(
            scope["examined"]
                .as_array()
                .unwrap()
                .iter()
                .all(|s| s["version"] == format!("git:{rev}")),
            "{args:?}: {scope}"
        );
    }
}
#[test]
fn same_catalog_can_span_independent_git_repositories() {
    let d = catalog();
    let root = d.path();
    let a = commit(&root.join("a"));
    let b = commit(&root.join("b"));
    assert_ne!(a, b);
    let r = query(
        root,
        &["graph", "--catalog-scope", "--revision", "HEAD", "--json"],
    );
    let scope = record(&r, "scope");
    let examined = scope["examined"].as_array().unwrap();
    assert!(examined
        .iter()
        .any(|s| s["id"] == "acme.a" && s["version"] == format!("git:{a}")));
    assert!(examined
        .iter()
        .any(|s| s["id"] == "acme.b" && s["version"] == format!("git:{b}")));
}
#[test]
fn missing_historical_config_and_configured_ontology_are_diagnosed() {
    let d = catalog();
    let root = d.path();
    fs::remove_file(root.join("okf.toml")).unwrap();
    let old = commit(root);
    write(
        root,
        "okf.toml",
        "catalog='config/catalog.yaml'\ndefault_bundle={id='acme.a'}\n",
    );
    let r = run(
        root,
        &["graph", "--catalog-scope", "--revision", &old, "--json"],
    );
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("historical configuration unavailable"));
    // Historical config exists but names an ontology that did not exist at that revision.
    write(root,"okf.toml","catalog='config/catalog.yaml'\ndefault_bundle={id='acme.a'}\n[bundle_settings.\"acme.a\"]\nontology='later.yaml'\n");
    let configured = commit(root);
    write(
        root,
        "later.yaml",
        "okf_ontology: '0.1'\nconcepts: {Note: {}}\n",
    );
    let r = run(
        root,
        &[
            "graph",
            "--catalog-scope",
            "--revision",
            &configured,
            "--json",
        ],
    );
    assert!(!r.status.success());
    assert!(
        String::from_utf8_lossy(&r.stderr).contains("configured historical ontology unavailable")
    );
}
#[test]
fn source_drift_is_reported_separately_from_successful_target_and_snapshot() {
    let d = catalog();
    let root = d.path();
    commit(root);
    let digest =
        okf_core::fingerprint::canonicalize::sha256_hex(&fs::read(root.join("b/one.md")).unwrap());
    write(root,"a/one.md",&format!("---\ntype: Note\nsources:\n- resource: b/one.md\n  kind: git-path\n  fingerprint: {{blob: wrong}}\n  bundle_ref:\n    id: acme.b\n    path: one.md\n    snapshot: {{digest: '{digest}'}}\n---\n"));
    let r = query(
        root,
        &["links", "one", "--scope-bundle", "acme.b", "--json"],
    );
    let edge = record(&r, "bundle-edge");
    assert_eq!(edge["status"], "resolved");
    assert_eq!(edge["snapshot"]["status"], "matched");
    assert_eq!(edge["fingerprint_status"], "changed");
}
#[test]
fn unreadable_index_content_reports_io_instead_of_empty_coverage() {
    let d = tempfile::tempdir().unwrap();
    write(d.path(), "a.md", "---\ntype: Note\n---\n");
    fs::write(d.path().join("index.md"), [0xff]).unwrap();
    let r = run(d.path(), &["lint", "--fail-on", "never", "--json"]);
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("index.md"));
}
#[cfg(unix)]
#[test]
fn catalog_canonicalization_rejects_symlink_aliases_of_registered_roots() {
    let d = catalog();
    std::os::unix::fs::symlink(d.path().join("a"), d.path().join("alias")).unwrap();
    write(d.path(),"config/catalog.yaml","catalog_version: 1\nbundles:\n  acme.a: {location: {type: directory, path: ../a}}\n  acme.alias: {location: {type: directory, path: ../alias}}\n");
    reject(d.path(), &["catalog", "--json"]);
}

#[cfg(unix)]
#[test]
fn partial_clone_missing_blob_is_unavailable_without_invoking_remote_helper() {
    use std::os::unix::fs::PermissionsExt;
    let d = catalog();
    let root = d.path();
    let revision = commit(root);
    let oid = git(root, &["rev-parse", "HEAD:b/one.md"]);
    fs::remove_file(root.join(".git/objects").join(&oid[..2]).join(&oid[2..])).unwrap();
    git(root, &["config", "extensions.partialClone", "origin"]);
    git(root, &["config", "remote.origin.promisor", "true"]);
    git(
        root,
        &["config", "remote.origin.partialclonefilter", "blob:none"],
    );
    git(root, &["config", "remote.origin.url", "audit::unused"]);
    git(root, &["config", "protocol.audit.allow", "always"]);
    let helper = tempfile::tempdir().unwrap();
    let probe = root.join("fetch-attempt");
    write(
        helper.path(),
        "git-remote-audit",
        "#!/bin/sh\nprintf attempted > \"$OKF_FETCH_PROBE\"\nexit 1\n",
    );
    fs::set_permissions(
        helper.path().join("git-remote-audit"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let path = format!(
        "{}:{}",
        helper.path().display(),
        std::env::var("PATH").unwrap()
    );
    // Control: the synthetic partial clone would invoke our local helper without the guard.
    let control = Command::new("git")
        .current_dir(root)
        .env("PATH", &path)
        .env("OKF_FETCH_PROBE", &probe)
        .env_remove("GIT_NO_LAZY_FETCH")
        .args(["cat-file", "-p", &oid])
        .output()
        .unwrap();
    assert!(!control.status.success());
    assert!(
        probe.exists(),
        "control did not exercise lazy fetching: {}",
        String::from_utf8_lossy(&control.stderr)
    );
    fs::remove_file(&probe).unwrap();
    let output = command(
        root,
        &[
            "graph",
            "--catalog-scope",
            "--revision",
            &revision,
            "--json",
        ],
    )
    .env("PATH", &path)
    .env("OKF_FETCH_PROBE", &probe)
    .output()
    .unwrap();
    assert!(
        !probe.exists(),
        "OKF invoked remote acquisition for missing local Git material"
    );
    if output.status.success() {
        let r = rows(output);
        assert!(!record(&r, "scope")["unavailable"]
            .as_array()
            .unwrap()
            .is_empty());
    } else {
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn deferred_acquisition_and_cross_bundle_artifact_reads_are_not_enabled_by_catalogs() {
    let d = catalog();
    let root = d.path();
    write(root, "b/secret.txt", "not a concept");
    let result = run(
        root,
        &[
            "artifact",
            "show",
            "../b/secret.txt",
            "a",
            "--from",
            "one",
            "--json",
        ],
    );
    assert!(
        !result.status.success(),
        "catalog unexpectedly allowed cross-bundle artifact reading"
    );
    for kind in ["git", "http"] {
        write(root,"config/catalog.yaml",&format!("catalog_version: 1\nbundles:\n  acme.a: {{location: {{type: {kind}, path: ../a}}}}\n"));
        let result = run(root, &["catalog", "--json"]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("only directory is supported"));
    }
}

#[test]
fn expansion_follows_only_one_hop_and_only_the_selected_primary_page() {
    let d = tempfile::tempdir().unwrap();
    let root = d.path();
    write(root,"ontology.yaml","okf_ontology: '0.1'\nconcepts:\n  Note:\n    references:\n      next: {target: Note, cardinality: 0..1}\n    relationships:\n      follows: {reference: next, kind: follows}\n");
    write(
        root,
        "a.md",
        "---\ntype: Note\ntitle: First\nnext: /b.md\n---\n",
    );
    write(
        root,
        "b.md",
        "---\ntype: Note\ntitle: Second\nnext: /c.md\n---\n",
    );
    write(root, "c.md", "---\ntype: Note\ntitle: Third\n---\n");
    for extra in [
        vec!["--field", "title=First"],
        vec!["--limit", "1", "--offset", "0", "--sort", "id"],
    ] {
        let mut args = vec!["search", "--expand", "follows", "--json"];
        args.extend(extra);
        let r = query(root, &args);
        assert_eq!(r.iter().filter(|v| v["kind"] == "relationship").count(), 1);
        assert_eq!(record(&r, "related-concept")["id"], "/b");
        assert!(!r
            .iter()
            .any(|v| v["kind"] == "related-concept" && v["id"] == "/c"));
    }
}

#[test]
fn scoped_lint_honors_named_bundle_rule_settings() {
    let d = catalog();
    let root = d.path();
    write(
        root,
        "a/one.md",
        "---\ntype: Unknown\n---\n[Missing](/missing)\n",
    );
    write(
        root,
        "a/ontology.yaml",
        "okf_ontology: '0.1'\nconcepts:\n  Note: {}\n",
    );
    for value in ["off", "info", "warn", "error"] {
        write(root, "okf.toml", &format!(
            "catalog='config/catalog.yaml'\ndefault_bundle={{id='acme.a'}}\n[bundle_settings.\"acme.b\".lint]\nmissing_title='off'\nmissing_description='off'\norphan='off'\nindex_coverage='off'\n[bundle_settings.\"acme.a\".lint]\nmissing_title='off'\nmissing_description='off'\norphan='off'\nindex_coverage='off'\nontology_violation='{value}'\nbroken_link='{value}'\n"
        ));
        let output = run(
            root,
            &["lint", "--json", "--catalog-scope", "--fail-on", "warn"],
        );
        assert_eq!(
            output.status.code(),
            Some(if matches!(value, "warn" | "error") {
                1
            } else {
                0
            }),
            "{value}: {output:?}"
        );
        let records: Vec<Value> = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        for rule in ["ontology-violation", "broken-link"] {
            let findings: Vec<_> = records.iter().filter(|row| row["rule"] == rule).collect();
            if value == "off" {
                assert!(findings.is_empty());
            } else {
                assert!(!findings.is_empty(), "{rule}: {records:?}");
                assert!(findings.iter().all(|row| row["severity"] == value));
            }
        }
        assert!(!records.iter().any(|row| row["rule"] == "index-coverage"));
    }
}
