//! Executable checks for skill examples and scenario evidence, not agent-performance claims.
use serde_json::Value;
use std::fs;
use std::path::{Component, Path};
use std::process::Command;

fn run(root: &Path, args: &[&str]) -> Vec<Value> {
    let output = Command::new(env!("CARGO_BIN_EXE_okf"))
        .current_dir(root)
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if args.contains(&"--json") {
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    } else {
        Vec::new()
    }
}

fn record<'a>(rows: &'a [Value], kind: &str) -> &'a Value {
    rows.iter()
        .find(|row| row["kind"] == kind)
        .unwrap_or_else(|| panic!("missing {kind}: {rows:?}"))
}

// File examples are the fenced block immediately after a ## `relative/path` heading.
// Read the actual shipped reference, so example edits are checked against the CLI.
fn example_files(doc: &str) -> Vec<(&str, &str)> {
    doc.split("\n## `")
        .skip(1)
        .map(|section| {
            let (name, rest) = section.split_once("`\n").unwrap();
            assert!(Path::new(name)
                .components()
                .all(|part| matches!(part, Component::Normal(_))));
            let (_, fence) = rest.split_once("```").unwrap();
            let (_, contents) = fence.split_once('\n').unwrap();
            (name, contents.split_once("\n```").unwrap().0)
        })
        .collect()
}

fn materialize(root: &Path, doc: &str) {
    let files = example_files(doc);
    assert!(!files.is_empty());
    for (name, contents) in files {
        let path = root.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("{contents}\n")).unwrap();
    }
}

// These example command blocks deliberately contain plain arguments with no shell operators.
fn example_commands(doc: &str) -> Vec<Vec<&str>> {
    doc.split("```sh\n")
        .skip(1)
        .flat_map(|block| block.split_once("```").unwrap().0.lines())
        .map(|line| {
            let words: Vec<_> = line.split_whitespace().collect();
            assert_eq!(words.first(), Some(&"okf"));
            assert!(!line.contains(['\'', '"', ';', '|', '&']));
            words[1..].to_vec()
        })
        .collect()
}

fn incomplete_inventory() {
    let root = tempfile::tempdir().unwrap();
    for n in 0..1001 {
        let tags = if n == 1000 { "[exception]" } else { "[]" };
        fs::write(
            root.path().join(format!("{n:04}.md")),
            format!("---\ntype: Note\ntags: {tags}\n---\nNote {n}.\n"),
        )
        .unwrap();
    }
    let query = [
        "search",
        ".",
        "--facet-filter",
        "tags[]=\"exception\"",
        "--limit",
        "1",
        "--offset",
        "0",
        "--json",
    ];
    let partial = run(root.path(), &query);
    let summary = record(&partial, "query-summary");
    assert_eq!(summary["scan_complete"], false);
    assert_eq!(summary["examined_documents"], 1000);
    assert_eq!(summary["observed_matches"], 0);
    assert!(summary["total_matches"].is_null());
    assert!(summary["next_offset"].is_null());
    record(&partial, "warning");

    for recovery in [vec!["--scan-limit", "1001"], vec!["--full-scan"]] {
        let mut args = query.to_vec();
        args.extend(recovery);
        let complete = run(root.path(), &args);
        let summary = record(&complete, "query-summary");
        assert_eq!(summary["scan_complete"], true);
        assert_eq!(summary["total_matches"], 1);
        assert!(complete.iter().any(|row| row["id"] == "/1000"));
    }
    let inventory = run(root.path(), &["list", ".", "--full-scan", "--json"]);
    assert_eq!(record(&inventory, "query-summary")["total_matches"], 1001);
    assert_eq!(
        inventory
            .iter()
            .filter(|row| row.get("trust_tier").is_some())
            .count(),
        1001
    );
}

fn cross_record_conditions(doc: &str) {
    let root = tempfile::tempdir().unwrap();
    materialize(root.path(), doc);
    let rows = run(
        root.path(),
        &[
            "search",
            ".",
            "--facet-filter",
            "norms[].bearer=\"/teams/platform.md\"",
            "--facet-filter",
            "norms[].deadline.within=24",
            "--project",
            "norms",
            "--expand",
            "obligations",
            "--full-scan",
            "--json",
        ],
    );
    assert_eq!(record(&rows, "query-summary")["total_matches"], 1);
    let projection = record(&rows, "projection");
    let norms = &projection["fields"][0]["occurrences"][0]["value"];
    assert_eq!(norms[0]["bearer"], "/teams/platform.md");
    assert_eq!(norms[0]["deadline"]["within"], 72);
    assert_eq!(norms[1]["deadline"]["within"], 24);
    let platform = rows
        .iter()
        .find(|row| row["kind"] == "relationship" && row["edge"]["target"] == "/teams/platform")
        .unwrap();
    assert_eq!(platform["edge"]["field_path"], "norms[0].bearer");
    assert_eq!(platform["edge"]["attributes"]["deadline"]["within"], 72);
}

fn nested_authoring(doc: &str) {
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("knowledge/product");
    materialize(&bundle, doc);
    // Merge the documented model into a sidecar with an unrelated producer extension.
    let ontology = bundle.join("ontology.yaml");
    let model = fs::read_to_string(&ontology).unwrap();
    fs::write(
        &ontology,
        "# Preserve this producer extension.\nx-owner: keep-me\nokf_ontology: '0.1'\nconcepts: {Team: {}, Procedure: {}}\n",
    )
    .unwrap();
    let mut changes: serde_yaml::Value = serde_yaml::from_str(&model).unwrap();
    changes
        .as_mapping_mut()
        .unwrap()
        .remove(serde_yaml::Value::String("okf_ontology".into()));
    fs::write(
        root.path().join("changes.yaml"),
        serde_yaml::to_string(&changes).unwrap(),
    )
    .unwrap();
    let baseline = fs::read(&ontology).unwrap();
    run(
        root.path(),
        &[
            "ontology",
            "apply",
            "knowledge/product",
            "--from",
            "changes.yaml",
            "--dry-run",
        ],
    );
    assert_eq!(fs::read(&ontology).unwrap(), baseline);
    run(
        root.path(),
        &[
            "ontology",
            "apply",
            "knowledge/product",
            "--from",
            "changes.yaml",
        ],
    );
    let before: Vec<_> = example_files(doc)
        .into_iter()
        .filter(|(name, _)| name.ends_with(".md"))
        .map(|(name, _)| (name, fs::read(bundle.join(name)).unwrap()))
        .collect();
    let checks = doc.split_once("## Check the modeled behavior").unwrap().1;
    let outputs: Vec<_> = example_commands(checks)
        .into_iter()
        .map(|args| run(root.path(), &args))
        .collect();
    let effective = record(&outputs[0], "ontology_type");
    let fields = &effective["fields"][0]["effective"]["ty"]["item"]["fields"];
    assert_eq!(fields["bearer"]["required"], true);
    assert_eq!(
        fields["deadline"]["ty"]["fields"]["within"]["ty"]["constraints"]["min"],
        1
    );
    assert!(
        !outputs[2]
            .iter()
            .any(|row| row["rule"] == "ontology-violation"),
        "{:?}",
        outputs[2]
    );
    let incoming = outputs
        .last()
        .unwrap()
        .iter()
        .find(|row| row["kind"] == "relationship")
        .expect("semantic inverse");
    // Detailed backlinks may wrap the occurrence in edge; the assertion preserves its conditions.
    let edge = incoming.get("edge").unwrap_or(incoming);
    assert_eq!(edge["inverse"], "obligated-by");
    assert_eq!(edge["attributes"]["deadline"]["within"], 72);
    assert!(fs::read_to_string(&ontology)
        .unwrap()
        .contains("x-owner: keep-me"));
    for (name, bytes) in before {
        assert_eq!(fs::read(bundle.join(name)).unwrap(), bytes);
    }
}

fn scoped_cross_bundle(doc: &str) {
    let root = tempfile::tempdir().unwrap();
    let commands = example_commands(doc);
    for args in commands.iter().take(2) {
        run(root.path(), args);
    }
    materialize(root.path(), doc);
    let catalog = root.path().join("okf-catalog.yaml");
    let mut content = fs::read_to_string(&catalog).unwrap();
    content.push_str("  acme.unrelated:\n    location: {type: directory, path: unavailable}\n");
    fs::write(catalog, content).unwrap();
    for args in commands.iter().skip(2) {
        run(root.path(), args);
    }
    let unscoped = run(
        root.path(),
        &[
            "links",
            "metrics/revenue",
            "--bundle-id",
            "acme.product",
            "--json",
        ],
    );
    assert_eq!(record(&unscoped, "bundle-edge")["status"], "out-of-scope");
    let scoped = run(
        root.path(),
        &[
            "links",
            "metrics/revenue",
            "--bundle-id",
            "acme.product",
            "--scope-bundle",
            "acme.finance",
            "--json",
        ],
    );
    let source = scoped
        .iter()
        .find(|row| row["location"] == "sources[0].resource")
        .unwrap();
    assert_eq!(source["status"], "resolved");
    assert_eq!(source["evidence"], "same-local-file");
    assert_eq!(source["target"]["bundle"], "acme.finance");
    assert_eq!(source["target"]["id"], "/policies/margin-standard");
    let scope = record(&scoped, "scope");
    assert_eq!(scope["examined"].as_array().unwrap().len(), 2);
    assert!(scope["requested"]
        .as_array()
        .unwrap()
        .iter()
        .all(|id| id != "acme.unrelated"));
    assert!(scope["unavailable"].as_array().unwrap().is_empty());
}

fn expansion_and_pagination_cost(doc: &str) {
    let root = tempfile::tempdir().unwrap();
    materialize(root.path(), doc);
    let base = [
        "search",
        ".",
        "--type",
        "Procedure",
        "--expand",
        "obligations",
        "--target-field",
        "title",
        "--full-scan",
        "--limit",
        "1",
        "--offset",
        "0",
        "--json",
    ];
    let mut bounded = base.to_vec();
    bounded.extend(["--expansion-targets", "1"]);
    let rows = run(root.path(), &bounded);
    assert_eq!(record(&rows, "query-summary")["scan_complete"], true);
    assert_eq!(record(&rows, "expansion-summary")["truncated"], true);
    assert!(rows
        .iter()
        .any(|row| row["kind"] == "warning" && row["reason"] == "target-limit"));
    let complete = run(root.path(), &base);
    assert_eq!(record(&complete, "expansion-summary")["truncated"], false);
    assert_eq!(
        complete
            .iter()
            .filter(|row| row["kind"] == "related-concept")
            .count(),
        2
    );
    for offset in ["0", "1"] {
        let page = run(
            root.path(),
            &[
                "list",
                ".",
                "--full-scan",
                "--limit",
                "1",
                "--offset",
                offset,
                "--json",
            ],
        );
        let summary = record(&page, "query-summary");
        assert_eq!(summary["returned"], 1);
        assert_eq!(summary["examined_documents"], 3);
        assert_eq!(summary["total_matches"], 3);
    }
}

fn migrate_example_document(root: &Path, bundle: &str, source_name: &str) {
    let original = fs::read_to_string(root.join(format!("legacy/{source_name}.md"))).unwrap();
    let (_, rest) = original.split_once("---\n").unwrap();
    let (metadata, body) = rest.split_once("---\n").unwrap();
    let metadata: serde_yaml::Value = serde_yaml::from_str(metadata).unwrap();
    let body_file = root.join("migration-body.md");
    fs::write(
        &body_file,
        body.replace(
            "(finance.md#review)",
            "(../../finance/notes/shared.md#review)",
        ),
    )
    .unwrap();
    let body_arg = format!("@{}", body_file.display());
    run(
        root,
        &[
            "add",
            "notes/shared",
            "--bundle-id",
            bundle,
            "--type",
            metadata["type"].as_str().unwrap(),
            "--title",
            metadata["title"].as_str().unwrap(),
            "--body",
            &body_arg,
        ],
    );
    let destination = root.join(format!("knowledge/{source_name}/notes/shared.md"));
    if source_name == "product" {
        // Narrow insertion into a newly created document: retain the exact authored extension
        // bytes, and rewrite only the declared target fields and the renamed list key.
        let custom = original
            .split_once("requirements:\n")
            .unwrap()
            .1
            .split_once("---\n")
            .unwrap()
            .0;
        let custom = format!(
            "relations:\n{}",
            custom.replace(
                "target: finance.md",
                "target: ../../finance/notes/shared.md"
            )
        );
        let created = fs::read_to_string(&destination).unwrap();
        let (front, body) = created
            .strip_prefix("---\n")
            .unwrap()
            .split_once("---\n")
            .unwrap();
        fs::write(&destination, format!("---\n{front}{custom}---\n{body}")).unwrap();
    }
    let source = serde_json::json!({"id":"original", "resource":format!("../../../legacy/{source_name}.md")});
    let source_file = root.join("original-source.json");
    fs::write(&source_file, source.to_string()).unwrap();
    let source_arg = format!("@{}", source_file.display());
    run(
        root,
        &[
            "edit",
            "notes/shared",
            "--bundle-id",
            bundle,
            "--add-source-json",
            &source_arg,
        ],
    );
}

fn resumable_structured_migration(doc: &str) {
    let root = tempfile::tempdir().unwrap();
    materialize(root.path(), doc);
    let product = root.path().join("knowledge/product/notes/shared.md");
    let finance = root.path().join("knowledge/finance/notes/shared.md");
    let sources: Vec<_> = ["product", "finance"]
        .into_iter()
        .map(|name| {
            (
                name,
                fs::read(root.path().join(format!("legacy/{name}.md"))).unwrap(),
            )
        })
        .collect();
    let mut ledger = serde_json::json!([
        {"source":"legacy/product.md", "bundle":"acme.product", "id":"notes/shared", "state":"planned"},
        {"source":"legacy/finance.md", "bundle":"acme.finance", "id":"notes/shared", "state":"planned"}
    ]);
    migrate_example_document(root.path(), "acme.product", "product");
    ledger[0]["state"] = "content-written".into();
    ledger[0]["result"] = fs::read_to_string(&product).unwrap().into();
    ledger[0]["source_bytes"] = fs::read_to_string(root.path().join("legacy/product.md"))
        .unwrap()
        .into();
    let ledger_file = root.path().join("migration-ledger.json");
    fs::write(&ledger_file, ledger.to_string()).unwrap();

    // A failed second creation must not leave a skeleton. Resume using the persisted ledger.
    let failed = Command::new(env!("CARGO_BIN_EXE_okf"))
        .current_dir(root.path())
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args([
            "add",
            "notes/shared",
            "--bundle-id",
            "acme.finance",
            "--type",
            "Policy",
            "--body",
            "@missing-body.md",
        ])
        .output()
        .unwrap();
    assert!(!failed.status.success());
    assert!(!finance.exists());
    let mut resumed: Value =
        serde_json::from_str(&fs::read_to_string(&ledger_file).unwrap()).unwrap();
    assert_eq!(
        fs::read_to_string(&product).unwrap(),
        resumed[0]["result"].as_str().unwrap()
    );
    assert_eq!(
        fs::read_to_string(root.path().join("legacy/product.md")).unwrap(),
        resumed[0]["source_bytes"].as_str().unwrap()
    );
    let completed_bytes = fs::read(&product).unwrap();
    migrate_example_document(root.path(), "acme.finance", "finance");
    resumed[1]["state"] = "content-written".into();
    assert_eq!(fs::read(&product).unwrap(), completed_bytes);

    let outputs: Vec<_> = example_commands(doc)
        .into_iter()
        .map(|args| run(root.path(), &args))
        .collect();
    assert!(
        !outputs[2]
            .iter()
            .any(|row| row["severity"] == "error" || row["rule"] == "ontology-violation"),
        "{:?}",
        outputs[2]
    );
    assert_eq!(record(&outputs[3], "query-summary")["total_matches"], 2);
    let identities: Vec<_> = outputs[3]
        .iter()
        .filter(|row| row["kind"] == "concept-identity")
        .collect();
    assert_eq!(identities.len(), 2);
    assert_ne!(identities[0]["bundle"], identities[1]["bundle"]);
    assert!(identities.iter().all(|row| row["id"] == "/notes/shared"));
    let edges: Vec<_> = outputs[6]
        .iter()
        .filter(|row| row["kind"] == "relationship")
        .collect();
    assert_eq!(edges.len(), 2);
    assert_eq!(edges[0]["edge"]["attributes"]["deadline"], 24);
    assert_eq!(edges[1]["edge"]["attributes"]["deadline"], 72);
    assert!(edges.iter().all(|row| row["edge"]["status"] == "resolved"));
    assert!(outputs[4].iter().any(
        |row| row["resource"] == "../../finance/notes/shared.md#review"
            && row["target"]["bundle"] == "acme.finance"
    ));
    let incoming: Vec<_> = outputs[5]
        .iter()
        .filter(|row| row["relationship"].is_object())
        .collect();
    assert_eq!(incoming.len(), 2);
    for name in ["product", "finance"] {
        let rows = run(
            root.path(),
            &[
                "show",
                "notes/shared",
                "--bundle-id",
                &format!("acme.{name}"),
                "--json",
            ],
        );
        let row = &rows[0];
        assert_eq!(row["sources"].as_array().unwrap().len(), 1);
        assert_eq!(
            row["sources"][0]["resource"],
            format!("../../../legacy/{name}.md")
        );
        if name == "product" {
            assert_eq!(row["x-import"]["enabled"], false);
            assert_eq!(row["x-import"]["code"], "001");
            assert!(row["x-import"]["nullable"].is_null());
            assert_eq!(row["x-import"]["empty"], serde_json::json!([]));
            assert_eq!(row["x-import"]["attachment"], "finance.md");
            assert!(row.get("requirements").is_none());
        }
    }
    for (name, bytes) in sources {
        assert_eq!(
            fs::read(root.path().join(format!("legacy/{name}.md"))).unwrap(),
            bytes
        );
    }
    for row in resumed.as_array_mut().unwrap() {
        row["state"] = "accepted".into();
    }
    fs::write(&ledger_file, resumed.to_string()).unwrap();
    // A completed rerun inspects destinations; it does not append sources or replay writes.
    assert_eq!(fs::read(&product).unwrap(), completed_bytes);
}

fn structured_mutation() {
    let root = tempfile::tempdir().unwrap();
    run(
        root.path(),
        &[
            "add",
            "procedures/onboarding",
            ".",
            "--type",
            "Procedure",
            "--body",
            "Keep this body.",
            "--set-yaml",
            "norms=[{bearer: /teams/platform.md, deadline: {within: 72}}]",
            "--set-yaml",
            "opaque={preserve: false}",
        ],
    );
    let concept = root.path().join("procedures/onboarding.md");
    let baseline = fs::read(&concept).unwrap();
    run(
        root.path(),
        &[
            "edit",
            "procedures/onboarding",
            ".",
            "--set-path",
            "deadline.within=48",
            "--dry-run",
        ],
    );
    assert_eq!(fs::read(&concept).unwrap(), baseline);
    run(
        root.path(),
        &[
            "edit",
            "procedures/onboarding",
            ".",
            "--set-path",
            "deadline.within=48",
        ],
    );
    fs::write(
        root.path().join("patch.yaml"),
        r#"
- op: test
  path: /norms/0/bearer
  value: /teams/platform.md
- op: replace
  path: /norms/0/deadline/within
  value: 48
- op: add
  path: /norms/-
  value: {bearer: /teams/security.md, deadline: {within: 24}}
"#,
    )
    .unwrap();
    run(
        root.path(),
        &[
            "edit",
            "procedures/onboarding",
            ".",
            "--patch",
            "@patch.yaml",
        ],
    );
    let content = fs::read_to_string(&concept).unwrap();
    let metadata: serde_yaml::Value =
        serde_yaml::from_str(content.split("---").nth(1).unwrap()).unwrap();
    assert_eq!(metadata["deadline"]["within"].as_i64(), Some(48));
    assert_eq!(
        metadata["norms"][0]["deadline"]["within"].as_i64(),
        Some(48)
    );
    assert_eq!(metadata["norms"].as_sequence().unwrap().len(), 2);
    assert_eq!(metadata["opaque"]["preserve"].as_bool(), Some(false));
    assert!(content.contains("Keep this body."));
    let failed = Command::new(env!("CARGO_BIN_EXE_okf"))
        .current_dir(root.path()).env_remove("OKF_BUNDLE").env_remove("OKF_CATALOG_OVERRIDES")
        .args(["edit", "procedures/onboarding", ".", "--patch",
            "[{op: replace, path: /deadline/within, value: 1}, {op: test, path: /deadline/within, value: 999}]"])
        .output().unwrap();
    assert!(!failed.status.success());
    assert_eq!(fs::read_to_string(&concept).unwrap(), content);
}

#[test]
fn skill_behavior_scenarios() {
    let scenarios: Value =
        serde_json::from_str(include_str!("../../../xtask/skill-behaviors.json")).unwrap();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut seen = std::collections::HashSet::new();
    for case in scenarios.as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        assert!(seen.insert(id), "duplicate scenario {id}");
        assert!(!case["prompt"].as_str().unwrap().is_empty());
        assert!(!case["setup"].as_str().unwrap().is_empty());
        for key in ["skills", "pass", "fail"] {
            assert!(!case[key].as_array().unwrap().is_empty(), "{id}: {key}");
        }
        for skill in case["skills"].as_array().unwrap() {
            assert!(repo
                .join("plugins/okf/skills")
                .join(skill.as_str().unwrap())
                .join("SKILL.md")
                .is_file());
        }
        let doc = case["fixture"]
            .as_str()
            .map(|path| fs::read_to_string(repo.join(path)).unwrap())
            .unwrap_or_default();
        match id {
            "incomplete-inventory" => incomplete_inventory(),
            "cross-record-conditions" => cross_record_conditions(&doc),
            "nested-authoring" => nested_authoring(&doc),
            "structured-mutation" => structured_mutation(),
            "scoped-cross-bundle" => scoped_cross_bundle(&doc),
            "expansion-and-pagination-cost" => expansion_and_pagination_cost(&doc),
            "resumable-structured-migration" => resumable_structured_migration(&doc),
            _ => panic!("scenario {id} has no executable workflow"),
        }
        println!("skill workflow passed: {id}");
    }
    assert_eq!(seen.len(), 7);
}
