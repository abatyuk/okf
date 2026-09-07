//! Workspace automation. `cargo xtask <task>`.
//!
//! Tasks:
//!   docs            Regenerate the schema-derived docs (writes files).
//!   docs --check    Verify they are up to date; exit 1 if regeneration would change anything.
//!   build [args…]   `cargo build [args…]`, then regenerate the docs.
//!   install         `cargo install --path crates/okf-cli --force`, then regenerate the docs.
//!
//! `build`/`install` exist because `okf schema` (the source the docs derive from) only exists
//! once the CLI is compiled — a `build.rs` cannot run the not-yet-built binary, and would
//! deadlock re-entering cargo. So the regenerate step is bolted onto the build/install commands
//! here instead: use `cargo xtask install` (not bare `cargo install`) to keep references current.
//!
//! "Schema-derived docs" = the CLI argument reference bundled into every plugin skill
//! (`skills/*/okf-cli-reference.md`) and the `## Arguments` section of every command
//! concept (`knowledge/commands/*.md`). Both are generated from `okf schema --json`, so this is
//! the single command that keeps them in lockstep with the CLI. Type `## Schema` sections are
//! curated (they mix real Rust items with serialized-shape docs) and are not regenerated here.
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const GROUP_ORDER: &[&str] = &["meta", "query", "check", "mutate", "render"];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let task = args.first().map(String::as_str);
    let check = args.iter().any(|a| a == "--check");
    let root = repo_root();
    match task {
        Some("docs") => run_docs(check),
        Some("build") => {
            let mut cargo_args = vec!["build"];
            cargo_args.extend(args[1..].iter().map(String::as_str));
            cargo(&root, &cargo_args);
            run_docs(false);
        }
        Some("install") => {
            cargo(&root, &["install", "--path", "crates/okf-cli", "--force"]);
            run_docs(false);
        }
        _ => {
            eprintln!("usage: cargo xtask <docs [--check] | build [args…] | install>");
            std::process::exit(2);
        }
    }
}

/// Run a cargo subcommand from `root`, inheriting stdio; exit on failure.
fn cargo(root: &Path, args: &[&str]) {
    let status = Command::new(env!("CARGO"))
        .args(args)
        .current_dir(root)
        .status()
        .expect("spawn cargo");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn run_docs(check: bool) {
    let root = repo_root();
    let records = schema_records(&root);
    let header = records
        .iter()
        .find(|r| r["kind"] == "schema")
        .expect("schema header record");
    let commands: Vec<&Value> = records.iter().filter(|r| r["kind"] == "command").collect();

    // Collect (path, desired-content) for every generated artifact.
    let mut artifacts: Vec<(PathBuf, String)> = Vec::new();

    // 1. The CLI reference, copied into each skill directory.
    let reference = cli_reference(header, &commands);
    for skill in skill_dirs(&root) {
        artifacts.push((skill.join("okf-cli-reference.md"), reference.clone()));
    }

    // 2. The `## Arguments` section of each command concept.
    for cmd in &commands {
        let stem = cmd["name"].as_str().unwrap().replace(' ', "-");
        let path = root.join("knowledge/commands").join(format!("{stem}.md"));
        if !path.exists() {
            eprintln!("warning: no command concept for {stem}");
            continue;
        }
        let body = std::fs::read_to_string(&path).unwrap();
        let block = format!(
            "{}\n\nEvery command also accepts global `--json` and an optional trailing `bundle` \
             positional.\n\nOutput stream: `{}`.",
            args_table(cmd),
            cmd["output"]["stream"].as_str().unwrap_or("")
        );
        artifacts.push((path, replace_section(&body, "Arguments", &block)));
    }

    if check {
        let stale: Vec<&PathBuf> = artifacts
            .iter()
            .filter(|(p, want)| std::fs::read_to_string(p).ok().as_deref() != Some(want.as_str()))
            .map(|(p, _)| p)
            .collect();
        if stale.is_empty() {
            println!("docs: up to date ({} artifacts)", artifacts.len());
        } else {
            eprintln!("docs: {} stale artifact(s); run `cargo xtask docs`:", stale.len());
            for p in stale {
                eprintln!("  {}", rel(&root, p));
            }
            std::process::exit(1);
        }
    } else {
        for (path, content) in &artifacts {
            std::fs::write(path, content).unwrap();
        }
        println!("docs: wrote {} artifacts", artifacts.len());
    }
}

/// The workspace root (this crate lives at `<root>/xtask`).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a parent")
        .to_path_buf()
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root).unwrap_or(p).display().to_string()
}

/// The plugin skill directories that bundle the CLI reference (those containing a `SKILL.md`).
fn skill_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root.join("skills"))
        .expect("skills dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.join("SKILL.md").exists())
        .collect();
    dirs.sort();
    dirs
}

/// Run `okf schema --json` and parse the NDJSON stream.
fn schema_records(root: &Path) -> Vec<Value> {
    let out = Command::new(env!("CARGO"))
        .args(["run", "-q", "-p", "okf-cli", "--bin", "okf", "--", "schema", "--json"])
        .current_dir(root)
        .output()
        .expect("run okf schema");
    if !out.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&out.stderr));
        std::process::exit(1);
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("valid NDJSON from okf schema"))
        .collect()
}

// ---- rendering (mirrors the argument tables in the command concepts) ----

fn flag_display(a: &Value) -> String {
    let name = a["name"].as_str().unwrap();
    if a["kind"] == "positional" {
        return format!("`<{name}>`");
    }
    let flag = format!("--{}", name.replace('_', "-"));
    let vns: Vec<&str> = a["value_names"]
        .as_array()
        .map(|v| v.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    if !vns.is_empty() {
        let parts: Vec<String> = vns.iter().map(|v| format!("<{v}>")).collect();
        return format!("`{flag} {}`", parts.join(" "));
    }
    if a["type"] == "bool" {
        format!("`{flag}`")
    } else {
        format!("`{flag} <value>`")
    }
}

fn arg_type(a: &Value) -> String {
    if a["kind"] == "positional" {
        "positional".into()
    } else {
        a["type"].as_str().unwrap_or("string").into()
    }
}

fn arg_desc(a: &Value) -> String {
    let mut d = a["help"].as_str().unwrap_or("").trim().to_string();
    if let Some(def) = a["default"].as_str() {
        if !def.is_empty() {
            d.push_str(&format!(" (default: `{def}`)"));
        }
    }
    if d.is_empty() {
        "—".into()
    } else {
        d
    }
}

fn args_table(cmd: &Value) -> String {
    let args = cmd["args"].as_array().cloned().unwrap_or_default();
    if args.is_empty() {
        return "_No arguments._".into();
    }
    let mut rows = vec![
        "| Argument | Type | Required | Description |".to_string(),
        "|----------|------|----------|-------------|".to_string(),
    ];
    for a in &args {
        let req = if a["required"].as_bool().unwrap_or(false) { "yes" } else { "no" };
        rows.push(format!(
            "| {} | {} | {} | {} |",
            flag_display(a),
            arg_type(a),
            req,
            arg_desc(a)
        ));
    }
    rows.join("\n")
}

fn cli_reference(header: &Value, commands: &[&Value]) -> String {
    let mut out = String::new();
    out.push_str("# okf CLI — argument reference\n\n");
    out.push_str(&format!(
        "> **Generated** by `cargo xtask docs` from `okf schema --json` (tool {}, OKF spec {}). \
         Do not hand-edit; regenerate instead.\n\n",
        header["tool_version"].as_str().unwrap_or("?"),
        header["okf_spec"]
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "))
            .unwrap_or_default()
    ));
    out.push_str(
        "**For skills:** consult this file to learn a command's arguments. **Do not** run \
         `okf <cmd> --help` or `okf schema` first just to discover flags — they are all listed \
         here. Every command also accepts the global `--json` flag (NDJSON output) and takes an \
         optional trailing `bundle` positional that falls back to `$OKF_BUNDLE`, then the cwd.\n\n",
    );

    let mut groups: Vec<&str> = GROUP_ORDER.to_vec();
    for c in commands {
        let g = c["group"].as_str().unwrap_or("");
        if !groups.contains(&g) {
            groups.push(g);
        }
    }
    for g in groups {
        let mut in_group: Vec<&&Value> = commands.iter().filter(|c| c["group"] == g).collect();
        if in_group.is_empty() {
            continue;
        }
        in_group.sort_by_key(|c| c["name"].as_str().unwrap_or(""));
        out.push_str(&format!("## {g}\n\n"));
        for c in in_group {
            let mutates = if c["mutates"].as_bool().unwrap_or(false) { " · _mutates_" } else { "" };
            out.push_str(&format!("### `okf {}`{mutates}\n\n", c["name"].as_str().unwrap()));
            if let Some(s) = c["summary"].as_str() {
                if !s.is_empty() {
                    out.push_str(&format!("{}.\n\n", s.trim_end_matches('.')));
                }
            }
            out.push_str(&args_table(c));
            out.push_str(&format!(
                "\n\nOutput stream: `{}`.\n\n",
                c["output"]["stream"].as_str().unwrap_or("")
            ));
        }
    }
    format!("{}\n", out.trim_end())
}

/// Replace everything from `## <heading>` to EOF with `new_block`; append if the heading is
/// absent. Idempotent, and never touches the prose above the section.
fn replace_section(body: &str, heading: &str, new_block: &str) -> String {
    let marker = format!("\n## {heading}\n");
    let base = match body.find(&marker) {
        Some(i) => &body[..i],
        None => body,
    };
    format!(
        "{}\n\n## {heading}\n\n{}\n",
        base.trim_end_matches('\n'),
        new_block.trim_end_matches('\n')
    )
}
