//! Additional boundary scenarios for the implemented structured-metadata capability.
use okf_core::{
    model::frontmatter::Frontmatter,
    ontology::{
        field_types::{check_concept, ViolationKind},
        load::parse_ontology,
    },
    query::selector::Selector,
};

fn findings(schema: &str, metadata: &str) -> Vec<(String, String)> {
    let o = parse_ontology(schema).unwrap();
    let f = Frontmatter::from_map(serde_yaml::from_str(metadata).unwrap());
    check_concept(&o, &f, |_| None)
        .into_iter()
        .filter_map(|v| match v.kind {
            ViolationKind::Metadata { code, path } => Some((code, path)),
            _ => None,
        })
        .collect()
}
#[test]
fn every_primitive_and_constraint_has_positive_and_negative_boundaries() {
    let cases = [
        ("{type: string}", "'123'", "123", "metadata-type"),
        ("{type: text}", "'true'", "true", "metadata-type"),
        (
            "{type: int}",
            "-9223372036854775808",
            "9223372036854775808",
            "metadata-type",
        ),
        ("{type: bool}", "false", "'false'", "metadata-type"),
        (
            "{type: date}",
            "'2024-02-29'",
            "'2023-02-29'",
            "metadata-type",
        ),
        (
            "{type: datetime}",
            "'2026-10-04T12:00:00Z'",
            "'2026-10-04T12:00:00'",
            "metadata-type",
        ),
        (
            "{type: uri}",
            "'urn:example:policy'",
            "'relative/path'",
            "metadata-type",
        ),
        (
            "{type: enum, values: [open, closed]}",
            "open",
            "Open",
            "metadata-enum",
        ),
        ("{type: int, min: 2, max: 3}", "2", "4", "metadata-range"),
        (
            "{type: list, item: int, min: 1, max: 2}",
            "[1,2]",
            "[]",
            "metadata-range",
        ),
        (
            "{type: string, pattern: abc}",
            "'xabcx'",
            "'ac'",
            "metadata-pattern",
        ),
        (
            "{type: string, pattern: '^abc$'}",
            "'abc'",
            "'xabcx'",
            "metadata-pattern",
        ),
        (
            "{type: object, additional_properties: false, fields: {x: {type: bool}}}",
            "{x: false}",
            "{extra: false}",
            "metadata-unknown-property",
        ),
        ("{type: object}", "{x: true}", "{1: true}", "metadata-type"),
    ];
    for (field, good, bad, code) in cases {
        let schema =
            format!("okf_ontology: '0.1'\nconcepts:\n  Note:\n    fields:\n      value: {field}\n");
        assert!(
            findings(&schema, &format!("type: Note\nvalue: {good}\n")).is_empty(),
            "{field}: {good}"
        );
        let f = findings(&schema, &format!("type: Note\nvalue: {bad}\n"));
        assert!(f.iter().any(|(c, _)| c == code), "{field}: {bad}: {f:?}");
    }
}
#[test]
fn malformed_constraints_and_named_types_are_configuration_errors() {
    for field in [
        "{type: enum, values: []}",
        "{type: string, pattern: '['}",
        "{type: int, min: 5, max: 3}",
        "{type: bool, pattern: x}",
        "{type: string, min: '2'}",
        "{type: DoesNotExist}",
        "{type: list}",
        "{type: int, unique_items: true}",
        "{type: list, item: string, additional_properties: false}",
    ] {
        let schema =
            format!("okf_ontology: '0.1'\nconcepts:\n  Note:\n    fields:\n      value: {field}\n");
        assert!(parse_ontology(&schema).is_err(), "accepted {field}");
    }
}
#[test]
fn derived_enum_values_and_list_item_replace_inherited_declarations() {
    let schema="okf_ontology: '0.1'\nfield_types:\n  Old: {base: enum, values: [old]}\n  New: {extends: Old, values: [new]}\n  Strings: {base: list, item: string}\n  Numbers: {extends: Strings, item: int}\nconcepts:\n  Note:\n    fields:\n      state: {type: New}\n      items: {type: Numbers}\n";
    assert!(findings(schema, "type: Note\nstate: new\nitems: [2]\n").is_empty());
    let f = findings(schema, "type: Note\nstate: old\nitems: ['2']\n");
    assert!(f.iter().any(|(c, p)| c == "metadata-enum" && p == "state"));
    assert!(f
        .iter()
        .any(|(c, p)| c == "metadata-type" && p == "items[0]"));
}
#[test]
fn unknown_constraints_remain_visible_and_root_metadata_stays_open() {
    let schema="okf_ontology: '0.1'\nconcepts:\n  Note:\n    fields:\n      value: {type: string, x-unimplemented: true}\n";
    let f = findings(
        schema,
        "type: Note\nvalue: okay\nunknown: {nested: [1, false]}\n",
    );
    assert_eq!(f, vec![("metadata-unenforced".into(), "value".into())]);
    let ontology = parse_ontology(schema).unwrap();
    assert!(serde_yaml::to_string(&ontology)
        .unwrap()
        .contains("x-unimplemented"));
}
#[test]
fn selector_subset_and_case_sensitive_explicit_traversal() {
    let root: serde_yaml::Value = serde_yaml::from_str(
        "norms:\n- action.name: inspect\n- other: false\npolicy.status: live\nquote\"key: yes\n",
    )
    .unwrap();
    for valid in [
        "title",
        "deadline.within",
        "tags[]",
        "norms[].deadline.from",
        "[\"policy.status\"]",
        "norms[][\"action.name\"]",
        "[\"quote\\\"key\"]",
    ] {
        assert!(Selector::parse(valid).is_ok(), "{valid}");
    }
    for invalid in [
        "$.title",
        "tags[*]",
        "tags[0]",
        "tags[0:1]",
        "..target",
        "norms[?(@.x)]",
        "a|b",
        "length(tags)",
    ] {
        assert!(Selector::parse(invalid).is_err(), "{invalid}");
    }
    assert_eq!(
        Selector::parse("norms[][\"action.name\"]")
            .unwrap()
            .select(&root)
            .leaves
            .len(),
        1
    );
    assert!(Selector::parse("norms.action.name")
        .unwrap()
        .select(&root)
        .leaves
        .is_empty());
    assert!(Selector::parse("Norms[]")
        .unwrap()
        .select(&root)
        .leaves
        .is_empty());
}
#[test]
fn reference_cardinality_counts_duplicate_and_unresolved_occurrences() {
    let schema="okf_ontology: '0.1'\nconcepts:\n  Note:\n    references:\n      links: {selector: 'rows[].target', target: Note, cardinality: 0..1}\n";
    let o = parse_ontology(schema).unwrap();
    let f = Frontmatter::from_map(
        serde_yaml::from_str("type: Note\nrows: [{target: /missing}, {target: /missing}]\n")
            .unwrap(),
    );
    let violations = check_concept(&o, &f, |_| None);
    let debug = format!("{violations:?}");
    assert!(
        debug.contains("Cardinality") || debug.contains("cardinality"),
        "{debug}"
    );
}
#[test]
fn invalid_relationship_anchors_and_dynamic_vocabularies_are_rejected() {
    for relationship in [
        "reference: targets\n        kind: fixed\n        kind_selector: rows[].kind",
        "reference: targets\n        kind_selector: unrelated[].kind\n        kinds: {one: {inverse: inverse}}",
        "reference: absent\n        kind: fixed",
        "reference: targets\n        kind: fixed\n        attributes: {detail: unrelated[].detail}",
    ] {
        let schema=format!("okf_ontology: '0.1'\nconcepts:\n  Note:\n    references:\n      targets: {{selector: 'rows[].target', target: Note, cardinality: 0..n}}\n    relationships:\n      relation:\n        {relationship}\n");
        assert!(parse_ontology(&schema).is_err(),"{schema}");
    }
}

#[test]
fn recursive_value_depth_limit_reports_incomplete_checks() {
    let mut schema = String::from("okf_ontology: '0.1'\nfield_types:\n");
    for i in 0..66 {
        schema.push_str(&format!(
            "  T{i}: {{base: object, fields: {{child: {{type: T{}}}}}}}\n",
            i + 1
        ));
    }
    schema.push_str(
        "  T66: {base: string}\nconcepts:\n  Note:\n    fields:\n      root: {type: T0}\n",
    );
    let value = format!("{}'leaf'{}", "{child: ".repeat(66), "}".repeat(66));
    let result = findings(&schema, &format!("type: Note\nroot: {value}\n"));
    assert!(
        result.iter().any(|(code, _)| code == "incomplete-check"),
        "{result:?}"
    );
}

#[test]
fn dynamic_kind_widening_and_vocabulary_disagreement_are_configuration_errors() {
    let base="okf_ontology: '0.1'\nconcepts:\n  Policy: {}\n  Team: {}\n  SecurePolicy: {extends: [Policy]}\n  Note:\n    fields:\n      kind: {type: enum, values: [implements]}\n    references:\n      target: {target: SecurePolicy, cardinality: 0..1}\n    relationships:\n      policy:\n        reference: target\n        kind_selector: kind\n        kinds:\n          implements: {inverse: implemented-by, target: SecurePolicy}\n";
    assert!(parse_ontology(base).is_ok(), "{:?}", parse_ontology(base));
    assert!(parse_ontology(&base.replace(
        "implements: {inverse: implemented-by, target: SecurePolicy}",
        "implements: {inverse: implemented-by, target: Policy}"
    ))
    .is_err());
    assert!(parse_ontology(&base.replace(
        "implements: {inverse: implemented-by, target: SecurePolicy}",
        "implements: {inverse: implemented-by, target: Team}"
    ))
    .is_err());
    assert!(parse_ontology(&base.replace(
        "implements: {inverse: implemented-by, target: SecurePolicy}",
        "other: {inverse: implemented-by, target: SecurePolicy}"
    ))
    .is_err());
}
