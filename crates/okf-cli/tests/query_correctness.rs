use assert_cmd::prelude::*;
use serde_json::Value;
use std::process::{Command, Output};

fn run(root: &std::path::Path, args: &[&str]) -> Output {
    Command::cargo_bin("okf")
        .unwrap()
        .current_dir(root)
        .env_remove("OKF_BUNDLE")
        .env_remove("OKF_CATALOG_OVERRIDES")
        .args(args)
        .output()
        .unwrap()
}

fn record(output: Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn missing_standalone_reference_exits_one_and_structural_index_resolves() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("index.md"), "# Bundle\n").unwrap();
    let missing = run(root.path(), &["resolve", "/missing.md", "--json"]);
    assert_eq!(missing.status.code(), Some(1));
    assert_eq!(record(missing)["exists"], false);
    let index = run(root.path(), &["resolve", "/index.md", "--json"]);
    assert!(index.status.success());
    assert_eq!(record(index)["exists"], true);
}

#[test]
fn human_projection_heading_matches_the_fields_rendered() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("note.md"),
        "---\ntype: Note\ntitle: Example\n---\n",
    )
    .unwrap();
    let output = run(
        root.path(),
        &["search", "--columns", "title", "--project", "type"],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "type\nNote\n");
}

#[test]
fn malformed_computation_contracts_fail_inspection() {
    let root = tempfile::tempdir().unwrap();
    for (metadata, body, expected) in [
        ("executor: nope\n", "# Computation\n```sql\nselect 1\n```\n", "executor"),
        ("attester: {}\n", "# Computation\n```sql\nselect 1\n```\n", "attester.resource"),
        ("parameters:\n- {name: x, type: string, required: true}\n- {name: x, type: string, required: false}\n", "# Computation\n```sql\nselect 1\n```\n", "duplicate"),
        ("", "# Computation\n```sql\n```\n", "empty"),
        ("", "# Computation\n```sql\nselect 1\n```\n```sql\nselect 2\n```\n", "single"),
    ] {
        std::fs::write(root.path().join("job.md"), format!("---\ntype: Attested Computation\nruntime: sql\n{metadata}---\n{body}")).unwrap();
        let output = run(root.path(), &["computation", "check", "job", "--json"]);
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        let value = record(output);
        assert_eq!(value["valid"], false);
        assert!(value["issues"].as_array().unwrap().iter().any(|v| v.as_str().unwrap().contains(expected)), "{value}");
    }
}

fn catalog() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for bundle in ["a", "b"] {
        std::fs::create_dir(root.path().join(bundle)).unwrap();
        std::fs::write(
            root.path().join(bundle).join("note.md"),
            "---\ntype: Note\ntitle: Shared ID\n---\n",
        )
        .unwrap();
        std::fs::write(root.path().join(bundle).join("index.md"), "# Bundle\n").unwrap();
    }
    std::fs::write(
        root.path().join("okf.toml"),
        "catalog = 'catalog.yaml'\ndefault_bundle = {id = 'acme.a'}\n",
    )
    .unwrap();
    std::fs::write(root.path().join("catalog.yaml"), "catalog_version: 1\nbundles:\n  acme.a: {location: {type: directory, path: a}}\n  acme.b: {location: {type: directory, path: b}}\n").unwrap();
    root
}

#[test]
fn human_scoped_results_identify_each_bundle_in_plain_and_projected_rows() {
    let root = catalog();
    for args in [
        vec!["list", "--catalog-scope"],
        vec!["search", "--catalog-scope", "--columns", "title"],
        vec!["search", "--catalog-scope", "--project", "title"],
    ] {
        let output = run(root.path(), &args);
        assert!(output.status.success(), "{output:?}");
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("acme.a\tworking-tree\t/note"), "{text}");
        assert!(text.contains("acme.b\tworking-tree\t/note"), "{text}");
    }
}

#[test]
fn catalog_structural_resolution_and_qualified_from_use_correct_bundle() {
    let root = catalog();
    for (link, from, id) in [
        ("/index.md", "/context", "/index"),
        ("note.md", "acme.b:/context", "/note"),
    ] {
        let output = run(
            root.path(),
            &["resolve", link, "--from", from, "--catalog-scope", "--json"],
        );
        assert!(output.status.success(), "{output:?}");
        let values: Vec<Value> = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let edge = values.iter().find(|v| v["kind"] == "bundle-edge").unwrap();
        assert_eq!(edge["status"], "resolved");
        assert_eq!(edge["target"]["id"], id);
        if from.starts_with("acme.b:") {
            assert_eq!(edge["target"]["bundle"], "acme.b");
        }
    }
}

#[cfg(feature = "url-sources")]
#[test]
fn fetched_artifact_honors_line_selection() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let resource = format!("http://{}/data.txt", listener.local_addr().unwrap());
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(10))
                }
                Err(e) => panic!("test HTTP client did not connect: {e}"),
            }
        };
        // Accepted sockets may inherit the listener's nonblocking mode on macOS.
        // Only acceptance is polled; request reads use the bounded blocking timeout.
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let mut request = [0u8; 4096];
        stream.read(&mut request).unwrap();
        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 14\r\nConnection: close\r\n\r\none\ntwo\nthree\n").unwrap();
    });
    let root = tempfile::tempdir().unwrap();
    let output = run(
        root.path(),
        &[
            "artifact", "show", &resource, "--fetch", "--lines", "2:2", "--json",
        ],
    );
    server
        .join()
        .unwrap_or_else(|_| panic!("test HTTP server failed; client output: {output:?}"));
    assert!(output.status.success(), "{output:?}");
    assert_eq!(record(output)["text"], "two");
}
