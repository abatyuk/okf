//! Structured authoring semantics, lifecycle behavior and atomic failure boundaries.
use okf_core::model::concept::ConceptId;
use okf_core::model::frontmatter::Frontmatter;
use okf_core::mutate::add::{add, AddOptions};
use okf_core::mutate::edit::{edit, EditSpec};
use okf_core::mutate::structured::{
    object_path, parse_path_assignment, pointer, PatchOperation, StructuredEdits,
};
use okf_core::parse::parse_concept;
use serde_yaml::Value;

fn yaml(text: &str) -> Value {
    serde_yaml::from_str(text).unwrap()
}
fn fm(text: &str) -> Frontmatter {
    Frontmatter::from_map(serde_yaml::from_str(text).unwrap())
}
fn patch(text: &str) -> StructuredEdits {
    StructuredEdits {
        patch: serde_yaml::from_str(text).unwrap(),
        ..Default::default()
    }
}
fn concept() -> &'static str {
    "---\ntype: Procedure\ntitle: Onboarding\ndeadline: {within: 24, unit: hours}\ngenerated: {by: 'mailto:writer@example.com', at: '2020-01-01T00:00:00Z'}\nverified: [{by: 'mailto:reviewer@example.com', at: '2020-01-02T00:00:00Z'}]\ncustom: {preserved: yes}\n---\n# Body\n\nUnchanged body.\n"
}

#[test]
fn object_paths_support_quoted_keys_but_not_array_or_query_syntax() {
    assert_eq!(
        object_path("deadline.within").unwrap(),
        ["deadline", "within"]
    );
    assert_eq!(
        object_path(r#"["deadline.within"]["a=b"].unit"#).unwrap(),
        ["deadline.within", "a=b", "unit"]
    );
    assert_eq!(
        object_path(r#"details["a\\b\"c"]"#).unwrap(),
        ["details", "a\\b\"c"]
    );
    assert_eq!(
        parse_path_assignment(r#"["a=b"].within=72"#).unwrap(),
        (r#"["a=b"].within"#.into(), "72".into())
    );
    for invalid in [
        "",
        ".a",
        "a.",
        "a..b",
        "a[]",
        "a[0]",
        "a.*",
        "a.0",
        "a key",
        "a[\"b\"]c",
        "a.[\"b\"]",
    ] {
        assert!(object_path(invalid).is_err(), "{invalid:?}");
    }
    assert_eq!(pointer("/a~1b/~0key/").unwrap(), ["a/b", "~key", ""]);
    for invalid in ["a", "/a~", "/a~2"] {
        assert!(pointer(invalid).is_err());
    }
}

#[test]
fn path_creation_and_deletion_preserve_siblings_and_are_atomic() {
    let mut document = fm("type: Procedure\ndeadline: {unit: hours}\nkeep: [a, b]\n");
    let operations = StructuredEdits {
        set_paths: vec![
            ("deadline.within".into(), yaml("72")),
            (r#"["literal.key"].nested"#.into(), yaml("{a: [1, 2]}")),
        ],
        unset_paths: vec!["missing.child".into()],
        ..Default::default()
    };
    operations.validate_conflicts(&[]).unwrap();
    operations.apply(&mut document).unwrap();
    assert_eq!(document.map["deadline"], yaml("{unit: hours, within: 72}"));
    assert_eq!(document.map["literal.key"], yaml("{nested: {a: [1, 2]}}"));
    assert_eq!(document.map["keep"], yaml("[a, b]"));
    let original = document.clone();
    let failing = StructuredEdits {
        set_paths: vec![
            ("deadline.other".into(), yaml("true")),
            ("keep.child".into(), yaml("1")),
        ],
        ..Default::default()
    };
    assert!(failing.apply(&mut document).is_err());
    assert_eq!(document, original);
    StructuredEdits {
        unset_paths: vec!["deadline.within".into()],
        ..Default::default()
    }
    .apply(&mut document)
    .unwrap();
    assert_eq!(document.map["deadline"], yaml("{unit: hours}"));
}

#[test]
fn structured_whole_values_replace_lists_and_store_null() {
    let mut document = fm("type: Procedure\nnorms: [{action: old}]\n");
    StructuredEdits {
        sets: vec![
            ("norms".into(), yaml("[{action: new}]")),
            ("optional".into(), Value::Null),
            ("version".into(), yaml("'123'")),
        ],
        ..Default::default()
    }
    .apply(&mut document)
    .unwrap();
    assert_eq!(document.map["norms"], yaml("[{action: new}]"));
    assert_eq!(document.map["optional"], Value::Null);
    assert_eq!(document.map["version"].as_str(), Some("123"));
}

#[test]
fn conflicting_assignments_rejected_without_restricting_siblings() {
    let siblings = StructuredEdits {
        set_paths: vec![
            ("deadline.within".into(), yaml("72")),
            ("deadline.unit".into(), yaml("hours")),
        ],
        ..Default::default()
    };
    siblings.validate_conflicts(&[]).unwrap();
    assert!(siblings.validate_conflicts(&["deadline".into()]).is_err());
    let mut overlap = siblings.clone();
    overlap.sets.push(("deadline".into(), yaml("{}")));
    assert!(overlap.validate_conflicts(&[]).is_err());
    assert!(StructuredEdits {
        set_paths: vec![("a".into(), yaml("1"))],
        unset_paths: vec!["a".into()],
        ..Default::default()
    }
    .validate_conflicts(&[])
    .is_err());
    let mut overlap = patch("[{op: replace, path: /deadline/within, value: 48}]");
    overlap.set_paths = siblings.set_paths;
    assert!(overlap.validate_conflicts(&[]).is_err());
}

#[test]
fn all_patch_operations_use_strict_pointer_and_array_semantics() {
    let mut document = fm("type: Procedure\nlist: [a, b, c]\na/b: {'~key': 1}\n");
    let operations = patch(
        r#"
- {op: test, path: '/a~1b/~0key', value: 1}
- {op: replace, path: '/a~1b/~0key', value: 2}
- {op: add, path: /list/1, value: inserted}
- {op: remove, path: /list/2}
- {op: move, from: /list/0, path: /list/2}
- {op: copy, from: /list/0, path: /list/-}
- {op: add, path: /new, value: {nested: true}}
- {op: replace, path: /new/nested, value: false}
"#,
    );
    operations.validate_conflicts(&[]).unwrap();
    operations.apply(&mut document).unwrap();
    assert_eq!(document.map["list"], yaml("[inserted, c, a, inserted]"));
    assert_eq!(document.map["a/b"], yaml("{'~key': 2}"));
    assert_eq!(document.map["new"], yaml("{nested: false}"));
    // Replacing the root is allowed if the result remains a string-keyed mapping.
    patch("[{op: replace, path: '', value: {type: Procedure, replacement: true}}]")
        .apply(&mut document)
        .unwrap();
    assert_eq!(document, fm("type: Procedure\nreplacement: true\n"));
}

#[test]
fn patch_failure_preserves_original_in_memory_and_on_disk() {
    let original = fm("type: Procedure\nlist: [a, b]\nobject: {nested: true}\n");
    for failing in [
        "[{op: add, path: /list/01, value: x}]",
        "[{op: replace, path: /list/-, value: x}]",
        "[{op: add, path: /list/3, value: x}]",
        "[{op: remove, path: /list/2}]",
        "[{op: add, path: /absent/nested, value: x}]",
        "[{op: replace, path: /missing, value: x}]",
        "[{op: move, from: /missing, path: /missing}]",
        "[{op: move, from: /object, path: /object/nested/child}]",
        "[{op: replace, path: '', value: []}]",
        "[{op: add, path: /new, value: x}, {op: test, path: /list/0, value: wrong}]",
    ] {
        let mut document = original.clone();
        assert!(patch(failing).apply(&mut document).is_err(), "{failing}");
        assert_eq!(document, original, "{failing}");
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("onboarding.md");
    std::fs::write(&path, concept()).unwrap();
    assert!(edit(dir.path(), "onboarding", &EditSpec {
        structured: patch("[{op: replace, path: /deadline/within, value: 48}, {op: test, path: /deadline/unit, value: days}]"),
        ..Default::default()
    }).is_err());
    assert_eq!(std::fs::read_to_string(path).unwrap(), concept());
}

#[test]
fn structured_edit_lifecycle_preview_and_standard_validation_match_writes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("onboarding.md");
    std::fs::write(&path, concept()).unwrap();
    let spec = EditSpec {
        structured: StructuredEdits {
            set_paths: vec![("deadline.within".into(), yaml("72"))],
            ..Default::default()
        },
        dry_run: true,
        ..Default::default()
    };
    let preview = edit(dir.path(), "onboarding", &spec).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), concept());
    assert_eq!(preview.before, concept());
    let changed = parse_concept(ConceptId::parse("onboarding").unwrap(), &preview.after).unwrap();
    assert!(!changed.frontmatter.map.contains_key("verified"));
    assert_eq!(
        changed.frontmatter.map["generated"]["by"].as_str(),
        Some("mailto:writer@example.com")
    );
    assert_ne!(
        changed.frontmatter.map["generated"]["at"].as_str(),
        Some("2020-01-01T00:00:00Z")
    );
    assert_eq!(changed.frontmatter.map["custom"], yaml("{preserved: yes}"));
    assert_eq!(changed.body, "# Body\n\nUnchanged body.\n");
    let persisted = edit(
        dir.path(),
        "onboarding",
        &EditSpec {
            dry_run: false,
            ..spec
        },
    )
    .unwrap();
    assert_eq!(persisted.after, std::fs::read_to_string(&path).unwrap());
    let original = std::fs::read_to_string(&path).unwrap();
    for dry_run in [true, false] {
        assert!(edit(
            dir.path(),
            "onboarding",
            &EditSpec {
                structured: patch("[{op: remove, path: /type}]"),
                dry_run,
                ..Default::default()
            }
        )
        .is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    }
}

#[test]
fn test_only_patch_does_not_invalidate_verification_or_generated_timestamp() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("onboarding.md");
    std::fs::write(&path, concept()).unwrap();
    let result = edit(
        dir.path(),
        "onboarding",
        &EditSpec {
            structured: patch("[{op: test, path: /deadline/within, value: 24}]"),
            ..Default::default()
        },
    )
    .unwrap();
    let changed = parse_concept(ConceptId::parse("onboarding").unwrap(), &result.after).unwrap();
    assert!(changed.frontmatter.map.contains_key("verified"));
    assert_eq!(
        changed.frontmatter.map["generated"]["at"].as_str(),
        Some("2020-01-01T00:00:00Z")
    );
}

#[test]
fn add_preview_creates_no_directories_and_can_write_structured_values() {
    let dir = tempfile::tempdir().unwrap();
    let opts = AddOptions {
        concept_type: Some("Procedure".into()),
        structured: StructuredEdits {
            sets: vec![(
                "norms".into(),
                yaml("[{bearer: /teams/platform.md, action: review-access}]"),
            )],
            set_paths: vec![
                ("deadline.within".into(), yaml("72")),
                ("deadline.unit".into(), yaml("hours")),
            ],
            ..Default::default()
        },
        dry_run: true,
        ..Default::default()
    };
    let result = add(dir.path(), "procedures/onboarding", None, &opts).unwrap();
    assert!(!dir.path().join("procedures").exists());
    assert!(result.after.contains("within: 72"));
    let result = add(
        dir.path(),
        "procedures/onboarding",
        None,
        &AddOptions {
            dry_run: false,
            ..opts
        },
    )
    .unwrap();
    assert_eq!(result.after, std::fs::read_to_string(&result.path).unwrap());
}

#[test]
fn add_structured_runtime_is_available_to_attested_validation_and_conflicts_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let opts = AddOptions {
        attested: true,
        inline_computation: Some("print('ok')".into()),
        structured: StructuredEdits {
            sets: vec![("runtime".into(), yaml("python"))],
            ..Default::default()
        },
        ..Default::default()
    };
    let result = add(dir.path(), "run", None, &opts).unwrap();
    assert!(result.after.contains("```python"));
    let error = add(
        dir.path(),
        "other",
        None,
        &AddOptions {
            runtime: Some("python".into()),
            ..opts
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("conflicts"));
    assert!(!dir.path().join("other.md").exists());
}

#[test]
fn patch_operations_require_all_standard_members() {
    for malformed in [
        "[{op: unknown, path: /x}]",
        "[{op: add, path: /x}]",
        "[{op: move, path: /x}]",
        "[{op: remove}]",
    ] {
        assert!(
            serde_yaml::from_str::<Vec<PatchOperation>>(malformed).is_err(),
            "{malformed}"
        );
    }
    // RFC 6902 requires operation members not defined by the operation to be ignored.
    let operations: Vec<PatchOperation> =
        serde_yaml::from_str("[{op: add, path: /x, value: null, ignored: true}]").unwrap();
    assert_eq!(operations.len(), 1);
}

#[test]
fn patch_tests_compare_numbers_exactly_and_objects_without_order() {
    let mut document = fm("type: Procedure\nobject: {first: 1, second: [1.0]}\nlarge: 9007199254740993\nmaximum: 18446744073709551615\n");
    patch("[{op: test, path: /object, value: {second: [1], first: 1.0}}]")
        .apply(&mut document)
        .unwrap();
    patch("[{op: test, path: /large, value: 9007199254740993}]")
        .apply(&mut document)
        .unwrap();
    assert!(
        patch("[{op: test, path: /large, value: 9007199254740992.0}]")
            .apply(&mut document)
            .is_err()
    );
    assert!(
        patch("[{op: test, path: /maximum, value: 18446744073709551616.0}]")
            .apply(&mut document)
            .is_err()
    );
}
