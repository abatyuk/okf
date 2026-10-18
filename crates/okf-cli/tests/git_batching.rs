//! Git subprocess counts scale with repositories/batches, not source occurrences.
#![cfg(unix)]
use assert_cmd::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Fixture {
    temp: tempfile::TempDir,
    git: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let git = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|p| p.join("git"))
            .find(|p| p.is_file())
            .unwrap();
        for dir in ["knowledge", "planning", "tools", "src"] {
            std::fs::create_dir(root.join(dir)).unwrap();
        }
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(
            root.join("okf.toml"),
            "catalog = 'catalog.yaml'\ndefault_bundle = {id = 'test.notes'}\n",
        )
        .unwrap();
        std::fs::write(root.join("catalog.yaml"), "catalog_version: 1\nbundles:\n  test.notes:\n    location: {type: directory, path: knowledge}\n  test.plans:\n    location: {type: directory, path: planning}\n").unwrap();
        std::fs::write(
            root.join("knowledge/ontology.yaml"),
            "okf_ontology: '0.1'\nconcepts:\n  Note: {}\n",
        )
        .unwrap();
        std::fs::write(
            root.join("knowledge/target.md"),
            "---\ntype: Note\n---\nTarget\n",
        )
        .unwrap();
        for index in 0..20 {
            std::fs::write(root.join(format!("knowledge/n{index}.md")), "---\ntype: Note\nsources:\n- resource: knowledge/target.md\n  kind: git-path\n  fingerprint: {blob_sha: old}\n  bundle_ref:\n    id: test.notes\n    path: target.md\n    snapshot: {ref: HEAD}\n- resource: knowledge/target.md\n  kind: git-commit\n  fingerprint: {commit_sha: old}\n- {resource: src/main.rs, kind: git-path}\n---\nNeedle\n").unwrap();
        }
        let fixture = Self { temp, git };
        fixture.git(&["init", "-q"]);
        fixture.git(&["add", "."]);
        fixture.git(&[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.test",
            "commit",
            "-qm",
            "fixture",
        ]);
        let shim = fixture.root().join("tools/git");
        std::fs::write(&shim, "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$GIT_CALLS\"\nif [ -n \"$FAIL_DISCOVERY\" ] && [ \"$3\" = rev-parse ] && [ \"$4\" = --show-toplevel ]; then exit 1; fi\nexec \"$REAL_GIT\" \"$@\"\n").unwrap();
        std::fs::set_permissions(shim, std::fs::Permissions::from_mode(0o755)).unwrap();
        fixture
    }
    fn root(&self) -> &Path {
        self.temp.path()
    }
    fn git(&self, args: &[&str]) {
        let output = Command::new(&self.git)
            .current_dir(self.root())
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    }
    fn command(&self, cwd: &Path) -> Command {
        let mut command = Command::cargo_bin("okf").unwrap();
        command
            .current_dir(cwd)
            .env("REAL_GIT", &self.git)
            .env("GIT_CALLS", self.root().join("calls.log"))
            .env(
                "PATH",
                std::env::join_paths(
                    std::iter::once(self.root().join("tools"))
                        .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
                )
                .unwrap(),
            )
            .env_remove("OKF_BUNDLE")
            .env_remove("OKF_CATALOG_OVERRIDES")
            .env_remove("FAIL_DISCOVERY");
        command
    }
    fn calls(&self) -> Vec<String> {
        std::fs::read_to_string(self.root().join("calls.log"))
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}
fn success(output: &Output) {
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn historical_bundle_and_repeated_snapshots_share_one_object_reader() {
    for historical in [false, true] {
        let fixture = Fixture::new();
        let mut cmd = fixture.command(fixture.root());
        cmd.args([
            "backlinks",
            "/target",
            "--bundle-id",
            "test.notes",
            "--details",
            "--json",
        ]);
        if historical {
            cmd.args(["--revision", "HEAD"]);
        }
        let out = cmd.output().unwrap();
        success(&out);
        let records: Vec<serde_json::Value> = String::from_utf8(out.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let edges: Vec<_> = records
            .iter()
            .filter(|r| r["kind"] == "bundle-edge")
            .collect();
        assert_eq!(edges.len(), 40);
        assert!(edges
            .iter()
            .all(|edge| edge["fingerprint_status"] == "changed"));
        assert_eq!(
            edges
                .iter()
                .filter(|edge| edge["snapshot"]["status"] == "matched")
                .count(),
            20
        );
        let calls = fixture.calls();
        assert_eq!(
            calls
                .iter()
                .filter(|call| call.contains("cat-file --batch"))
                .count(),
            1,
            "{calls:?}"
        );
        assert!(
            !calls.iter().any(|call| call.contains(" show ")),
            "{calls:?}"
        );
        assert_eq!(
            calls
                .iter()
                .filter(|call| call.contains("hash-object"))
                .count(),
            1,
            "{calls:?}"
        );
        assert_eq!(
            calls.iter().filter(|call| call.contains(" log ")).count(),
            1,
            "{calls:?}"
        );
        let discoveries: Vec<_> = calls
            .iter()
            .filter(|call| call.contains("rev-parse --show-toplevel"))
            .collect();
        let unique: std::collections::BTreeSet<_> = discoveries.iter().collect();
        assert_eq!(discoveries.len(), unique.len(), "{calls:?}");
        assert!(calls.len() < 15, "{calls:?}");
    }
}

#[test]
fn standalone_affected_caches_discovery_success_and_failure() {
    for failure in [false, true] {
        let fixture = Fixture::new();
        let outside = tempfile::tempdir().unwrap();
        let mut cmd = fixture.command(outside.path());
        cmd.arg("affected")
            .arg(fixture.root().join("knowledge"))
            .arg("--changed")
            .arg("../src/main.rs")
            .arg("--json");
        if failure {
            cmd.env("FAIL_DISCOVERY", "1");
        }
        let out = cmd.output().unwrap();
        success(&out);
        assert_eq!(fixture.calls().len(), 1);
        assert_eq!(
            String::from_utf8(out.stdout).unwrap().lines().count(),
            if failure { 0 } else { 20 }
        );
    }
}

#[test]
fn refresh_batches_duplicate_sources_and_failed_discovery() {
    for failure in [false, true] {
        let fixture = Fixture::new();
        let mut text = "---\ntype: Note\nsources:\n".to_owned();
        for _ in 0..20 {
            text.push_str("- {resource: src/main.rs, kind: git-path}\n- {resource: src/main.rs, kind: git-commit}\n");
        }
        text.push_str("---\nBody\n");
        std::fs::write(fixture.root().join("knowledge/refresh.md"), text).unwrap();
        let mut cmd = fixture.command(fixture.root());
        cmd.args(["refresh", "/refresh", "--bundle-id", "test.notes", "--json"]);
        if failure {
            cmd.env("FAIL_DISCOVERY", "1");
        }
        let out = cmd.output().unwrap();
        success(&out);
        let record: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        let calls = fixture.calls();
        assert_eq!(
            calls
                .iter()
                .filter(|call| call.contains("rev-parse --show-toplevel"))
                .count(),
            1,
            "{calls:?}"
        );
        if failure {
            assert_eq!(calls.len(), 1);
            assert_eq!(record["skipped"].as_array().unwrap().len(), 40);
        } else {
            assert_eq!(
                calls
                    .iter()
                    .filter(|call| call.contains("hash-object --stdin-paths"))
                    .count(),
                1,
                "{calls:?}"
            );
            assert_eq!(
                calls.iter().filter(|call| call.contains(" log ")).count(),
                1,
                "{calls:?}"
            );
            assert_eq!(record["updated"].as_array().unwrap().len(), 40);
        }
    }
}

#[test]
fn historical_search_expansion_reuses_reader_for_targets_outside_scan_window() {
    let fixture = Fixture::new();
    let root = fixture.root();
    std::fs::write(root.join("knowledge/ontology.yaml"), "okf_ontology: '0.1'\nconcepts:\n  Note:\n    references:\n      next: {target: Note, cardinality: '0..n'}\n    relationships:\n      follows: {reference: next, kind: follows}\n").unwrap();
    std::fs::write(
        root.join("knowledge/n0.md"),
        "---\ntype: Note\nnext: [/target, /target2, /target3]\n---\nNeedle\n",
    )
    .unwrap();
    for name in ["target", "target2", "target3"] {
        std::fs::write(
            root.join(format!("knowledge/{name}.md")),
            format!("---\ntype: Note\ntitle: Historical {name}\n---\n"),
        )
        .unwrap();
    }
    fixture.git(&["add", "knowledge"]);
    fixture.git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.test",
        "commit",
        "-qm",
        "relationships",
    ]);
    for name in ["target", "target2", "target3"] {
        std::fs::write(
            root.join(format!("knowledge/{name}.md")),
            "---\ntype: Note\ntitle: Wrong working tree\n---\n",
        )
        .unwrap();
    }
    let out = fixture
        .command(root)
        .args([
            "search",
            "--text",
            "Needle",
            "--bundle-id",
            "test.notes",
            "--revision",
            "HEAD",
            "--scan-limit",
            "1",
            "--expand",
            "follows",
            "--json",
        ])
        .output()
        .unwrap();
    success(&out);
    let records: Vec<serde_json::Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let related: Vec<_> = records
        .iter()
        .filter(|row| row["kind"] == "related-concept")
        .collect();
    assert_eq!(related.len(), 3, "{records:?}");
    assert!(
        related
            .iter()
            .all(|row| row.to_string().contains("Historical")),
        "{related:?}"
    );
    let calls = fixture.calls();
    assert_eq!(
        calls
            .iter()
            .filter(|call| call.contains("cat-file --batch"))
            .count(),
        1,
        "{calls:?}"
    );
    assert!(
        !calls.iter().any(|call| call.contains(" show ")),
        "{calls:?}"
    );
    assert!(calls.len() < 15, "{calls:?}");
}
