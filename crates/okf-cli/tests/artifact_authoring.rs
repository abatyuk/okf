//! Regression coverage for artifact authoring and bundle-path diagnostics.
use assert_cmd::prelude::*;
use predicates::str::contains;
use std::path::Path;
use std::process::Command;

fn okf(cwd: &Path) -> Command {
    let mut command = Command::cargo_bin("okf").unwrap();
    command.current_dir(cwd).env_remove("OKF_BUNDLE");
    command
}

#[test]
fn show_cwd_relative_path_explains_bundle_resolution_and_suggests_the_path() {
    let project = tempfile::tempdir().unwrap();
    let bundle = project.path().join("b");
    std::fs::create_dir_all(bundle.join("contracts/x/references")).unwrap();
    std::fs::write(bundle.join("contracts/x/references/s.json"), "{\"a\":1}\n").unwrap();
    okf(project.path())
        .args(["artifact", "show", "b/contracts/x/references/s.json", "b"])
        .assert()
        .code(2)
        .stderr(contains("paths are resolved relative to the bundle root"))
        .stderr(contains("did you mean contracts/x/references/s.json?"));
    okf(project.path())
        .args(["artifact", "show", "contracts/x/references/s.json", "b"])
        .assert()
        .success()
        .stdout("{\"a\":1}\n");
    okf(project.path())
        .args([
            "artifact",
            "show",
            "references/s.json",
            "b",
            "--from",
            "/contracts/x/c",
        ])
        .assert()
        .success()
        .stdout("{\"a\":1}\n");
}

#[test]
fn list_subdirectory_positionals_point_to_directory_flag() {
    let project = tempfile::tempdir().unwrap();
    okf(project.path())
        .args(["init", "knowledge"])
        .assert()
        .success();
    let bundle = project.path().join("knowledge");
    std::fs::create_dir_all(bundle.join("contracts/x/references")).unwrap();
    std::fs::write(bundle.join("contracts/x/references/s.json"), "{}\n").unwrap();
    okf(&bundle)
        .args(["artifact", "list", "contracts/x"])
        .assert()
        .code(2)
        .stderr(contains("use --directory contracts/x"));
    std::fs::write(project.path().join("okf.toml"), "bundle = \"knowledge\"\n").unwrap();
    okf(project.path())
        .args(["artifact", "list", "contracts/x"])
        .assert()
        .code(2)
        .stderr(contains("use --directory contracts/x"));
    okf(project.path())
        .args(["artifact", "list", "knowledge/contracts/x"])
        .assert()
        .code(2)
        .stderr(contains("use --directory contracts/x"));
    okf(project.path())
        .args(["artifact", "list", "--directory", "contracts/x"])
        .assert()
        .success()
        .stdout(contains("contracts/x/references/s.json"));
}

#[test]
fn put_creates_binary_artifacts_and_requires_explicit_replacement() {
    let project = tempfile::tempdir().unwrap();
    okf(project.path()).args(["init", "b"]).assert().success();
    let input = project.path().join("input.bin");
    std::fs::write(&input, [0, 159, 255]).unwrap();
    okf(project.path())
        .args([
            "artifact",
            "put",
            "contracts/x/references/s.bin",
            "@input.bin",
            "b",
            "--create-only",
            "--json",
        ])
        .assert()
        .success()
        .stdout(contains("\"kind\":\"artifact-write\""));
    let target = project.path().join("b/contracts/x/references/s.bin");
    assert_eq!(std::fs::read(&target).unwrap(), [0, 159, 255]);
    std::fs::write(&input, "new\n").unwrap();
    okf(project.path())
        .args([
            "artifact",
            "put",
            "contracts/x/references/s.bin",
            "@input.bin",
            "b",
        ])
        .assert()
        .code(2)
        .stderr(contains("use --replace"));
    assert_eq!(std::fs::read(&target).unwrap(), [0, 159, 255]);
    okf(project.path())
        .args([
            "artifact",
            "put",
            "contracts/x/references/s.bin",
            "@input.bin",
            "b",
            "--replace",
            "--json",
        ])
        .assert()
        .success()
        .stdout(contains("\"replaced\":true"));
    assert_eq!(std::fs::read(&target).unwrap(), b"new\n");
    okf(project.path())
        .args([
            "artifact",
            "put",
            "x.json",
            "@input.bin",
            "b",
            "--replace",
            "--create-only",
        ])
        .assert()
        .code(2);
}

#[test]
fn put_preserves_citing_concepts_and_exposes_source_drift() {
    let project = tempfile::tempdir().unwrap();
    okf(project.path()).args(["init", "b"]).assert().success();
    std::fs::write(project.path().join("input.json"), "{\"a\":1}\n").unwrap();
    okf(project.path())
        .args([
            "artifact",
            "put",
            "contracts/x/references/s.json",
            "@input.json",
            "b",
        ])
        .assert()
        .success();
    okf(project.path())
        .args([
            "add",
            "contracts/x/c",
            "b",
            "--type",
            "Concept",
            "--title",
            "T",
            "--description",
            "D",
            "--add-source-json",
            "{\"resource\":\"contracts/x/references/s.json\",\"kind\":\"file\"}",
        ])
        .assert()
        .success();
    okf(project.path())
        .args(["refresh", "/contracts/x/c", "b"])
        .assert()
        .success();
    let concept_path = project.path().join("b/contracts/x/c.md");
    let original = std::fs::read(&concept_path).unwrap();
    std::fs::write(project.path().join("input.json"), "{\"a\":2}\n").unwrap();
    okf(project.path())
        .args([
            "artifact",
            "put",
            "contracts/x/references/s.json",
            "@input.json",
            "b",
            "--replace",
        ])
        .assert()
        .success();
    assert_eq!(std::fs::read(&concept_path).unwrap(), original);
    okf(project.path())
        .args(["stale", "b", "--fail-on", "any"])
        .assert()
        .code(1)
        .stdout(contains("contracts/x/c"))
        .stdout(contains("drifted"));
}

#[test]
fn put_rejects_concepts_control_paths_and_escape() {
    let project = tempfile::tempdir().unwrap();
    okf(project.path()).args(["init", "b"]).assert().success();
    std::fs::write(project.path().join("input"), "replacement").unwrap();
    for path in [
        "concept.md",
        "../outside.json",
        ".git/config",
        "okf.toml",
        "ontology.yaml",
        "https://example.com/a.json",
    ] {
        okf(project.path())
            .args(["artifact", "put", path, "@input", "b", "--replace"])
            .assert()
            .code(2);
    }
    assert!(!project.path().join("outside.json").exists());
    assert!(!project.path().join("b/concept.md").exists());
    // Reserved Markdown remains supported.
    okf(project.path())
        .args(["artifact", "put", "log.md", "@input", "b"])
        .assert()
        .success();
    assert_eq!(
        std::fs::read(project.path().join("b/log.md")).unwrap(),
        b"replacement"
    );
}

#[cfg(unix)]
#[test]
fn put_rejects_symlink_files_and_parents_including_dangling_links() {
    use std::os::unix::fs::symlink;
    let project = tempfile::tempdir().unwrap();
    okf(project.path()).args(["init", "b"]).assert().success();
    std::fs::write(project.path().join("input"), "new").unwrap();
    std::fs::write(project.path().join("outside.json"), "original").unwrap();
    symlink(
        project.path().join("outside.json"),
        project.path().join("b/link.json"),
    )
    .unwrap();
    symlink(
        project.path().join("missing.json"),
        project.path().join("b/dangling.json"),
    )
    .unwrap();
    symlink(project.path(), project.path().join("b/escape")).unwrap();
    for path in [
        "link.json",
        "dangling.json",
        "escape/outside.json",
        "escape/new.json",
    ] {
        okf(project.path())
            .args(["artifact", "put", path, "@input", "b", "--replace"])
            .assert()
            .code(2)
            .stderr(contains("without symlinks"));
    }
    assert_eq!(
        std::fs::read(project.path().join("outside.json")).unwrap(),
        b"original"
    );
    assert!(!project.path().join("missing.json").exists());
    assert!(!project.path().join("new.json").exists());
}
