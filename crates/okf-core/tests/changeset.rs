use okf_core::mutate::{
    changeset::{self, ChangeSet, Operation},
    transaction::{self, FileChange},
};
use std::{fs, path::PathBuf};
fn note(body: &str) -> String {
    format!("---\ntype: Note\n---\n{body}")
}
fn set(operations: Vec<Operation>) -> ChangeSet {
    ChangeSet {
        version: 1,
        operations,
    }
}

#[test]
fn split_retargets_selected_records_and_commits_only_after_preview() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("old.md"), note("old\n")).unwrap();
    fs::write(root.path().join("a.md"), note("[old](old.md)\n")).unwrap();
    fs::write(root.path().join("b.md"), note("[old](old.md)\n")).unwrap();
    let plan = changeset::prepare(
        root.path(),
        &set(vec![
            Operation::Create {
                path: "left".into(),
                content: note("left\n"),
            },
            Operation::Create {
                path: "right".into(),
                content: note("right\n"),
            },
            Operation::Retarget {
                from: "old".into(),
                to: "left".into(),
                within: Some(vec!["a".into()]),
            },
            Operation::Retarget {
                from: "old".into(),
                to: "right".into(),
                within: Some(vec!["b".into()]),
            },
            Operation::Remove {
                path: "old.md".into(),
            },
        ]),
    )
    .unwrap();
    assert!(root.path().join("old.md").exists());
    assert!(!root.path().join("left.md").exists());
    let digest = plan.base_digest.clone();
    changeset::apply(root.path(), plan, Some(&digest)).unwrap();
    assert!(!root.path().join("old.md").exists());
    assert!(fs::read_to_string(root.path().join("a.md"))
        .unwrap()
        .contains("(left.md)"));
    assert!(fs::read_to_string(root.path().join("b.md"))
        .unwrap()
        .contains("(right.md)"));
    assert!(!root.path().join(transaction::JOURNAL).exists());
}

#[test]
fn incomplete_cleanup_rejects_deleted_csv_source_without_writes() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("nested")).unwrap();
    fs::write(
        root.path().join("nested/note.md"),
        "---\ntype: Note\nsources:\n- kind: file\n  resource: data.csv\n---\n",
    )
    .unwrap();
    fs::write(root.path().join("data.csv"), "a,b\n1,2\n").unwrap();
    let error = changeset::prepare(
        root.path(),
        &set(vec![Operation::Remove {
            path: "data.csv".into(),
        }]),
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("data.csv"), "{error}");
    assert!(root.path().join("data.csv").exists());
}

#[test]
fn staged_input_change_and_wrong_expected_digest_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("old.md"), note("old\n")).unwrap();
    let operations = set(vec![Operation::Move {
        from: "old".into(),
        to: "new".into(),
    }]);
    let plan = changeset::prepare(root.path(), &operations).unwrap();
    assert!(changeset::apply(root.path(), plan, Some("wrong")).is_err());
    let plan = changeset::prepare(root.path(), &operations).unwrap();
    fs::write(root.path().join("old.md"), note("concurrent edit\n")).unwrap();
    assert!(changeset::apply(root.path(), plan, None).is_err());
    assert!(!root.path().join("new.md").exists());
}

#[test]
fn replacement_invalidates_verification() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("old.md"),
        "---\ntype: Note\nverified:\n- by: human/test\n  at: 2026-01-01T00:00:00Z\n---\nold\n",
    )
    .unwrap();
    let plan = changeset::prepare(
        root.path(),
        &set(vec![Operation::Replace {
            path: "old".into(),
            content: note("new\n"),
        }]),
    )
    .unwrap();
    changeset::apply(root.path(), plan, None).unwrap();
    let after = fs::read_to_string(root.path().join("old.md")).unwrap();
    assert!(!after.contains("verified:"));
    assert!(after.contains("new"));
}

#[test]
fn hidden_authored_files_remain_in_staging_validation() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(".hidden.md"), note("hidden\n")).unwrap();
    fs::write(root.path().join("old.md"), note("[hidden](.hidden.md)\n")).unwrap();
    let plan = changeset::prepare(
        root.path(),
        &set(vec![Operation::Move {
            from: "old".into(),
            to: "new".into(),
        }]),
    )
    .unwrap();
    changeset::apply(root.path(), plan, None).unwrap();
    assert_eq!(
        fs::read_to_string(root.path().join(".hidden.md")).unwrap(),
        note("hidden\n")
    );
}

#[test]
fn recovery_restores_partial_publication_and_refuses_intervening_edits() {
    let root = tempfile::tempdir().unwrap();
    let changes = vec![
        FileChange {
            path: PathBuf::from("a.txt"),
            before: Some(b"old".to_vec()),
            after: Some(b"new".to_vec()),
        },
        FileChange {
            path: PathBuf::from("b.txt"),
            before: None,
            after: Some(b"created".to_vec()),
        },
    ];
    let journal = root.path().join(transaction::JOURNAL);
    fs::create_dir(&journal).unwrap();
    fs::write(
        journal.join("manifest.json"),
        serde_json::to_vec(&changes).unwrap(),
    )
    .unwrap();
    fs::write(root.path().join("a.txt"), "new").unwrap();
    fs::write(root.path().join("b.txt"), "intervening edit").unwrap();
    assert!(transaction::recover(root.path()).is_err());
    assert_eq!(
        fs::read_to_string(root.path().join("a.txt")).unwrap(),
        "new"
    );
    fs::write(root.path().join("b.txt"), "created").unwrap();
    transaction::recover(root.path()).unwrap();
    assert_eq!(
        fs::read_to_string(root.path().join("a.txt")).unwrap(),
        "old"
    );
    assert!(!root.path().join("b.txt").exists());
    assert!(!journal.exists());
}

#[test]
fn transaction_preflight_rejects_bad_destination_before_any_publication() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.txt"), "old").unwrap();
    fs::write(root.path().join("blocked"), "file").unwrap();
    let result = transaction::commit(
        root.path(),
        vec![
            FileChange {
                path: "a.txt".into(),
                before: Some(b"old".to_vec()),
                after: Some(b"new".to_vec()),
            },
            FileChange {
                path: "blocked/new.txt".into(),
                before: None,
                after: Some(b"new".to_vec()),
            },
        ],
    );
    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(root.path().join("a.txt")).unwrap(),
        "old"
    );
}

#[test]
fn retarget_can_select_structural_navigation_and_rejects_misspelled_referrers() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("old.md"), note("old\n")).unwrap();
    fs::write(root.path().join("new.md"), note("new\n")).unwrap();
    fs::write(
        root.path().join("index.md"),
        "# Records\n\n- [Old](old.md)\n",
    )
    .unwrap();
    let plan = changeset::prepare(
        root.path(),
        &set(vec![Operation::Retarget {
            from: "old".into(),
            to: "new".into(),
            within: Some(vec!["index.md".into()]),
        }]),
    )
    .unwrap();
    changeset::apply(root.path(), plan, None).unwrap();
    assert!(fs::read_to_string(root.path().join("index.md"))
        .unwrap()
        .contains("(new.md)"));
    assert!(changeset::prepare(
        root.path(),
        &set(vec![Operation::Retarget {
            from: "old".into(),
            to: "new".into(),
            within: Some(vec!["typo".into()])
        }])
    )
    .is_err());
}

#[test]
fn opaque_image_cleanup_and_bare_declared_links_are_guarded() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("image.png"), "image bytes").unwrap();
    fs::write(root.path().join("note.md"), note("![image](image.png)\n")).unwrap();
    assert!(changeset::prepare(
        root.path(),
        &set(vec![Operation::Remove {
            path: "image.png".into()
        }])
    )
    .is_err());
    fs::write(root.path().join("ontology.yaml"), "okf_ontology: '0.1'\nconcepts:\n  Note:\n    references:\n      depends_on: {target: Note, cardinality: 0..1}\n").unwrap();
    fs::write(
        root.path().join("ref.md"),
        "---\ntype: Note\ndepends_on: note\n---\n",
    )
    .unwrap();
    assert!(changeset::prepare(
        root.path(),
        &set(vec![Operation::Remove {
            path: "note.md".into()
        }])
    )
    .is_err());
    assert!(okf_core::mutate::rm::rm(root.path(), "note", false).is_err());
}

#[test]
fn final_state_validation_allows_transient_dangling_links_inside_a_plan() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("old.md"), note("old\n")).unwrap();
    fs::write(root.path().join("ref.md"), note("[old](old.md)\n")).unwrap();
    let plan = changeset::prepare(
        root.path(),
        &set(vec![
            Operation::Remove {
                path: "old.md".into(),
            },
            Operation::Retarget {
                from: "old".into(),
                to: "new".into(),
                within: None,
            },
            Operation::Create {
                path: "new".into(),
                content: note("replacement\n"),
            },
        ]),
    )
    .unwrap();
    assert!(root.path().join("old.md").exists());
    changeset::apply(root.path(), plan, None).unwrap();
    assert!(!root.path().join("old.md").exists());
    assert!(fs::read_to_string(root.path().join("ref.md"))
        .unwrap()
        .contains("(new.md)"));
}

#[test]
fn typed_text_sources_prevent_dangling_cleanup_and_follow_moves() {
    for (kind, resource) in [
        ("line-range", "source.md#L1"),
        ("markdown-heading", "source.md#heading"),
    ] {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("nested")).unwrap();
        fs::write(root.path().join("source.md"), note("# Heading\nEvidence\n")).unwrap();
        fs::write(
            root.path().join("nested/consumer.md"),
            format!("---\ntype: Note\nsources:\n- {{kind: {kind}, resource: '{resource}'}}\n---\n"),
        )
        .unwrap();
        let error = changeset::prepare(
            root.path(),
            &set(vec![Operation::Remove {
                path: "source.md".into(),
            }]),
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains("source.md"), "{error}");
        let plan = changeset::prepare(
            root.path(),
            &set(vec![Operation::Move {
                from: "source".into(),
                to: "moved/source".into(),
            }]),
        )
        .unwrap();
        changeset::apply(root.path(), plan, None).unwrap();
        let updated = fs::read_to_string(root.path().join("nested/consumer.md")).unwrap();
        assert!(updated.contains(&format!("moved/{resource}")), "{updated}");
    }
}

#[test]
fn preview_rejects_unsupported_file_to_directory_transition_without_writes() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("blocked"), "existing artifact").unwrap();
    let result = changeset::prepare(
        root.path(),
        &set(vec![
            Operation::Remove {
                path: "blocked".into(),
            },
            Operation::Create {
                path: "blocked/new".into(),
                content: note("new\n"),
            },
        ]),
    );
    let error = result
        .err()
        .expect("preview must reject unsupported parent type transition");
    assert!(error.to_string().contains("real directories"), "{error}");
    assert_eq!(
        fs::read_to_string(root.path().join("blocked")).unwrap(),
        "existing artifact"
    );
    assert!(!root.path().join(transaction::JOURNAL).exists());
    assert!(!root.path().join(transaction::LOCK).exists());
}

#[test]
fn a_new_typed_source_cannot_hide_behind_an_existing_broken_consumer() {
    let root = tempfile::tempdir().unwrap();
    let content = "---\ntype: Note\nsources:\n- {kind: file, resource: missing.csv}\n---\n";
    fs::write(root.path().join("existing.md"), content).unwrap();
    let error = changeset::prepare(
        root.path(),
        &set(vec![Operation::Create {
            path: "new".into(),
            content: content.into(),
        }]),
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("/new: missing.csv"), "{error}");
    assert!(!root.path().join("new.md").exists());
}
