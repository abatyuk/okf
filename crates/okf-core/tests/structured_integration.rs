use okf_core::{
    bundle::loader::load_bundle,
    check::lint::rules::index_coverage::check_indexes,
    graph::{build::build_graph, relationships::relationships},
    ontology::load::parse_ontology,
    query::metadata::{facet, Condition},
};
fn document(dir: &std::path::Path, path: &str, meta: &str) {
    let p = dir.join(path);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, format!("---\n{meta}\n---\nBody\n")).unwrap();
}
fn ontology() -> okf_core::ontology::schema::Ontology {
    parse_ontology(
        r#"
okf_ontology: '0.1'
concepts:
  Policy: {}
  Procedure:
    fields:
      relations:
        type: list
        item: Relation
    references:
      policies: {selector: 'relations[].target', target: Policy, cardinality: 0..n}
    relationships:
      policy-relations:
        reference: policies
        kind_selector: 'relations[].kind'
        kinds:
          implements: {inverse: implemented-by}
          supersedes: {inverse: superseded-by}
        attributes:
          qualifier: 'relations[].qualifier'
field_types:
  Relation:
    base: object
    fields:
      target: {type: string}
      kind: {type: enum, values: [implements, supersedes]}
      qualifier: {type: string}
"#,
    )
    .unwrap()
}
#[test]
fn occurrence_pairing_self_edges_and_neighbors() {
    let dir = tempfile::tempdir().unwrap();
    document(dir.path(), "p.md", "type: Policy");
    document(dir.path(),"a.md","type: Procedure\nrelations:\n- {target: /p.md, kind: implements, qualifier: first}\n- {target: /p.md, kind: supersedes, qualifier: second}\n- {target: /a.md, kind: implements}");
    let bundle = load_bundle(dir.path()).unwrap();
    let ont = ontology();
    let edges = relationships(bundle.get("/a").unwrap(), &ont);
    assert_eq!(edges.len(), 3);
    assert_eq!(edges[0].field_path, "relations[0].target");
    assert_eq!(edges[0].inverse.as_deref(), Some("implemented-by"));
    assert_eq!(edges[1].attributes["qualifier"].as_str(), Some("second"));
    assert_eq!(edges[2].target, "/a");
    let g = build_graph(&bundle, Some(&ont));
    assert_eq!(g.outbound("/a").len(), 1);
    assert_eq!(g.inbound("/p")[0].0, "/a");
}
#[test]
fn membership_is_whole_list_and_missing_differs_from_empty() {
    let dir = tempfile::tempdir().unwrap();
    document(dir.path(),"a.md","type: Procedure\ntags: [security, draft, security]\nnorms: [{bearer: x}, {bearer: y}]\n'policy.status': active");
    document(dir.path(), "b.md", "type: Procedure\ntags: []\nnorms: []");
    document(dir.path(), "c.md", "type: Procedure\ntags: null");
    let b = load_bundle(dir.path()).unwrap();
    let not = Condition::parse("tags[] not in [\"draft\"]").unwrap();
    assert!(!not.matches(b.get("/a").unwrap()).unwrap());
    assert!(not.matches(b.get("/b").unwrap()).unwrap());
    assert!(!not.matches(b.get("/c").unwrap()).unwrap());
    assert!(!Condition::parse("norms[].bearer not in [\"x\"]")
        .unwrap()
        .matches(b.get("/b").unwrap())
        .unwrap());
    assert!(Condition::parse("[\"policy.status\"]=\"active\"")
        .unwrap()
        .matches(b.get("/a").unwrap())
        .unwrap());
    for raw in [
        "tags[] in []",
        "tags[] in [null]",
        "tags[]=true extra",
        "tags[]={}",
    ] {
        assert!(Condition::parse(raw).is_err());
    }
    let f = facet(
        &b.concepts.iter().collect::<Vec<_>>(),
        "tags[]",
        100,
        false,
        true,
    )
    .unwrap();
    assert_eq!(f["values"].as_array().unwrap().len(), 2);
    assert!(f["values"]
        .as_array()
        .unwrap()
        .iter()
        .all(|v| v["count"] == 1));
}
#[test]
fn indexes_require_immediate_markdown_children_and_exclude_subtrees() {
    let d = tempfile::tempdir().unwrap();
    document(d.path(), "policies/a.md", "type: Policy");
    document(d.path(), "policies/b.md", "type: Policy");
    document(d.path(), "policies/nested/deep.md", "type: Policy");
    document(d.path(), "drafts/private.md", "type: Policy");
    std::fs::write(
        d.path().join("index.md"),
        "[Policies](policies/index.md)\n[Drafts](drafts/)",
    )
    .unwrap();
    std::fs::write(d.path().join("policies/index.md"),"[A](a.md#section)\n[deep](nested/deep.md)\n b.md\n![image](b.md)\n```\n[B](b.md)\n```\n[malformed](%😀)\n").unwrap();
    let b = load_bundle(d.path()).unwrap();
    let f = check_indexes(&b, &["drafts/**".into()]).unwrap();
    assert!(f
        .iter()
        .any(|f| f.directory == "policies" && f.missing_child == "b.md"));
    assert!(f
        .iter()
        .any(|f| f.directory == "policies" && f.missing_child == "nested/"));
    assert!(f
        .iter()
        .any(|f| f.code == "index-missing" && f.directory == "policies/nested"));
    assert!(!f
        .iter()
        .any(|f| f.directory.starts_with("drafts") || f.missing_child == "drafts/"));
    std::fs::write(
        d.path().join("policies/index.md"),
        "[A](a)\n[B][b]\n[nested](nested/index.md#x)\n\n[b]: b.md",
    )
    .unwrap();
    assert!(!check_indexes(&b, &["drafts/**".into()])
        .unwrap()
        .iter()
        .any(|f| f.directory == "policies"));
}
#[test]
fn finding_budget_covers_required_references_and_relationships() {
    let ont = ontology();
    let fm=okf_core::parse::yaml::parse_frontmatter("type: Procedure\nrelations:\n- {target: /missing, kind: unknown}\n- {target: /another, kind: missing}\n").unwrap();
    let out = okf_core::ontology::field_types::check_concept_with_budget(
        &ont,
        &okf_core::model::frontmatter::Frontmatter::from_map(fm.clone()),
        |_| None,
        1,
    );
    assert_eq!(out.len(), 2);
    assert!(
        matches!(&out[1].kind,okf_core::ontology::field_types::ViolationKind::Metadata{code,..} if code=="incomplete-check")
    );
    let bad="okf_ontology: '0.1'\nfield_types:\n  Parent: {base: string}\n  Child: {extends: Parent, base: Typo}\n";
    assert!(parse_ontology(bad).is_err());
}
#[test]
fn uri_authorities_percent_escapes_and_components_are_checked() {
    let ont = parse_ontology(
        "okf_ontology: '0.1'\nconcepts:\n  Note:\n    fields:\n      link: {type: uri}\n",
    )
    .unwrap();
    for (link, valid) in [
        ("https://example.com:443/a%20b?q=x#part", true),
        ("https://[2001:db8::1]/", true),
        ("urn:isbn:9780141036144", true),
        ("mailto:user@example.com", true),
        ("file:///a", true),
        ("https://[broken", false),
        ("https://host:badport", false),
        ("https://host/[%20]", false),
        ("https://host/%😀", false),
        ("https://host/%GG", false),
        ("relative/path", false),
        ("https://h/a#b#c", false),
    ] {
        let yaml = format!(
            "type: Note\nlink: {}\n",
            serde_json::to_string(link).unwrap()
        );
        let fm = okf_core::parse::yaml::parse_frontmatter(&yaml).unwrap();
        assert_eq!(
            okf_core::ontology::field_types::check_concept(
                &ont,
                &okf_core::model::frontmatter::Frontmatter::from_map(fm.clone()),
                |_| None
            )
            .is_empty(),
            valid,
            "{link}"
        );
    }
}
#[test]
fn concept_subtypes_narrow_relationship_targets() {
    let schema = r#"
okf_ontology: '0.1'
concepts:
  Policy: {}
  SecurePolicy: {extends: Policy}
  Procedure:
    fields:
      kind: {type: enum, values: [implements]}
    references:
      policies: {target: Policy, cardinality: 0..1}
    relationships:
      implementation:
        reference: policies
        kind_selector: kind
        kinds:
          implements: {inverse: implemented-by, target: SecurePolicy}
"#;
    let ont = parse_ontology(schema).unwrap();
    assert!(okf_core::ontology::field_types::concept_is_a(
        &ont,
        "SecurePolicy",
        "Policy"
    ));
    let fm = okf_core::parse::yaml::parse_frontmatter(
        "type: Procedure\npolicies: /target\nkind: implements",
    )
    .unwrap();
    assert!(okf_core::ontology::field_types::check_concept(
        &ont,
        &okf_core::model::frontmatter::Frontmatter::from_map(fm.clone()),
        |_| Some("SecurePolicy".into())
    )
    .is_empty());
    assert!(okf_core::ontology::field_types::check_concept(&ont,&okf_core::model::frontmatter::Frontmatter::from_map(fm.clone()),|_|Some("Policy".into())).iter().any(|v|matches!(&v.kind,okf_core::ontology::field_types::ViolationKind::Metadata{code,..} if code=="metadata-reference-target")));
}
