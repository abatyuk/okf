use assert_cmd::prelude::*;
use std::process::Command;

fn okf() -> Command {
    Command::cargo_bin("okf").unwrap()
}

#[test]
fn raw_body_and_document_numbers_do_not_include_display_header() {
    let dir = tempfile::tempdir().unwrap();
    let body = "\n# Heading\n\nBody with --- inside.\n";
    std::fs::write(
        dir.path().join("c.md"),
        format!("---\ntype: Concept\ntitle: T\ndescription: D\n---\n{body}"),
    )
    .unwrap();
    let run = |flags: &[&str]| {
        let out = okf()
            .arg("show")
            .arg("c")
            .arg(dir.path())
            .args(flags)
            .output()
            .unwrap();
        assert!(out.status.success(), "{:?}", out);
        String::from_utf8(out.stdout).unwrap()
    };
    assert_eq!(run(&["--body"]), body);
    let json: serde_json::Value = serde_json::from_str(&run(&["--body", "--json"])).unwrap();
    assert_eq!(json["body"], body);
    assert_eq!(json["kind"], "body");
    let numbered = run(&["-n"]);
    assert!(numbered.starts_with("1: ---\n"));
    let heading = numbered.lines().find(|s| s.ends_with("# Heading")).unwrap();
    let number = heading.split_once(':').unwrap().0;
    assert_eq!(run(&["--lines", number]), format!("{heading}\n"));
    let range: serde_json::Value = serde_json::from_str(&run(&["-n", "--json"])).unwrap();
    assert_eq!(range["start"], 1);
    assert_eq!(range["lines"][0]["text"], "---");
    okf()
        .arg("show")
        .arg("c")
        .arg(dir.path())
        .args(["--body", "--lines", "1"])
        .assert()
        .failure();
}
