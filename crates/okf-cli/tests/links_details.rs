use std::process::Command;

#[test]
fn outgoing_details_preserve_occurrences_and_plain_links() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("ontology.yaml"),
        r#"okf_ontology: '0.1'
concepts:
  Note:
    references:
      targets: {selector: 'relations[].target', target: Note, cardinality: 0..n}
    relationships:
      relations:
        reference: targets
        kind_selector: 'relations[].kind'
        kinds:
          implements: {inverse: implemented-by}
          cites: {inverse: cited-by}
        attributes:
          rationale: 'relations[].rationale'
"#,
    )
    .unwrap();
    std::fs::write(root.join("a.md"), "---\ntype: Note\nrelations:\n- {target: /b, kind: implements, rationale: required}\n- {target: /b, kind: cites, rationale: context}\n- {target: /missing, kind: cites}\n- {target: /b, kind: unrecognized}\n---\n[Plain](plain.md)\n").unwrap();
    std::fs::write(root.join("b.md"), "---\ntype: Note\n---\nB\n").unwrap();
    let run = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_okf"))
            .current_dir(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("compatibility"));
        String::from_utf8(output.stdout).unwrap()
    };
    let compact = run(&["links", "a", "--json"]);
    assert_eq!(compact.lines().count(), 3);
    let detailed = run(&["links", "a", "--details", "--json"]);
    let records: Vec<serde_json::Value> = detailed
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let edges: Vec<_> = records
        .iter()
        .filter(|r| r["kind"] == "relationship")
        .collect();
    assert_eq!(edges.len(), 4);
    assert_eq!(edges[0]["direction"], "outgoing");
    assert_eq!(edges[0]["edge"]["inverse"], "implemented-by");
    assert_eq!(edges[0]["edge"]["attributes"]["rationale"], "required");
    assert_ne!(
        edges[0]["edge"]["field_path"],
        edges[1]["edge"]["field_path"]
    );
    assert_eq!(edges[0]["edge"]["status"], "resolved");
    assert_eq!(edges[2]["edge"]["status"], "missing");
    assert_eq!(edges[3]["edge"]["status"], "unknown-kind");
    assert!(records
        .iter()
        .any(|r| r["kind"] == "link" && r["target"] == "/plain" && r["exists"] == false));
    let text = run(&["links", "a", "--details"]);
    assert!(text.contains("—implements→ /b (resolved)"));
    assert!(text.contains("inverse: implemented-by"));
    assert!(text.contains("rationale: \"required\""));
    std::fs::write(
        root.join("okf.toml"),
        "catalog = 'catalog.yaml'\ndefault_bundle = {id = 'test.notes'}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("catalog.yaml"),
        "catalog_version: 1\nbundles:\n  test.notes:\n    location: {type: directory, path: '.'}\n",
    )
    .unwrap();
    let catalog_text = run(&["links", "a", "--details"]);
    assert!(catalog_text.contains("inverse: implemented-by"));
    assert!(catalog_text.contains("rationale: \"required\""));
    let catalog_json = run(&["links", "a", "--details", "--json"]);
    let catalog_edges: Vec<serde_json::Value> = catalog_json
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(catalog_edges
        .iter()
        .any(|r| r["kind"] == "bundle-edge" && r["relationship"]["inverse"] == "implemented-by"));
    std::fs::remove_file(root.join("okf.toml")).unwrap();
    // Details remain useful without an ontology and for empty concepts.
    std::fs::remove_file(root.join("ontology.yaml")).unwrap();
    assert!(run(&["links", "a", "--details", "--json"]).contains("/plain"));
    assert_eq!(run(&["links", "b", "--details"]), "(no links)\n");
}
