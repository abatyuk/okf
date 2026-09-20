//! Workspace automation. `cargo xtask <task>`.
//!
//! Tasks:
//!   docs            Regenerate the schema-derived docs (writes files).
//!   docs --check    Verify they are up to date; exit 1 if regeneration would change anything.
//!   skills          Validate authored skills against the current CLI schema and generated docs.
//!   build [args…]   `cargo build [args…]`, then regenerate the docs.
//!   install         `cargo install --path crates/okf-cli --force`, then regenerate the docs.
//!
//! `build`/`install` exist because `okf schema` (the source the docs derive from) only exists
//! once the CLI is compiled — a `build.rs` cannot run the not-yet-built binary, and would
//! deadlock re-entering cargo. So the regenerate step is bolted onto the build/install commands
//! here instead: use `cargo xtask install` (not bare `cargo install`) to keep references current.
//!
//! "Schema-derived docs" = one complete developer CLI reference
//! (`docs/okf-cli-reference.md`), focused per-skill references
//! (`plugins/okf/skills/*/references/cli.md`), and the `## Arguments` section of every command
//! concept (`knowledge/commands/*.md`). All three surfaces are generated from
//! `okf schema --json`, so this is the single command that keeps them in lockstep with the CLI.
//! Type `## Schema` sections are curated (they mix real Rust items with serialized-shape docs)
//! and are not regenerated here.
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
        Some("skills") => {
            run_skill_checks();
            run_docs(true);
        }
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
            eprintln!("usage: cargo xtask <docs [--check] | skills | build [args…] | install>");
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
    let scenarios = skill_scenarios();
    let header = records
        .iter()
        .find(|r| r["kind"] == "schema")
        .expect("schema header record");
    let commands: Vec<&Value> = records.iter().filter(|r| r["kind"] == "command").collect();

    // Collect (path, desired-content) for every generated artifact.
    let mut artifacts: Vec<(PathBuf, String)> = Vec::new();

    // 1. One complete developer reference and one focused reference per skill. Keeping each
    // skill's subset local makes the plugin portable without loading unrelated command docs.
    artifacts.push((
        root.join("docs/okf-cli-reference.md"),
        cli_reference(header, &commands, None),
    ));
    for skill in skill_dirs(&root) {
        let name = skill
            .file_name()
            .and_then(|value| value.to_str())
            .expect("UTF-8 skill directory name");
        let selected = commands_for_skill(name, &scenarios, &commands)
            .unwrap_or_else(|error| panic!("{error}"));
        artifacts.push((
            skill.join("references/cli.md"),
            cli_reference(header, &selected, Some(name)),
        ));
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
        let has_bundle = cmd["args"]
            .as_array()
            .is_some_and(|args| args.iter().any(|arg| arg["name"] == "bundle"));
        let invocation_note = if cmd["name"] == "schema" {
            "This command is always NDJSON; `--json` is accepted but unnecessary."
        } else if has_bundle {
            "Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves \
             as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd."
        } else {
            "Global `--json` requests NDJSON."
        };
        let block = format!(
            "{}\n\n{}\n\nOutput stream: `{}`.",
            args_table(cmd),
            invocation_note,
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
            eprintln!(
                "docs: {} stale artifact(s); run `cargo xtask docs`:",
                stale.len()
            );
            for p in stale {
                eprintln!("  {}", rel(&root, p));
            }
            std::process::exit(1);
        }
    } else {
        for (path, content) in &artifacts {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
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
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root.join("plugins/okf/skills"))
        .expect("skills dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.join("SKILL.md").exists())
        .collect();
    dirs.sort();
    dirs
}

fn skill_scenarios() -> Value {
    serde_json::from_str(include_str!("../skill-scenarios.json"))
        .expect("valid skill scenario checklist JSON")
}

fn commands_for_skill<'a>(
    skill: &str,
    scenarios: &Value,
    commands: &[&'a Value],
) -> Result<Vec<&'a Value>, String> {
    let entry = scenarios
        .as_array()
        .and_then(|entries| entries.iter().find(|entry| entry["skill"] == skill))
        .ok_or_else(|| format!("{skill}: missing skill scenario checklist"))?;
    let names = entry["reference_commands"]
        .as_array()
        .ok_or_else(|| format!("{skill}: reference_commands must be an array"))?;
    let mut selected = Vec::new();
    let mut seen = Vec::new();
    for name in names {
        let name = name
            .as_str()
            .ok_or_else(|| format!("{skill}: reference command names must be strings"))?;
        if seen.contains(&name) {
            return Err(format!("{skill}: duplicate reference command `{name}`"));
        }
        seen.push(name);
        let command = commands
            .iter()
            .find(|command| command["name"] == name)
            .copied()
            .ok_or_else(|| format!("{skill}: unknown reference command `{name}`"))?;
        selected.push(command);
    }
    if selected.is_empty() {
        return Err(format!("{skill}: reference_commands must not be empty"));
    }
    Ok(selected)
}

/// Run `okf schema --json` and parse the NDJSON stream.
fn schema_records(root: &Path) -> Vec<Value> {
    let out = Command::new(env!("CARGO"))
        .args([
            "run", "-q", "-p", "okf-cli", "--bin", "okf", "--", "schema", "--json",
        ])
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
    let default = match &a["default"] {
        Value::String(value) if !value.is_empty() => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Array(values) if !values.is_empty() => Some(
            values
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(","),
        ),
        _ => None,
    };
    if let Some(default) = default {
        d.push_str(&format!(" (default: `{default}`)"));
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
        let req = if a["required"].as_bool().unwrap_or(false) {
            "yes"
        } else {
            "no"
        };
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

fn cli_reference(header: &Value, commands: &[&Value], skill: Option<&str>) -> String {
    let mut out = String::new();
    match skill {
        Some(name) => out.push_str(&format!("# okf CLI — {name} command reference\n\n")),
        None => out.push_str("# okf CLI — complete argument reference\n\n"),
    }
    out.push_str(&format!(
        "> **Generated** by `cargo xtask docs` from `okf schema --json` and curated usage notes (tool {}, OKF spec {}). \
         Do not hand-edit; regenerate instead.\n\n",
        header["tool_version"].as_str().unwrap_or("?"),
        header["okf_spec"]
            .as_array()
            .map(|a| a
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_default()
    ));
    if let Some(name) = skill {
        out.push_str(&format!(
            "This focused reference contains only commands selected for the `{name}` workflow. \
             Consult it when exact arguments or output shapes are needed. If the installed \
             `okf` version differs from the generated tool version above, or rejects documented \
             syntax, use that command's `--help` output as the runtime authority.\n\n"
        ));
    } else {
        out.push_str(
            "This complete reference is for developers and general CLI lookup. Skills carry \
             smaller generated subsets so they do not load unrelated commands.\n\n",
        );
    }
    out.push_str(
        "Commands with a human form accept global `--json` for NDJSON; `schema` is always \
         NDJSON. Bundle-aware commands take an optional trailing `bundle` positional resolved as \
         explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd. Meta commands have no \
         bundle, and `source-scan` takes an explicit arbitrary directory.\n\n\
         Exit codes: 0 means success under the selected failure threshold, not necessarily no \
         findings; 1 means findings or an unsuccessful resolution; 2 means usage errors; \
         3 means environment/I/O/YAML errors; 4 means an internal error. Inspect findings even with \
         `--fail-on never`. NDJSON is one record per line, not a JSON array.\n\n",
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
            let mutates = if !c["mutates_when"].is_null() {
                " · _conditionally mutates_"
            } else if c["mutates"].as_bool().unwrap_or(false) {
                " · _mutates_"
            } else {
                ""
            };
            out.push_str(&format!(
                "### `okf {}`{mutates}\n\n",
                c["name"].as_str().unwrap()
            ));
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
            let notes = command_guidance(c["name"].as_str().unwrap_or(""));
            if !notes.is_empty() {
                out.push_str(notes);
                out.push_str("\n\n");
            }
        }
    }
    format!("{}\n", out.trim_end())
}

/// Curated usage semantics alongside schema-derived arguments. Keep these notes focused on
/// observable behavior that cannot be inferred from the argument table.
fn command_guidance(name: &str) -> &'static str {
    match name {
        "show" => "Without `--json`, show includes the serialized frontmatter and full Markdown body. \
            Plain `show --json` returns metadata only: frontmatter plus `id`, `trust_tier`, \
            `effective_status`, `effective_generated_at`, `latest_verified_at`, and \
            `verification_current`. It does not include the body. `--outline --json` returns \
            `headings` with `line`, `level`, and `text`; `--lines START:END --json` returns \
            `start`, actual `end`, and `lines` containing `line` and `text`. Line numbers refer \
            to the serialized document, including frontmatter. An outline or selected slice \
            does not establish complete document-review coverage.",
        "list" => "JSON records contain frontmatter and computed lifecycle/trust metadata, not bodies. \
            Aggregate field occurrence counts from these records before opening prose. Inventory \
            is unbounded; scope or filter the output before loading a large result into context.",
        "search" => "The positional argument is the bundle, never query text. Text requires `--text`; \
            structured filters work without it. No filters means inventory. Text-search JSON \
            adds `search.score` and bounded `search.matches` to metadata records, not full bodies. \
            Structured-only search has no text-match evidence. `--in title,description` narrows \
            the default fields; adding `frontmatter` broadens them. Empty results exit successfully \
            and establish only that this query found no matches.",
        "artifact resolve" => "**Path namespaces:** a leading `/` means bundle-root-relative, not an \
            operating-system absolute path. Other local paths resolve against the declaring concept's \
            directory when `--from` is supplied, otherwise the bundle root. Keep `--from` on the \
            subsequent read too. JSON uses `artifact_kind` (concept, artifact, reserved, external, \
            scope, missing, or blocked), `path`, `exists`, `size`, and `message`. Missing/blocked \
            resolution exits 1. A scope descriptor is provenance, not a missing file.\n\n\
            **Repository sources:** with repo `/work/app`, bundle `/work/app/knowledge`, and declaring \
            concept `notes/service`, `/references/spec.txt` resolves to \
            `/work/app/knowledge/references/spec.txt`. A `kind=git-path` or `git-commit` source \
            `src/service.rs` is instead fingerprinted from the Git worktree root as \
            `/work/app/src/service.rs`; the artifact resolver does not reinterpret paths by source kind. \
            File/line-range/markdown-heading fingerprints use bundle-relative paths. Record the \
            actual source location and convention; do not assume fingerprint and artifact paths coincide. \
            Inspect external repository evidence with normal source tools within existing read \
            authorization. Do not rewrite provenance, bypass containment, or mirror files merely \
            to make artifact resolution succeed.",
        "artifact show" => "For document-relative paths, pass the same `--from` used during resolution. \
            JSON includes `text`, `binary`, `truncated`, `size`, `sha256`, and `path`. Binary files \
            provide metadata only. Inspect `truncated` before treating a read as complete; human \
            output alone does not expose this flag. Use bounded line windows and a sufficient byte \
            budget for the needed range; do not infer absence from a truncated result. Use `show` \
            for concepts and this command for opaque or reserved files.\n\n\
            Local artifact reads remain inside the canonical bundle after symlinks. For a URL, \
            `--fetch` requires a network-enabled build and authorization covering that source. \
            A user request to inspect a named source can supply that authorization; no separate \
            OKF policy file is specified by this command. If unavailable, use an authorized external \
            retrieval tool or report the evidence gap. Do not send ambient credentials. \
            Reading computation, executor, or attester code never authorizes execution.",
        "docs" => "`--format index` writes indexes throughout the bundle, replacing their bodies; \
            it does not merge curated prose. Use it only when all affected index bodies are generated \
            or replacement is already authorized. Preserve curated indexes and edit only necessary \
            links otherwise. Validate after writes. Other formats emit output rather than updating \
            indexes; the default is `md`.",
        "ontology add" | "ontology update" => "Built-in field types: `string`, `text`, `int`, `bool`, \
            `date`, `datetime`, `uri`, `enum`, `list`, `object`; custom type names must resolve in \
            the existing sidecar's `field_types`. Reference cardinalities are `0..1` (optional one), \
            `1..1` (exactly one), `0..n` (optional many), and `1..n` (at least one). For example, \
            `--field \"stage:enum:draft|active\"` declares choices and \
            `--ref \"depends_on:Service:0..n\"` permits zero or more Service links. Observed \
            presence alone does not justify a required rule. These flags describe advisory local \
            rules, not portable OKF conformance requirements.",
        "add" => "Creation scaffolds a concept; supply Markdown body afterward with `edit --set-body`. \
            `--generated-by` records the supplied actor and the CLI's current timestamp. Do not \
            invent a historical generation time or actor. For a requested computation only, \
            `--attested --runtime <runtime>` selects exact `Attested Computation`; declare actual \
            parameters with repeatable `--parameter name:type:required` (omit `:required` when optional). \
            Choose `--computation <resource>` or `--inline-computation @file`, then add reviewed \
            executor/receipt/attester fields as needed. These describe a contract and authorize no execution.",
        "edit" => "`--set` accepts scalar values, not arbitrary YAML objects; a dotted key is not a \
            nested-field update. Use `--add-source-json @file` for a complete source mapping. For \
            unsupported complex metadata preservation, inspect the existing representation and \
            use a narrow lossless file edit within scope, then validate. Do not flatten mappings \
            or fabricate verification. Meaningful edits remove active `verified` events and update \
            existing `generated.at`; preserve needed historical evidence separately. Body files \
            contain Markdown only, without frontmatter. Section flags take heading and text as \
            separate values. Malformed YAML must be repaired before this command can load it.",
        "refresh" => "Refresh updates supported source fingerprints across the concept, not content or \
            standard source modification dates. Review those sources against the final content first, \
            whether or not the content needed rewriting. JSON reports `updated`/`unchanged` counts and \
            `skipped` entries with reasons. `--fail-on skipped` exits 1 when any source was skipped; \
            successfully processed fingerprints may already have been written. Report unresolved \
            sources instead of treating refresh as proof of complete synchronization.",
        "computation check" => "Inspect-only: `execution: not-run` is not a passing runtime attestation. \
            Document verification and inspection of executable resources never establish a run verdict.",
        _ => "",
    }
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

/// Validate canonical skill metadata and every backticked `okf ...` command against the schema.
/// This deliberately checks executable snippets rather than prose wording.
fn run_skill_checks() {
    let root = repo_root();
    let records = schema_records(&root);
    let commands: Vec<&Value> = records.iter().filter(|r| r["kind"] == "command").collect();
    let scenarios = skill_scenarios();
    let mut errors = Vec::new();
    let dirs = skill_dirs(&root);

    for dir in &dirs {
        let path = dir.join("SKILL.md");
        let body = std::fs::read_to_string(&path).expect("read skill");
        let rel_path = rel(&root, &path);
        validate_frontmatter(dir, &body, &rel_path, &mut errors);

        let name = dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let reference_commands = match commands_for_skill(name, &scenarios, &commands) {
            Ok(selected) => selected
                .iter()
                .filter_map(|command| command["name"].as_str())
                .collect::<Vec<_>>(),
            Err(error) => {
                errors.push(error);
                Vec::new()
            }
        };

        for snippet in inline_code(&body).into_iter().filter(|code| {
            let mut words = code.split_whitespace();
            words.next() == Some("okf") && words.next().is_some()
        }) {
            validate_command(&snippet, &commands, &rel_path, &mut errors);
            if let Some(command) = command_name(&snippet, &commands) {
                if !reference_commands.contains(&command) {
                    errors.push(format!(
                        "{rel_path}: command `{command}` is not included in reference_commands"
                    ));
                }
            }
        }

        if body.contains("review-attest") {
            errors.push(format!(
                "{rel_path}: uses retired review-attest terminology"
            ));
        }
        if !body.contains("`references/`") {
            errors.push(format!(
                "{rel_path}: must state how the optional references/ convention is handled"
            ));
        }
        if !body.contains("`references/cli.md`") {
            errors.push(format!(
                "{rel_path}: must route exact CLI lookup to `references/cli.md`"
            ));
        }
        if dir.join("okf-cli-reference.md").exists() {
            errors.push(format!(
                "{rel_path}: legacy full-copy okf-cli-reference.md must be removed"
            ));
        }
    }

    let required = ["repair", "review-verify"];
    for name in required {
        if !root
            .join("plugins/okf/skills")
            .join(name)
            .join("SKILL.md")
            .exists()
        {
            errors.push(format!("missing canonical {name} skill"));
        }
    }
    if root
        .join("plugins/okf/skills/review-attest/SKILL.md")
        .exists()
    {
        errors.push("retired review-attest skill still exists".into());
    }
    validate_scenario_checklists(&root, &dirs, &scenarios, &mut errors);

    if errors.is_empty() {
        println!(
            "skills: valid ({} skills; command snippets match okf schema)",
            dirs.len()
        );
    } else {
        eprintln!("skills: {} error(s)", errors.len());
        for error in errors {
            eprintln!("  {error}");
        }
        std::process::exit(1);
    }
}

fn validate_scenario_checklists(
    root: &Path,
    dirs: &[PathBuf],
    scenarios: &Value,
    errors: &mut Vec<String>,
) {
    let entries = scenarios.as_array().expect("skill scenarios are an array");
    let mut seen = Vec::new();

    for entry in entries {
        let name = entry["skill"].as_str().unwrap_or("");
        if name.is_empty() || seen.contains(&name) {
            errors.push(format!(
                "skill scenario has empty or duplicate name `{name}`"
            ));
            continue;
        }
        seen.push(name);
        let path = root.join("plugins/okf/skills").join(name).join("SKILL.md");
        let Ok(body) = std::fs::read_to_string(&path) else {
            errors.push(format!("skill scenario references missing skill `{name}`"));
            continue;
        };

        let expected = entry["expected_commands"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if expected.is_empty() {
            errors.push(format!("{name}: scenario has no expected commands"));
        }
        let normalized_body = body.split_whitespace().collect::<Vec<_>>().join(" ");
        for command in expected.iter().filter_map(Value::as_str) {
            if !normalized_body.contains(command) {
                errors.push(format!(
                    "{name}: scenario expects command `{command}` to be taught"
                ));
            }
        }

        let prohibited = entry["prohibited_claims"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if prohibited.is_empty() {
            errors.push(format!("{name}: scenario has no prohibited claims"));
        }
        for claim in prohibited.iter().filter_map(Value::as_str) {
            if body.to_lowercase().contains(&claim.to_lowercase()) {
                errors.push(format!("{name}: contains prohibited claim `{claim}`"));
            }
        }

        if entry["mutation_boundary"]
            .as_str()
            .is_none_or(str::is_empty)
        {
            errors.push(format!("{name}: scenario lacks a mutation boundary"));
        }
        if entry["report_contains"]
            .as_array()
            .is_none_or(Vec::is_empty)
        {
            errors.push(format!("{name}: scenario lacks report requirements"));
        }
    }

    if entries.len() != dirs.len() {
        errors.push(format!(
            "skill scenarios cover {} entries but {} canonical skills exist",
            entries.len(),
            dirs.len()
        ));
    }
    for dir in dirs {
        let name = dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if !seen.contains(&name) {
            errors.push(format!("{name}: missing skill scenario checklist"));
        }
    }
}

fn validate_frontmatter(dir: &Path, body: &str, path: &str, errors: &mut Vec<String>) {
    let mut lines = body.lines();
    if lines.next() != Some("---") {
        errors.push(format!("{path}: missing opening frontmatter delimiter"));
        return;
    }
    let mut frontmatter = Vec::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if line == "---" {
            closed = true;
            break;
        }
        frontmatter.push(line);
    }
    if !closed {
        errors.push(format!("{path}: missing closing frontmatter delimiter"));
        return;
    }
    let name = frontmatter
        .iter()
        .find_map(|line| line.strip_prefix("name:").map(str::trim));
    let description = frontmatter
        .iter()
        .find_map(|line| line.strip_prefix("description:").map(str::trim));
    let expected = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name != Some(expected) {
        errors.push(format!(
            "{path}: frontmatter name {:?} does not match directory {expected}",
            name
        ));
    }
    if description.is_none_or(str::is_empty) {
        errors.push(format!("{path}: missing non-empty description"));
    }
}

fn inline_code(markdown: &str) -> Vec<String> {
    let mut code = Vec::new();
    let mut rest = markdown;
    while let Some(start) = rest.find('`') {
        rest = &rest[start + 1..];
        if rest.starts_with("``") {
            // Skills currently use no fenced command examples; skip fence markers defensively.
            rest = &rest[2..];
            continue;
        }
        let Some(end) = rest.find('`') else { break };
        code.push(rest[..end].to_string());
        rest = &rest[end + 1..];
    }
    code
}

fn validate_command(snippet: &str, commands: &[&Value], path: &str, errors: &mut Vec<String>) {
    let Some(words) = shlex::split(snippet) else {
        errors.push(format!("{path}: invalid quoting in `{snippet}`"));
        return;
    };
    let tokens: Vec<&str> = words.iter().map(String::as_str).collect();
    if tokens.first() != Some(&"okf") {
        return;
    }

    let matched = commands
        .iter()
        .filter_map(|command| {
            let parts: Vec<&str> = command["name"].as_str()?.split_whitespace().collect();
            (tokens.get(1..1 + parts.len()) == Some(parts.as_slice())).then_some((*command, parts))
        })
        .max_by_key(|(_, parts)| parts.len());
    let Some((command, command_parts)) = matched else {
        errors.push(format!("{path}: unknown command snippet `{snippet}`"));
        return;
    };

    // A bare command name is a prose mention. Once arguments appear, the snippet is
    // an invocation and must provide all required positionals, flags, and values.
    if tokens.len() == 1 + command_parts.len() {
        return;
    }

    let args = command["args"].as_array().cloned().unwrap_or_default();
    let positionals: Vec<&Value> = args
        .iter()
        .filter(|arg| arg["kind"] == "positional")
        .collect();
    let required_positionals = positionals
        .iter()
        .filter(|arg| arg["required"].as_bool().unwrap_or(false))
        .count();
    let mut positional_tokens = Vec::new();
    let mut seen_flags = Vec::new();
    let mut index = 1 + command_parts.len();

    while index < tokens.len() {
        let token = tokens[index];
        if token.starts_with("--") {
            let flag = token
                .trim_start_matches("--")
                .split('=')
                .next()
                .unwrap_or("");
            if flag == "json" {
                index += 1;
                continue;
            }
            let arg = args.iter().find(|arg| arg["name"] == flag);
            let Some(arg) = arg else {
                errors.push(format!("{path}: unknown flag `{token}` in `{snippet}`"));
                index += 1;
                continue;
            };
            seen_flags.push(flag);
            if arg["type"] != "bool" {
                let arity = arg["value_names"]
                    .as_array()
                    .map_or(1, |names| names.len().max(1));
                let mut supplied = usize::from(token.contains('='));
                while supplied < arity {
                    match tokens.get(index + 1) {
                        Some(value) if !value.starts_with("--") => {
                            index += 1;
                            supplied += 1;
                        }
                        _ => break,
                    }
                }
                if supplied < arity {
                    errors.push(format!(
                        "{path}: missing value for --{flag} (expected {arity}) in `{snippet}`"
                    ));
                }
            }
        } else {
            positional_tokens.push(token);
        }
        index += 1;
    }

    for arg in &args {
        if arg["kind"] != "positional" && arg["required"].as_bool().unwrap_or(false) {
            let flag = arg["name"].as_str().unwrap_or("");
            if !seen_flags.contains(&flag) {
                errors.push(format!(
                    "{path}: missing required flag --{flag} in `{snippet}`"
                ));
            }
        }
    }
    if positional_tokens.len() < required_positionals {
        errors.push(format!(
            "{path}: too few positional arguments in `{snippet}`"
        ));
    }
    for (position, token) in positional_tokens.iter().enumerate() {
        if position >= positionals.len() {
            errors.push(format!(
                "{path}: too many positional arguments in `{snippet}`"
            ));
            break;
        }
        let Some(placeholder) = placeholder_name(token) else {
            continue;
        };
        let expected = positionals[position]["name"].as_str().unwrap_or("");
        if placeholder != expected {
            errors.push(format!(
                "{path}: positional `{token}` is in slot {}, expected <{expected}> in `{snippet}`",
                position + 1
            ));
        }
    }
}

fn command_name<'a>(snippet: &str, commands: &[&'a Value]) -> Option<&'a str> {
    let tokens: Vec<&str> = snippet.split_whitespace().collect();
    commands
        .iter()
        .filter_map(|command| {
            let name = command["name"].as_str()?;
            let parts: Vec<&str> = name.split_whitespace().collect();
            (tokens.first() == Some(&"okf")
                && tokens.get(1..1 + parts.len()) == Some(parts.as_slice()))
            .then_some((name, parts.len()))
        })
        .max_by_key(|(_, parts)| *parts)
        .map(|(name, _)| name)
}

fn placeholder_name(token: &str) -> Option<&str> {
    let inner = token
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .or_else(|| {
            token
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
        })?;
    Some(match inner {
        "concept-id" => "concept",
        "git-ref" => "git_ref",
        "old-id" => "old",
        "new-id" => "new",
        "source-directory" => "directory",
        other => other,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn command(name: &str, group: &str) -> Value {
        json!({
            "kind": "command",
            "name": name,
            "group": group,
            "summary": format!("Run {name}"),
            "args": [],
            "mutates": false,
            "mutates_when": null,
            "output": {"stream": "test"}
        })
    }

    fn backlinks() -> Value {
        json!({
            "kind": "command",
            "name": "backlinks",
            "args": [
                {"name": "concept", "kind": "positional", "required": true},
                {"name": "bundle", "kind": "positional", "required": false}
            ]
        })
    }

    #[test]
    fn skill_command_check_accepts_schema_order() {
        let command = backlinks();
        let mut errors = Vec::new();
        validate_command(
            "okf backlinks <concept-id> <bundle>",
            &[&command],
            "skill",
            &mut errors,
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn skill_command_check_rejects_bundle_first_order() {
        let command = backlinks();
        let mut errors = Vec::new();
        validate_command(
            "okf backlinks <bundle> <concept-id>",
            &[&command],
            "skill",
            &mut errors,
        );
        assert!(errors.iter().any(|error| error.contains("slot 1")));
    }

    #[test]
    fn skill_command_check_rejects_unknown_flags() {
        let command = backlinks();
        let mut errors = Vec::new();
        validate_command(
            "okf backlinks <concept-id> --not-real",
            &[&command],
            "skill",
            &mut errors,
        );
        assert!(errors.iter().any(|error| error.contains("unknown flag")));
    }

    #[test]
    fn skill_command_mentions_are_not_invocations() {
        let command = backlinks();
        let mut errors = Vec::new();
        validate_command("okf backlinks", &[&command], "skill", &mut errors);
        assert!(errors.is_empty());
        validate_command("okf backlinks --json", &[&command], "skill", &mut errors);
        assert!(errors
            .iter()
            .any(|error| error.contains("too few positional")));
    }

    #[test]
    fn skill_command_check_requires_flags_and_their_values() {
        let command = json!({
            "name": "verify",
            "args": [
                {"name": "concept", "kind": "positional", "required": true},
                {"name": "by", "kind": "flag", "type": "string", "required": true}
            ]
        });
        for (snippet, expected) in [
            ("okf verify notes/test", "missing required flag --by"),
            ("okf verify notes/test --by", "missing value"),
            ("okf verify notes/test --by --json", "missing value"),
        ] {
            let mut errors = Vec::new();
            validate_command(snippet, &[&command], "skill", &mut errors);
            assert!(
                errors.iter().any(|error| error.contains(expected)),
                "{errors:?}"
            );
        }
        for snippet in [
            "okf verify notes/test --by process:ci",
            "okf verify notes/test --by=process:ci",
        ] {
            let mut errors = Vec::new();
            validate_command(snippet, &[&command], "skill", &mut errors);
            assert!(errors.is_empty(), "{errors:?}");
        }
    }

    #[test]
    fn skill_command_check_handles_quoted_multi_value_flags() {
        let command = json!({
            "name": "edit",
            "args": [
                {"name": "concept", "kind": "positional", "required": true},
                {"name": "set-section", "kind": "flag", "type": "list<string>",
                 "value_names": ["HEADING", "TEXT"]}
            ]
        });
        for snippet in [
            "okf edit notes/test --set-section \"Next steps\" '@/tmp/section body.md'",
            "okf edit notes/test --set-section='Next steps' 'Updated text'",
        ] {
            let mut errors = Vec::new();
            validate_command(snippet, &[&command], "skill", &mut errors);
            assert!(errors.is_empty(), "{errors:?}");
        }
        for (snippet, expected) in [
            (
                "okf edit notes/test --set-section 'Next steps'",
                "missing value",
            ),
            (
                "okf edit notes/test --set-section 'Next steps' --json",
                "missing value",
            ),
            (
                "okf edit notes/test --set-section 'Next steps",
                "invalid quoting",
            ),
        ] {
            let mut errors = Vec::new();
            validate_command(snippet, &[&command], "skill", &mut errors);
            assert!(
                errors.iter().any(|error| error.contains(expected)),
                "{errors:?}"
            );
        }
    }

    #[test]
    fn focused_reference_contains_only_mapped_commands() {
        let show = command("show", "query");
        let add = command("add", "mutate");
        let scenarios = json!([{
            "skill": "retrieval",
            "reference_commands": ["show"]
        }]);
        let commands = [&show, &add];
        let selected = commands_for_skill("retrieval", &scenarios, &commands).unwrap();
        let header = json!({"tool_version": "0.2.6", "okf_spec": ["0.2"]});
        let reference = cli_reference(&header, &selected, Some("retrieval"));

        assert!(reference.contains("# okf CLI — retrieval command reference"));
        assert!(reference.contains("### `okf show`"));
        assert!(!reference.contains("### `okf add`"));
    }

    #[test]
    fn focused_reference_rejects_duplicate_or_unknown_commands() {
        let show = command("show", "query");
        let commands = [&show];
        let duplicate = json!([{
            "skill": "retrieval",
            "reference_commands": ["show", "show"]
        }]);
        let unknown = json!([{
            "skill": "retrieval",
            "reference_commands": ["missing"]
        }]);

        assert!(commands_for_skill("retrieval", &duplicate, &commands)
            .unwrap_err()
            .contains("duplicate"));
        assert!(commands_for_skill("retrieval", &unknown, &commands)
            .unwrap_err()
            .contains("unknown"));
    }

    #[test]
    fn command_name_prefers_the_longest_subcommand_match() {
        let ontology_add = command("ontology add", "mutate");
        let ontology = command("ontology", "query");
        let commands = [&ontology, &ontology_add];

        assert_eq!(
            command_name("okf ontology add <name>", &commands),
            Some("ontology add")
        );
    }
}
