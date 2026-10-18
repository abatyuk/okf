//! Catalog graph construction must discover each bundle's Git root only once.
#![cfg(unix)]
use assert_cmd::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[test]
fn git_root_discovery_is_cached_per_bundle_including_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for dir in ["knowledge", "planning", "tools"] {
        std::fs::create_dir(root.join(dir)).unwrap();
    }
    std::fs::write(
        root.join("okf.toml"),
        "catalog = 'catalog.yaml'\ndefault_bundle = {id = 'test.notes'}\n",
    )
    .unwrap();
    std::fs::write(root.join("catalog.yaml"), "catalog_version: 1\nbundles:\n  test.notes:\n    location: {type: directory, path: knowledge}\n  test.plans:\n    location: {type: directory, path: planning}\n").unwrap();
    std::fs::write(root.join("knowledge/target.md"), "---\ntype: Note\n---\n").unwrap();
    for bundle in ["knowledge", "planning"] {
        for index in 0..4 {
            std::fs::write(root.join(format!("{bundle}/n{index}.md")), "---\ntype: Note\nsources:\n- {resource: src/main.rs, kind: git-path}\n- {resource: knowledge/target.md, kind: git-path}\n- {resource: src/other.rs, kind: git-commit}\n---\n").unwrap();
        }
    }
    let shim = root.join("tools/git");
    std::fs::write(&shim, "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$DISCOVERY_LOG\"\nif [ -n \"$DISCOVERY_FAIL\" ]; then exit 1; fi\nprintf '%s\\n' \"$DISCOVERY_ROOT\"\n").unwrap();
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
    let paths = std::env::join_paths(std::iter::once(root.join("tools")).chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .unwrap();
    for failed in [false, true] {
        let log = root.join(if failed { "failed.log" } else { "success.log" });
        let output = Command::cargo_bin("okf")
            .unwrap()
            .current_dir(root)
            .env("PATH", &paths)
            .env("DISCOVERY_LOG", &log)
            .env("DISCOVERY_ROOT", root.canonicalize().unwrap())
            .env("DISCOVERY_FAIL", if failed { "1" } else { "" })
            .env_remove("OKF_BUNDLE")
            .env_remove("OKF_CATALOG_OVERRIDES")
            .args([
                "backlinks",
                "/target",
                "--bundle-id",
                "test.notes",
                "--catalog-scope",
                "--details",
                "--json",
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let calls = std::fs::read_to_string(&log).unwrap();
        assert_eq!(calls.lines().count(), 2, "{calls}");
        assert!(calls
            .lines()
            .all(|line| line.ends_with("rev-parse --show-toplevel")));
        let records: Vec<serde_json::Value> = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let edges: Vec<_> = records
            .iter()
            .filter(|v| v["kind"] == "bundle-edge")
            .collect();
        if !failed {
            assert_eq!(edges.len(), 8);
            assert!(edges.iter().all(|edge| edge["status"] == "resolved"));
            assert!(edges
                .iter()
                .any(|edge| edge["source"]["bundle"] == "test.plans"));
        } else {
            assert!(edges.is_empty());
        }
    }
}
