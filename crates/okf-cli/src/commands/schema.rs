//! `okf schema` and `okf version`.
//!
//! `schema` is **derived** from the clap command tree (never hand-maintained): it walks the
//! parsed [`crate::cli::Cli`] `Command`, and for each leaf command emits one NDJSON line whose
//! args come straight from clap introspection. A small static table supplies each command's
//! `group`, `mutates` flag and output `stream` (things clap does not model).
use crate::cli::Cli;
use crate::output;
use clap::{ArgAction, Command as ClapCommand, CommandFactory};
use okf_core::error::Result;
use serde_json::{json, Value};

/// OKF spec version(s) this CLI targets.
const OKF_SPEC: &[&str] = &["0.2"];

/// `okf version`.
pub fn run_version(json: bool) -> Result<i32> {
    let ver = env!("CARGO_PKG_VERSION");
    if json {
        output::print_line(&json!({
            "kind": "version",
            "tool": "okf",
            "tool_version": ver,
            "okf_spec": OKF_SPEC,
        }))?;
    } else {
        println!("okf {ver} (OKF spec {})", OKF_SPEC.join(", "));
    }
    Ok(0)
}

/// `okf schema` — always NDJSON, regardless of `--json` (it is the machine-discovery surface).
pub fn run_schema(_json: bool) -> Result<i32> {
    // Header line describing the tool/contract.
    output::print_line(&json!({
        "kind": "schema",
        "tool": "okf",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "okf_spec": OKF_SPEC,
        "ndjson_schema": "1",
    }))?;

    let cli = Cli::command();
    for sub in cli.get_subcommands() {
        if sub.has_subcommands() {
            // Nested group (e.g. `ontology`): emit one line per child.
            let parent = sub.get_name();
            for child in sub.get_subcommands() {
                let full = format!("{parent} {}", child.get_name());
                output::print_line(&command_record(&full, child))?;
            }
        } else {
            output::print_line(&command_record(sub.get_name(), sub))?;
        }
    }
    Ok(0)
}

/// Build the `{"kind":"command",...}` record for one leaf command.
fn command_record(full_name: &str, cmd: &ClapCommand) -> Value {
    let (group, mutates, stream) = meta(full_name);
    let summary = cmd.get_about().map(|s| s.to_string()).unwrap_or_default();

    let args: Vec<Value> = cmd
        .get_arguments()
        .filter(|a| {
            let id = a.get_id().as_str();
            // Skip global/auto args (`--json`, `--help`, `--version`).
            id != "json" && id != "help" && id != "version" && !a.is_global_set()
        })
        .map(arg_record)
        .collect();

    json!({
        "kind": "command",
        "name": full_name,
        "group": group,
        "mutates": mutates,
        "summary": summary,
        "args": args,
        "output": {"stream": stream},
    })
}

/// Derive one arg's record from clap introspection.
fn arg_record(arg: &clap::Arg) -> Value {
    let kind = if arg.is_positional() {
        "positional"
    } else {
        "flag"
    };
    // Schema names normally retain clap's underscore-based id; renderers turn underscores into
    // hyphens. An explicitly renamed long flag can differ from that derived spelling, though
    // (for example, the `reference` field is exposed as `--ref`). In that case advertise the
    // spelling a user can actually pass to the CLI.
    let id = arg.get_id().as_str().trim_end_matches('_');
    let derived_long = id.replace('_', "-");
    let name = match arg.get_long() {
        Some(long) if long != derived_long => long,
        _ => id,
    }
    .to_string();
    let action = arg.get_action();
    let is_bool = matches!(action, ArgAction::SetTrue | ArgAction::SetFalse);
    let repeatable = matches!(action, ArgAction::Append);
    let ty = if is_bool {
        "bool".to_string()
    } else if repeatable {
        "list<string>".to_string()
    } else {
        "string".to_string()
    };

    let mut default: Option<String> = arg
        .get_default_values()
        .first()
        .map(|s| s.to_string_lossy().into_owned());
    // Bundle positionals resolve to $OKF_BUNDLE / cwd at runtime; advertise "." as the default.
    if name == "bundle" && default.is_none() {
        default = Some(".".to_string());
    }

    // The doc-comment help text, so consumers can document an arg without a `--help` round-trip.
    let help = arg.get_help().map(|s| s.to_string());
    // Multi-value args (e.g. `--set-section <HEADING> <TEXT>`) advertise their value names.
    let value_names: Vec<String> = arg
        .get_value_names()
        .map(|names| names.iter().map(|n| n.to_string()).collect())
        .filter(|v: &Vec<String>| v.len() > 1)
        .unwrap_or_default();

    json!({
        "name": name,
        "kind": kind,
        "type": ty,
        "required": arg.is_required_set(),
        "repeatable": repeatable,
        "default": default,
        "help": help,
        "value_names": value_names,
    })
}

/// Static per-command metadata clap cannot model: (group, mutates, output stream).
fn meta(name: &str) -> (&'static str, bool, &'static str) {
    match name {
        "schema" => ("meta", false, "schema"),
        "version" => ("meta", false, "version"),

        "list" => ("query", false, "concept"),
        "search" => ("query", false, "concept"),
        "show" => ("query", false, "concept"),
        "browse" => ("query", false, "index"),
        "backlinks" => ("query", false, "concept"),
        "graph" => ("query", false, "graph"),
        "resolve" => ("query", false, "resolved"),
        "artifact list" => ("query", false, "artifact"),
        "artifact resolve" => ("query", false, "artifact-resolution"),
        "artifact show" => ("query", false, "artifact-content"),
        "computation check" => ("check", false, "computation-contract"),
        "ontology list" => ("query", false, "ontology_type"),
        "ontology show" => ("query", false, "ontology_type"),

        "scan" => ("check", false, "scan"),
        "source-scan" => ("check", false, "source-file"),
        "validate" => ("check", false, "violation"),
        "lint" => ("check", false, "finding"),
        "stale" => ("check", false, "drift"),
        "affected" => ("check", false, "affected"),
        "diff" => ("check", false, "diff"),
        "stats" => ("check", false, "stats"),
        "doctor" => ("check", true, "doctor-finding,doctor-summary"),

        "init" => ("mutate", true, "change"),
        "add" => ("mutate", true, "change"),
        "edit" => ("mutate", true, "change"),
        "mv" => ("mutate", true, "change"),
        "rm" => ("mutate", true, "change"),
        "verify" => ("mutate", true, "change"),
        "refresh" => ("mutate", true, "change"),
        "ontology add" => ("mutate", true, "change"),
        "ontology update" => ("mutate", true, "change"),
        "ontology remove" => ("mutate", true, "change"),

        "docs" => ("render", false, "docs"),

        _ => ("query", false, "concept"),
    }
}
