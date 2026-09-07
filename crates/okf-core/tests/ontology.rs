//! Tests for the ontology layer: parse the INTENT example, field_types `extends` + object
//! composition, `extends` cycle rejection, cardinality parsing, and an add→load round-trip.
use std::path::PathBuf;

use okf_core::model::frontmatter::Frontmatter;
use okf_core::ontology::edit::{add_concept_type, save_ontology};
use okf_core::ontology::field_types::{check_concept, resolve_type_name, ViolationKind};
use okf_core::ontology::load::{load_ontology, parse_ontology, try_load};
use okf_core::ontology::schema::{
    Cardinality, ConceptType, Field, FieldType, ReferenceRule, Target,
};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ontology/ontology.yaml")
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ontology")
}

#[test]
fn parses_intent_example() {
    let ont = load_ontology(&fixture()).unwrap();
    assert_eq!(ont.okf_ontology, "0.1");

    // Concept keys preserved verbatim, including the spaced quoted key, in order.
    let keys: Vec<&str> = ont.concepts.keys().map(String::as_str).collect();
    assert_eq!(keys, vec!["Policy", "Computation", "BigQuery Table", "Metric"]);

    // Source kinds captured.
    assert_eq!(ont.source_kinds.first().map(String::as_str), Some("git-commit"));

    // Policy specifics.
    let policy = &ont.concepts["Policy"];
    assert_eq!(policy.requires, vec!["description"]);
    assert!(policy.fields["owner"].required);
    assert_eq!(policy.fields["review_cycle"].type_name, "enum");
    assert_eq!(
        policy.fields["review_cycle"].values.as_deref(),
        Some(&["monthly".into(), "quarterly".into(), "annual".into()][..])
    );
    assert_eq!(
        policy.trust.as_ref().unwrap().min_tier.as_deref(),
        Some("human-reviewed")
    );
    assert_eq!(policy.references["computations"].cardinality, Cardinality::ZeroN);

    // Computation: attested + a union reference target.
    let comp = &ont.concepts["Computation"];
    assert!(comp.attested);
    let inputs = &comp.references["inputs"];
    assert_eq!(inputs.cardinality, Cardinality::OneN);
    assert_eq!(inputs.target.types(), vec!["BigQuery Table", "Metric"]);
    assert!(matches!(inputs.target, Target::Many(_)));

    // Metric: single-target 1..1.
    assert_eq!(
        ont.concepts["Metric"].references["computed_by"].cardinality,
        Cardinality::OneOne
    );
}

#[test]
fn field_types_extends_and_object_composition() {
    let ont = load_ontology(&fixture()).unwrap();

    // `positive_money extends money` inherits base + pattern and adds `min`.
    let mut chain = Vec::new();
    let pm = resolve_type_name(&ont, "positive_money", &mut chain).unwrap();
    assert_eq!(pm.base, FieldType::String);
    assert!(pm.constraints.contains_key("pattern")); // inherited from `money`
    assert_eq!(
        pm.constraints.get("min").and_then(|v| v.as_i64()),
        Some(0)
    ); // added by derived def

    // `contact` object composes sub-fields.
    let mut chain = Vec::new();
    let contact = resolve_type_name(&ont, "contact", &mut chain).unwrap();
    assert_eq!(contact.base, FieldType::Object);
    assert!(contact.fields["name"].required);
    assert_eq!(contact.fields["name"].ty.base, FieldType::String);
    assert!(!contact.fields["email"].required);
    assert_eq!(contact.fields["email"].ty.base, FieldType::Uri);
}

#[test]
fn extends_cycle_is_rejected() {
    let yaml = r#"
okf_ontology: "0.1"
field_types:
  a: { extends: b }
  b: { extends: c }
  c: { extends: a }
concepts: {}
"#;
    let err = parse_ontology(yaml).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("cycle"), "expected cycle rejection, got: {msg}");
    // Cycles are a usage error (exit class 2).
    assert_eq!(err.exit_code(), 2);
}

#[test]
fn cardinality_parsing_vocabulary() {
    assert_eq!("0..1".parse::<Cardinality>().unwrap(), Cardinality::ZeroOne);
    assert_eq!("1..1".parse::<Cardinality>().unwrap(), Cardinality::OneOne);
    assert_eq!("0..n".parse::<Cardinality>().unwrap(), Cardinality::ZeroN);
    assert_eq!("1..n".parse::<Cardinality>().unwrap(), Cardinality::OneN);
    assert!("2..5".parse::<Cardinality>().is_err());

    // min/max/permits semantics.
    assert_eq!(Cardinality::ZeroOne.min(), 0);
    assert_eq!(Cardinality::ZeroOne.max(), Some(1));
    assert_eq!(Cardinality::OneN.max(), None);
    assert!(Cardinality::OneN.permits(3));
    assert!(!Cardinality::OneN.permits(0));
    assert!(!Cardinality::OneOne.permits(2));

    // Round-trips through Display.
    assert_eq!(Cardinality::ZeroN.to_string(), "0..n");
}

#[test]
fn try_load_is_non_fatal_when_absent() {
    let tmp = std::env::temp_dir().join("okf-ontology-absent-test");
    std::fs::create_dir_all(&tmp).unwrap();
    // Ensure no ontology.yaml here.
    let _ = std::fs::remove_file(tmp.join("ontology.yaml"));
    assert!(try_load(&tmp).unwrap().is_none());

    // And present-and-valid loads.
    assert!(try_load(&fixture_dir()).unwrap().is_some());
}

#[test]
fn add_then_load_round_trip() {
    let mut ont = load_ontology(&fixture()).unwrap();

    let mut new_type = ConceptType {
        description: Some("A dashboard built on metrics.".to_string()),
        requires: vec!["description".to_string()],
        ..Default::default()
    };
    new_type.fields.insert(
        "layout".to_string(),
        Field {
            type_name: "enum".to_string(),
            required: true,
            values: Some(vec!["grid".to_string(), "list".to_string()]),
            ..Default::default()
        },
    );
    new_type.references.insert(
        "metrics".to_string(),
        ReferenceRule {
            target: Target::One("Metric".to_string()),
            cardinality: Cardinality::OneN,
            extra: Default::default(),
        },
    );

    add_concept_type(&mut ont, "Dashboard", new_type).unwrap();

    // Adding a duplicate is rejected.
    assert!(add_concept_type(&mut ont, "Dashboard", ConceptType::default()).is_err());

    // Persist to a temp file and reload.
    let out = std::env::temp_dir().join("okf-ontology-roundtrip.yaml");
    save_ontology(&out, &ont).unwrap();
    let reloaded = load_ontology(&out).unwrap();

    let dash = &reloaded.concepts["Dashboard"];
    assert_eq!(dash.description.as_deref(), Some("A dashboard built on metrics."));
    assert!(dash.fields["layout"].required);
    assert_eq!(dash.references["metrics"].cardinality, Cardinality::OneN);
    assert!(dash.references["metrics"].target.allows("Metric"));

    // Original concepts survived the round-trip.
    assert!(reloaded.concepts.contains_key("BigQuery Table"));
    assert_eq!(reloaded.concepts.len(), ont.concepts.len());
}

#[test]
fn check_concept_flags_missing_and_cardinality_and_target() {
    let ont = load_ontology(&fixture()).unwrap();

    // A Policy missing required `owner`/`effective_date`, and `description` (a `requires`).
    let mut fm = Frontmatter::new();
    fm.map.insert("type".into(), "Policy".into());
    // one computations link pointing at a Metric (wrong target) — 0..n allows the count.
    fm.map.insert(
        "computations".into(),
        serde_yaml::Value::Sequence(vec!["/metrics/revenue".into()]),
    );

    let resolve = |link: &str| {
        if link == "/metrics/revenue" {
            Some("Metric".to_string())
        } else {
            None
        }
    };
    let violations = check_concept(&ont, &fm, resolve);
    let kinds: Vec<&ViolationKind> = violations.iter().map(|v| &v.kind).collect();

    assert!(kinds
        .iter()
        .any(|k| matches!(k, ViolationKind::MissingRequiredBuiltin(f) if f == "description")));
    assert!(kinds
        .iter()
        .any(|k| matches!(k, ViolationKind::MissingRequiredField(f) if f == "owner")));
    assert!(kinds
        .iter()
        .any(|k| matches!(k, ViolationKind::MissingRequiredField(f) if f == "effective_date")));
    // computations rule allows Computation; a Metric link is a wrong target.
    assert!(kinds
        .iter()
        .any(|k| matches!(k, ViolationKind::WrongReferenceTarget(key) if key == "computations")));
}

#[test]
fn check_concept_unknown_type_is_advisory() {
    let ont = load_ontology(&fixture()).unwrap();
    let mut fm = Frontmatter::new();
    fm.map.insert("type".into(), "SomethingElse".into());
    let violations = check_concept(&ont, &fm, |_| None);
    assert_eq!(violations.len(), 1);
    assert!(matches!(violations[0].kind, ViolationKind::UnknownType));
}
