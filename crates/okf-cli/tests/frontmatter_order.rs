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
