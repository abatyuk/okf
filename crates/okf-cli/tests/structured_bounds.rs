use std::process::Command;
#[test]
fn expansion_bounds_stop_unrequested_target_parsing() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("ontology.yaml"),"okf_ontology: '0.1'\nconcepts:\n  Policy: {}\n  Procedure:\n    references:\n      targets: {selector: 'relations[].target', target: Policy, cardinality: 0..n}\n    relationships:\n      policies: {reference: targets, kind: implements, inverse: implemented-by}\n").unwrap();
    std::fs::write(dir.path().join("a.md"),"---\ntype: Procedure\nrelations: [{target: /z-good.md}, {target: /zz-bad.md}]\n---\nBody\n").unwrap();
    std::fs::write(
        dir.path().join("z-good.md"),
        "---\ntype: Policy\ntitle: Good policy\n---\nPolicy\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("zz-bad.md"), "---\ntype: [invalid\n---\n").unwrap();
    for (flag, value, reason) in [
        ("--expansion-bytes", "1", "byte-limit"),
        ("--expansion-targets", "1", "target-limit"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_okf"))
            .args([
                "search",
                dir.path().to_str().unwrap(),
                "--type",
                "Procedure",
                "--scan-limit",
                "1",
                "--expand",
                "policies",
                flag,
                value,
                "--json",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let records = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
            .collect::<Vec<_>>();
        let summary = records
            .iter()
            .find(|r| r["kind"] == "expansion-summary")
            .unwrap();
        assert_eq!(summary["truncated"], true);
        assert!(summary["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == reason));
        assert!(records
            .iter()
            .any(|r| r["kind"] == "warning" && r["reason"] == reason));
    }
}

fn relationship_ontology() -> &'static str {
    "okf_ontology: '0.1'\nconcepts:\n  Policy: {}\n  Procedure:\n    references:\n      targets: {selector: 'relations[].target', target: Policy, cardinality: 0..n}\n    relationships:\n      policies: {reference: targets, kind: implements, inverse: implemented-by}\n"
}

#[test]
fn expansion_resolves_bundle_root_fragments_and_query_suffixes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("ontology.yaml"), relationship_ontology()).unwrap();
    std::fs::write(dir.path().join("a.md"), "---\ntype: Procedure\nrelations: [{target: '/peer.md#section'}, {target: '/peer.md?mode=read#section'}]\n---\nBody\n").unwrap();
    std::fs::write(
        dir.path().join("peer.md"),
        "---\ntype: Policy\ntitle: Peer\n---\nPolicy\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_okf"))
        .current_dir(dir.path())
        .args([
            "search",
            "--type",
            "Procedure",
            "--expand",
            "policies",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let records = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    let edges = records
        .iter()
        .filter(|record| record["kind"] == "relationship")
        .collect::<Vec<_>>();
    assert_eq!(edges.len(), 2);
    assert!(edges
        .iter()
        .all(|record| record["edge"]["status"] == "resolved"));
    assert_eq!(
        records
            .iter()
            .filter(|record| record["kind"] == "related-concept")
            .count(),
        1
    );
}

#[test]
fn expansion_retains_registered_unavailable_target_status() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("alpha")).unwrap();
    std::fs::write(
        dir.path().join("okf.toml"),
        "catalog = 'catalog.yaml'\ndefault_bundle = {id = 'acme.alpha'}\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("catalog.yaml"), "catalog_version: 1\nbundles:\n  acme.alpha:\n    location: {type: directory, path: alpha}\n  acme.missing:\n    location: {type: directory, path: missing}\n").unwrap();
    std::fs::write(
        dir.path().join("alpha/ontology.yaml"),
        relationship_ontology(),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("alpha/a.md"),
        "---\ntype: Procedure\nrelations: [{target: '../missing/peer.md'}]\n---\nBody\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_okf"))
        .current_dir(dir.path())
        .args([
            "search",
            "--type",
            "Procedure",
            "--scope-bundle",
            "acme.missing",
            "--expand",
            "policies",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let records = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    let edge = records
        .iter()
        .find(|record| record["kind"] == "relationship")
        .unwrap();
    assert_eq!(edge["edge"]["status"], "unavailable");
    assert_eq!(edge["edge"]["raw_reference"], "../missing/peer.md");
    assert!(!records
        .iter()
        .any(|record| record["kind"] == "related-concept"));
}
