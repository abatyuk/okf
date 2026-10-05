use std::fs;
use std::path::Path;
use std::process::Command;

fn malformed_bundle() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.md"), "---\ntype: [\n---\n").unwrap();
    fs::write(root.path().join("z.md"), "---\ntype: 'unterminated\n---\n").unwrap();
    root
}
fn failure(root: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_okf"))
        .env_remove("OKF_BUNDLE")
        .args(args)
        .arg(root)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    String::from_utf8(out.stderr).unwrap()
}
#[test]
fn graph_reports_all_malformed_files_in_bundle_order() {
    let root = malformed_bundle();
    let message = failure(root.path(), &["graph"]);
    let a = root.path().join("a.md").display().to_string();
    let z = root.path().join("z.md").display().to_string();
    assert!(
        message.contains("2 concept files failed to parse"),
        "{message}"
    );
    assert!(message.find(&a).unwrap() < message.find(&z).unwrap());
    assert!(message.contains("line"));
}
#[test]
fn direct_read_reports_only_the_requested_failing_file() {
    let root = malformed_bundle();
    let message = failure(root.path(), &["show", "a"]);
    assert!(
        message.contains(&root.path().join("a.md").display().to_string()),
        "{message}"
    );
    assert!(!message.contains("z.md"));
}
#[test]
fn bounded_query_reports_failures_only_within_its_scan_budget() {
    let root = malformed_bundle();
    let message = failure(root.path(), &["search", "--scan-limit", "1"]);
    assert!(
        message.contains(&root.path().join("a.md").display().to_string()),
        "{message}"
    );
    assert!(!message.contains("z.md"));
}
#[test]
fn ontology_parse_failure_names_the_ontology_file() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("ontology.yaml"), "okf_ontology: [\n").unwrap();
    let message = failure(root.path(), &["graph"]);
    assert!(
        message.contains(&root.path().join("ontology.yaml").display().to_string()),
        "{message}"
    );
}
#[test]
fn historical_graph_reports_malformed_files_instead_of_unavailable_revision() {
    let root = malformed_bundle();
    for args in [
        vec!["init", "-q"],
        vec!["add", "--", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "Malformed fixture",
        ],
    ] {
        assert!(Command::new("git")
            .current_dir(root.path())
            .args(args)
            .status()
            .unwrap()
            .success());
    }
    let message = failure(root.path(), &["graph", "--revision", "HEAD"]);
    assert!(
        message.contains("a.md") && message.contains("z.md") && message.contains("revision"),
        "{message}"
    );
    let message = failure(
        root.path(),
        &["search", "--revision", "HEAD", "--scan-limit", "1"],
    );
    assert!(
        message.contains("a.md") && !message.contains("z.md"),
        "{message}"
    );
}
