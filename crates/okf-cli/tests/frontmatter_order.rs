use assert_cmd::prelude::*;
use std::process::Command;

fn lint(root: &std::path::Path, fix: bool) -> Vec<serde_json::Value> {
    let mut cmd = Command::cargo_bin("okf").unwrap();
    cmd.current_dir(root)
        .arg("lint")
        .arg(root)
        .args(["--json", "--fail-on", "never"]);
    if fix {
        cmd.arg("--fix");
    }
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn lint_fix_normalizes_recursively_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("ontology.yaml"),
        r#"okf_ontology: '0.1'
field_types:
  Base:
    base: object
    fields:
      name: {type: string}
      code: {type: string}
  Entry:
    extends: Base
concepts:
  Note:
    fields:
      title: {type: string}
      details:
        type: object
        fields:
          second: {type: string}
          first: {type: string}
      entries: {type: list, item: Entry}
"#,
    )
    .unwrap();
    let original = "---\ncustom: retain\nentries:\n- extra: keep\n  code: C\n  name: N\ndetails:\n  extra: keep\n  first: A\n  second: B\ntitle: Test\ntype: Note\nverified:\n- by: human:reviewer\n  at: '2026-10-18T12:00:00Z'\n---\n# Body\n\nKeep this text.\n";
    std::fs::write(root.join("note.md"), original).unwrap();
    // An unknown type still puts type first, leaving all other keys in authored order.
    std::fs::write(
        root.join("other.md"),
        "---\nz: 1\ntype: Unknown\na: 2\n---\nOther\n",
    )
    .unwrap();
    let before = lint(root, false);
    let order: Vec<_> = before
        .iter()
        .filter(|v| v["rule"] == "frontmatter-order")
        .collect();
    assert_eq!(order.len(), 2);
    assert!(order.iter().all(|v| v["severity"] == "warn"));
    assert_eq!(
        std::fs::read_to_string(root.join("note.md")).unwrap(),
        original
    );
    let after = lint(root, true);
    assert!(after.iter().any(|v| v["kind"] == "fix" && v["fixed"] == 2));
    assert!(!after
        .iter()
        .any(|v| v["kind"] == "finding" && v["rule"] == "frontmatter-order"));
    let text = std::fs::read_to_string(root.join("note.md")).unwrap();
    assert!(text.starts_with("---\ntype: Note\ntitle: Test\ndetails:\n  second: B\n  first: A\n  extra: keep\nentries:\n- name: N\n  code: C\n  extra: keep\ncustom: retain\n"), "{text}");
    let id = okf_core::model::concept::ConceptId::from_relative("note");
    let old = okf_core::parse::parse_concept(id.clone(), original).unwrap();
    let new = okf_core::parse::parse_concept(id, &text).unwrap();
    assert_eq!(old.body, new.body);
    assert_eq!(
        old.frontmatter.get("verified"),
        new.frontmatter.get("verified")
    );
    assert!(std::fs::read_to_string(root.join("other.md"))
        .unwrap()
        .starts_with("---\ntype: Unknown\nz: 1\na: 2\n"));
    assert!(lint(root, true)
        .iter()
        .any(|v| v["kind"] == "fix" && v["fixed"] == 0));
    assert_eq!(std::fs::read_to_string(root.join("note.md")).unwrap(), text);
}

#[test]
fn ordinary_edit_normalizes_and_rule_can_be_disabled() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("okf.toml"),
        "[bundle_settings.default.lint]\nfrontmatter_order = 'off'\n",
    )
    .unwrap();
    let original = "---\ncustom: keep\ntype: Note\n---\nBody\n";
    std::fs::write(root.join("note.md"), original).unwrap();
    assert!(!lint(root, true)
        .iter()
        .any(|v| v["rule"] == "frontmatter-order"));
    assert_eq!(
        std::fs::read_to_string(root.join("note.md")).unwrap(),
        original
    );
    Command::cargo_bin("okf")
        .unwrap()
        .current_dir(root)
        .arg("edit")
        .arg("note")
        .arg(root)
        .args(["--set", "title=Updated"])
        .assert()
        .success();
    let text = std::fs::read_to_string(root.join("note.md")).unwrap();
    assert!(
        text.starts_with("---\ntype: Note\ncustom: keep\ntitle: Updated\n"),
        "{text}"
    );
}

#[test]
fn writes_and_previews_follow_configured_external_ontology() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir(root.join("bundle")).unwrap();
    std::fs::write(
        root.join("okf.toml"),
        "[bundle_settings.default]\nontology = 'schema.yaml'\n",
    )
    .unwrap();
    std::fs::write(root.join("schema.yaml"), "okf_ontology: '0.1'\nconcepts:\n  Note:\n    fields:\n      title: {type: string}\n      priority: {type: int}\n    references:\n      related: {target: Note, cardinality: '0..n'}\n").unwrap();
    let path = root.join("bundle/note.md");
    let original =
        "---\ncustom: keep\npriority: 1\nrelated: []\ntype: Note\ntitle: Old\n---\nBody\n";
    std::fs::write(&path, original).unwrap();
    let edit = |dry: bool| {
        let mut cmd = Command::cargo_bin("okf").unwrap();
        cmd.current_dir(root)
            .arg("edit")
            .arg("note")
            .arg(root.join("bundle"))
            .args(["--set", "title=Updated"]);
        if dry {
            cmd.arg("--dry-run");
        }
        cmd.assert().success();
    };
    edit(true);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    edit(false);
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "---\ntype: Note\ntitle: Updated\npriority: 1\nrelated: []\ncustom: keep\n---\nBody\n"
    );
}

#[test]
fn catalog_lint_fix_respects_scope_and_each_bundles_ontology_and_settings() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("okf.toml"), "catalog = 'catalog.yaml'\ndefault_bundle = {id = 'test.a'}\n[bundle_settings.\"test.off\".lint]\nfrontmatter_order = 'off'\n").unwrap();
    std::fs::write(root.join("catalog.yaml"), "catalog_version: 1\nbundles:\n  test.a:\n    location: {type: directory, path: a}\n  test.b:\n    location: {type: directory, path: b}\n  test.off:\n    location: {type: directory, path: off}\n").unwrap();
    let original = "---\ncustom: keep\nsecond: B\nfirst: A\ntype: Note\nverified:\n- by: human:reviewer\n  at: '2026-10-18T12:00:00Z'\n---\nKeep body.\n";
    for (name, fields) in [
        (
            "a",
            "      first: {type: string}\n      second: {type: string}\n",
        ),
        (
            "b",
            "      second: {type: string}\n      first: {type: string}\n",
        ),
        ("off", "      first: {type: string}\n"),
    ] {
        std::fs::create_dir(root.join(name)).unwrap();
        std::fs::write(
            root.join(name).join("ontology.yaml"),
            format!("okf_ontology: '0.1'\nconcepts:\n  Note:\n    fields:\n{fields}"),
        )
        .unwrap();
        std::fs::write(root.join(name).join("note.md"), original).unwrap();
    }
    let run = |extra: &[&str]| {
        let output = Command::cargo_bin("okf")
            .unwrap()
            .current_dir(root)
            .args(["lint", "--json", "--fail-on", "never"])
            .args(extra)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>()
    };
    assert!(run(&[]).iter().any(|r| r["rule"] == "frontmatter-order"));
    assert_eq!(
        std::fs::read_to_string(root.join("a/note.md")).unwrap(),
        original
    );
    let selected = run(&["--fix"]);
    assert!(selected
        .iter()
        .any(|r| r["kind"] == "fix" && r["bundle"] == "test.a" && r["fixed"] == 1));
    assert!(!selected
        .iter()
        .any(|r| r["kind"] == "finding" && r["rule"] == "frontmatter-order"));
    let a = std::fs::read_to_string(root.join("a/note.md")).unwrap();
    assert!(a.starts_with("---\ntype: Note\nfirst: A\nsecond: B\ncustom: keep\n"));
    assert_eq!(
        std::fs::read_to_string(root.join("b/note.md")).unwrap(),
        original
    );
    let all = run(&["--fix", "--catalog-scope"]);
    assert!(all
        .iter()
        .any(|r| r["kind"] == "fix" && r["bundle"] == "test.b" && r["fixed"] == 1));
    assert!(!all
        .iter()
        .any(|r| r["kind"] == "finding" && r["rule"] == "frontmatter-order"));
    let b = std::fs::read_to_string(root.join("b/note.md")).unwrap();
    assert!(b.starts_with("---\ntype: Note\nsecond: B\nfirst: A\ncustom: keep\n"));
    assert_eq!(
        std::fs::read_to_string(root.join("off/note.md")).unwrap(),
        original
    );
    let old = okf_core::parse::parse_concept(
        okf_core::model::concept::ConceptId::from_relative("note"),
        original,
    )
    .unwrap();
    for text in [&a, &b] {
        let new = okf_core::parse::parse_concept(old.id.clone(), text).unwrap();
        assert_eq!(new.body, old.body);
        assert_eq!(
            new.frontmatter.get("verified"),
            old.frontmatter.get("verified")
        );
    }
    assert!(run(&["--fix", "--scope-bundle", "test.b"])
        .iter()
        .filter(|r| r["kind"] == "fix")
        .all(|r| r["fixed"] == 0));
}
