//! End-to-end input, preview and compatibility checks for structured concept authoring.
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn cli(dir: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_okf"))
        .current_dir(dir)
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args(args)
        .output()
        .unwrap()
}
fn success(output: &Output) {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
fn read_fm(dir: &std::path::Path, path: &str) -> serde_yaml::Value {
    let text = std::fs::read_to_string(dir.join(path)).unwrap();
    serde_yaml::from_str(text.split("---").nth(1).unwrap()).unwrap()
}
fn setup() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("onboarding.md"), "---\ntype: Procedure\ndeadline: {within: 24, unit: hours}\ncustom: {keep: true}\n---\n# Onboarding\n").unwrap();
    dir
}

#[test]
fn add_typed_values_from_file_paths_and_legacy_literal_dots() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("norms.yaml"),
        "- bearer: /teams/platform.md\n  action: review-access\n",
    )
    .unwrap();
    let output = cli(
        dir.path(),
        &[
            "add",
            "procedures/onboarding",
            "--type",
            "Procedure",
            "--set-yaml",
            "norms=@norms.yaml",
            "--set-path",
            "deadline.within=72",
            "--set-path",
            "deadline.unit=hours",
            "--set",
            "deadline.within=literal",
            "--set-yaml",
            "version='123'",
        ],
    );
    success(&output);
    let fm = read_fm(dir.path(), "procedures/onboarding.md");
    assert_eq!(fm["norms"][0]["action"].as_str(), Some("review-access"));
    assert_eq!(fm["deadline"]["within"].as_i64(), Some(72));
    assert_eq!(fm["deadline.within"].as_str(), Some("literal"));
    assert_eq!(fm["version"].as_str(), Some("123"));
}

#[test]
fn edit_object_paths_delete_siblings_and_support_literal_equals_keys() {
    let dir = setup();
    let output = cli(
        dir.path(),
        &[
            "edit",
            "onboarding",
            "--set-path",
            "deadline.within=72",
            "--unset-path",
            "deadline.unit",
            "--set-path",
            r#"["a=b"].nested={enabled: true}"#,
        ],
    );
    success(&output);
    let fm = read_fm(dir.path(), "onboarding.md");
    assert_eq!(fm["deadline"]["within"].as_i64(), Some(72));
    assert!(fm["deadline"].as_mapping().unwrap().get("unit").is_none());
    assert_eq!(fm["a=b"]["nested"]["enabled"].as_bool(), Some(true));
    assert_eq!(fm["custom"]["keep"].as_bool(), Some(true));
}

#[test]
fn inline_block_patch_and_file_patch_apply_to_frontmatter() {
    let dir = setup();
    success(&cli(dir.path(), &["edit", "onboarding", "--patch", "- op: replace\n  path: /deadline/within\n  value: 72\n- op: add\n  path: /norms\n  value: []\n"]));
    std::fs::write(dir.path().join("patch.yaml"), "- op: add\n  path: /norms/-\n  value: {bearer: /teams/platform.md, action: review}\n- op: test\n  path: /deadline/within\n  value: 72.0\n").unwrap();
    success(&cli(
        dir.path(),
        &["edit", "onboarding", "--patch", "@patch.yaml"],
    ));
    let fm = read_fm(dir.path(), "onboarding.md");
    assert_eq!(fm["norms"][0]["action"].as_str(), Some("review"));
}

#[test]
fn add_and_edit_previews_validate_and_show_diff_without_writes() {
    let dir = setup();
    let before = std::fs::read(dir.path().join("onboarding.md")).unwrap();
    let output = cli(
        dir.path(),
        &[
            "edit",
            "onboarding",
            "--set-path",
            "deadline.within=72",
            "--dry-run",
            "--json",
        ],
    );
    success(&output);
    let record: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(record["kind"], "change");
    assert_eq!(record["op"], "edit");
    assert_eq!(record["dry_run"], true);
    assert!(record["diff"].as_str().unwrap().contains("+  within: 72"));
    assert_eq!(
        std::fs::read(dir.path().join("onboarding.md")).unwrap(),
        before
    );
    let output = cli(
        dir.path(),
        &[
            "add",
            "new/onboarding",
            "--type",
            "Procedure",
            "--set-yaml",
            "deadline={within: 72}",
            "--dry-run",
        ],
    );
    success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("--- /dev/null"));
    assert!(!dir.path().join("new").exists());
    for args in [
        vec![
            "edit",
            "onboarding",
            "--patch",
            "[{op: remove, path: /type}]",
            "--dry-run",
        ],
        vec![
            "add",
            "new/onboarding",
            "--type",
            "Procedure",
            "--set-yaml",
            "type=[]",
            "--dry-run",
        ],
    ] {
        assert!(!cli(dir.path(), &args).status.success());
    }
    assert!(!dir.path().join("new").exists());
}

#[test]
fn malformed_or_unsupported_yaml_and_patch_errors_leave_file_unchanged() {
    let dir = setup();
    let before = std::fs::read(dir.path().join("onboarding.md")).unwrap();
    for value in [
        "custom={key: 1, key: 2}",
        "custom={1: value}",
        "custom=&anchor [a]",
        "custom=!tag value",
        "custom={key: .inf}",
        "custom=one\n---\ntwo",
    ] {
        assert!(
            !cli(dir.path(), &["edit", "onboarding", "--set-yaml", value])
                .status
                .success(),
            "{value}"
        );
        assert_eq!(
            std::fs::read(dir.path().join("onboarding.md")).unwrap(),
            before
        );
    }
    for operations in [
        "[{op: replace, path: /deadline/within, value: 72}, {op: test, path: /deadline/unit, value: days}]",
        "[{op: remove, path: /type}]",
        "[{op: replace, path: /deadline/within, value: 72}, {op: remove, path: /missing}]",
    ] {
        assert!(!cli(dir.path(), &["edit", "onboarding", "--patch", operations]).status.success());
        assert_eq!(std::fs::read(dir.path().join("onboarding.md")).unwrap(), before);
    }
}

#[test]
fn conflicting_structured_and_legacy_operations_fail_atomically() {
    let dir = setup();
    let before = std::fs::read(dir.path().join("onboarding.md")).unwrap();
    for args in [
        vec![
            "--set-yaml",
            "deadline={within: 72}",
            "--set-path",
            "deadline.within=48",
        ],
        vec!["--set-path", "deadline.within=72", "--unset", "deadline"],
        vec![
            "--set-path",
            "deadline.within=72",
            "--unset-path",
            "deadline.within",
        ],
        vec!["--set-yaml", "custom={}", "--set", "custom=text"],
        vec![
            "--patch",
            "[{op: replace, path: /deadline/within, value: 72}]",
            "--set",
            "deadline=literal",
        ],
        vec![
            "--set-yaml",
            "sources=[]",
            "--add-source",
            "resource=/source.md",
        ],
    ] {
        let mut command = vec!["edit", "onboarding"];
        command.extend(args);
        let output = cli(dir.path(), &command);
        assert!(!output.status.success(), "{command:?}");
        assert_eq!(
            std::fs::read(dir.path().join("onboarding.md")).unwrap(),
            before
        );
    }
}

#[test]
fn typed_stdin_values_work_and_multiple_consumers_are_rejected_before_reading() {
    let dir = setup();
    let mut child = Command::new(env!("CARGO_BIN_EXE_okf"))
        .current_dir(dir.path())
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args(["edit", "onboarding", "--set-yaml", "norms=-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"- {action: review}\n")
        .unwrap();
    success(&child.wait_with_output().unwrap());
    assert_eq!(
        read_fm(dir.path(), "onboarding.md")["norms"][0]["action"].as_str(),
        Some("review")
    );
    for args in [
        vec!["--set-yaml", "a=-", "--set-path", "b.nested=-"],
        vec!["--set-body", "-", "--set-yaml", "a=-"],
        vec!["--append-section", "Onboarding", "-", "--patch", "-"],
        vec!["--add-source-json", "-", "--set-yaml", "a=-"],
    ] {
        let mut command = vec!["edit", "onboarding"];
        command.extend(args);
        let output = cli(dir.path(), &command);
        assert!(!output.status.success(), "{command:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("only one input"));
    }
}
