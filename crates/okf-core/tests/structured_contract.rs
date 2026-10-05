use okf_core::model::frontmatter::Frontmatter;
use okf_core::ontology::field_types::{check_concept, resolve_type_name, ViolationKind};
use okf_core::ontology::load::parse_ontology;

fn codes(ontology: &str, metadata: &str) -> Vec<(String, String)> {
    let ontology = parse_ontology(ontology).unwrap();
    let metadata = Frontmatter::from_map(serde_yaml::from_str(metadata).unwrap());
    check_concept(&ontology, &metadata, |_| None)
        .into_iter()
        .filter_map(|finding| match finding.kind {
            ViolationKind::Metadata { code, path } => Some((code, path)),
            _ => None,
        })
        .collect()
}

#[test]
fn inherited_field_map_replaces_and_closure_checks_replacement() {
    let schema = r#"
okf_ontology: '0.1'
field_types:
  Parent:
    base: object
    fields:
      mandatory: {type: string, required: true}
  Child:
    extends: Parent
    additional_properties: false
    fields:
      note: {type: string}
concepts:
  Note:
    fields:
      record: {type: Child}
"#;
    let ontology = parse_ontology(schema).unwrap();
    let effective = resolve_type_name(&ontology, "Child", &mut Vec::new()).unwrap();
    assert!(!effective.fields.contains_key("mandatory"));
    assert!(codes(schema, "type: Note\nrecord: {note: ok}\n").is_empty());
    assert_eq!(
        codes(schema, "type: Note\nrecord: {mandatory: x}\n"),
        vec![(
            "metadata-unknown-property".into(),
            "record.mandatory".into()
        )]
    );
}

#[test]
fn null_required_empty_and_container_failures_are_distinct() {
    let schema = r#"
okf_ontology: '0.1'
concepts:
  Note:
    fields:
      optional: {type: string}
      parent:
        type: object
        fields:
          child: {type: string, required: true}
      required: {type: string, required: true}
      items: {type: list, item: string, required: true}
"#;
    let findings = codes(
        schema,
        "type: Note\noptional: null\nparent: 5\nrequired: '  '\nitems: []\n",
    );
    assert_eq!(findings.len(), 4, "{findings:?}");
    assert!(findings.contains(&("metadata-type".into(), "optional".into())));
    assert!(findings.contains(&("metadata-type".into(), "parent".into())));
    assert!(findings.contains(&("metadata-required".into(), "required".into())));
    assert!(findings.contains(&("metadata-required".into(), "items".into())));
    let missing_parent = codes(schema, "type: Note\nrequired: yes\nitems: [x]\n");
    assert!(missing_parent.is_empty(), "{missing_parent:?}");
}

#[test]
fn named_composition_cycles_and_inherited_conflicts_are_configuration_errors() {
    for schema in [
        "field_types:\n  A: {base: object, fields: {child: {type: B}}}\n  B: {base: list, item: A}\nconcepts: {}\n",
        "field_types:\n  Parent: {base: int, min: 5}\n  Child: {extends: Parent, max: 3}\nconcepts: {}\n",
        "field_types:\n  Parent: {base: string}\n  Child: {extends: Parent, base: int}\nconcepts: {}\n",
        "concepts:\n  Note:\n    fields:\n      data: {type: list}\n",
        "concepts:\n  Note:\n    fields:\n      data: {type: bool, min: 1}\n",
    ] {
        assert!(parse_ontology(&format!("okf_ontology: '0.1'\n{schema}")).is_err(), "{schema}");
    }
}

#[test]
fn recursive_checks_keep_scalar_types_unicode_lengths_and_structural_uniqueness() {
    let schema = r#"
okf_ontology: '0.1'
concepts:
  Note:
    fields:
      value: {type: string, min: 2, max: 2, pattern: '^..$'}
      date: {type: date}
      timestamp: {type: datetime}
      number: {type: int}
      rows:
        type: list
        item: object
        unique_items: true
"#;
    assert!(codes(schema, "type: Note\nvalue: 'é😀'\ndate: '2000-02-29'\ntimestamp: '2026-10-04T12:30:00-07:00'\nnumber: 42\nrows: [{a: 1, b: 2}]\n").is_empty());
    let failures = codes(schema, "type: Note\nvalue: 'é'\ndate: '1900-02-29'\ntimestamp: '2026-10-04T12:30:00'\nnumber: 1.5\nrows: [{a: 1, b: 2}, {b: 2, a: 1}]\n");
    assert!(failures.contains(&("metadata-range".into(), "value".into())));
    assert!(failures.contains(&("metadata-type".into(), "date".into())));
    assert!(failures.contains(&("metadata-type".into(), "timestamp".into())));
    assert!(failures.contains(&("metadata-type".into(), "number".into())));
    assert!(failures.contains(&("metadata-duplicate".into(), "rows[1]".into())));
}

#[test]
fn missing_nested_branches_do_not_become_empty_selected_lists() {
    use okf_core::model::concept::{Concept, ConceptId};
    use okf_core::query::metadata::Condition;
    let concept = Concept {
        id: ConceptId::from_relative("note"),
        frontmatter: Frontmatter::from_map(
            serde_yaml::from_str("type: Note\nnorms: []\ntags: []\n").unwrap(),
        ),
        body: String::new(),
    };
    assert!(Condition::parse("tags[] not in [\"draft\"]")
        .unwrap()
        .matches(&concept)
        .unwrap());
    assert!(
        !Condition::parse("norms[].deadline.from not in [\"account-created\"]")
            .unwrap()
            .matches(&concept)
            .unwrap()
    );
    assert!(!Condition::parse("missing[] not in [\"draft\"]")
        .unwrap()
        .matches(&concept)
        .unwrap());
}

#[test]
fn facet_numeric_ties_use_numeric_order_and_keep_scalar_types_distinct() {
    use okf_core::model::concept::{Concept, ConceptId};
    use okf_core::query::metadata::facet;
    let concepts: Vec<_> = ["10", "2", "'2'"]
        .into_iter()
        .enumerate()
        .map(|(i, value)| Concept {
            id: ConceptId::from_relative(&format!("note{i}")),
            frontmatter: Frontmatter::from_map(
                serde_yaml::from_str(&format!("type: Note\nvalue: {value}\n")).unwrap(),
            ),
            body: String::new(),
        })
        .collect();
    let refs: Vec<_> = concepts.iter().collect();
    let record = facet(&refs, "value", 100, false, true).unwrap();
    let values = record["values"].as_array().unwrap();
    assert_eq!(values[0]["value"], 2);
    assert_eq!(values[1]["value"], 10);
    assert_eq!(values[2]["value"], "2");
}

#[cfg(unix)]
#[test]
fn index_coverage_does_not_follow_symlink_indexes() {
    use okf_core::bundle::loader::load_bundle;
    use okf_core::check::lint::rules::index_coverage::check_indexes;
    let root = tempfile::tempdir().unwrap();
    let bundle_root = root.path().join("bundle");
    std::fs::create_dir(&bundle_root).unwrap();
    std::fs::write(bundle_root.join("one.md"), "---\ntype: Note\n---\n").unwrap();
    std::fs::write(root.path().join("external-index.md"), "[One](one.md)\n").unwrap();
    std::os::unix::fs::symlink("../external-index.md", bundle_root.join("index.md")).unwrap();
    let bundle = load_bundle(&bundle_root).unwrap();
    let findings = check_indexes(&bundle, &[]).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "index-missing");
}
