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
        "ndjson_schema": "2",
        "global_args": [{
            "name": "json",
            "type": "bool",
            "default": false,
            "help": "Emit NDJSON instead of human text or a bare artifact; schema is always NDJSON",
        }],
        "bundle_resolution": ["explicit", "env:OKF_BUNDLE", "config:okf.toml", "cwd"],
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
        .map(|arg| arg_record(full_name, arg))
        .collect();

    let mutates_when = match full_name {
        "doctor" => json!({"fix-safe": true, "yes": true, "dry-run": false}),
        "docs" => json!({"format": "index"}),
        _ => Value::Null,
    };
    let output = match full_name {
        "docs" => json!({
            "stream": "docs,change",
            "stream_when": {"format=index": "change", "otherwise": "docs"},
        }),
        _ => json!({"stream": stream}),
    };

    json!({
        "kind": "command",
        "name": full_name,
        "group": group,
        "mutates": mutates,
        "mutates_when": mutates_when,
        "summary": summary,
        "args": args,
        "output": output,
    })
}

/// Derive one arg's record from clap introspection.
fn arg_record(command: &str, arg: &clap::Arg) -> Value {
    let kind = if arg.is_positional() {
        "positional"
    } else {
        "flag"
    };
    // Positionals use their clap id; flags advertise the spelling users actually pass.
    let id = arg.get_id().as_str().trim_end_matches('_');
    let derived_long = id.replace('_', "-");
    let name = if arg.is_positional() {
        id.to_string()
    } else {
        arg.get_long().unwrap_or(&derived_long).to_string()
    };
    let action = arg.get_action();
    let is_bool = matches!(action, ArgAction::SetTrue | ArgAction::SetFalse);
    let repeatable = matches!(action, ArgAction::Append);
    let ty = if is_bool {
        "bool".to_string()
    } else if repeatable {
        "list<string>".to_string()
    } else if matches!(
        (command, id),
        ("search", "limit")
            | ("graph", "depth")
            | ("artifact show", "max_bytes")
            | ("affected", "depth")
    ) {
        "int".to_string()
    } else if id == "bundle"
        || matches!(
            (command, id),
            ("source-scan", "directory") | ("add", "path")
        )
    {
        "path".to_string()
    } else {
        "string".to_string()
    };

    let mut default = if repeatable {
        let delimiter = arg.get_value_delimiter();
        Value::Array(
            arg.get_default_values()
                .iter()
                .flat_map(|value| {
                    let value = value.to_string_lossy();
                    match delimiter {
                        Some(delimiter) => value
                            .split(delimiter)
                            .map(|part| Value::String(part.to_string()))
                            .collect::<Vec<_>>(),
                        None => vec![Value::String(value.into_owned())],
                    }
                })
                .collect(),
        )
    } else if matches!(action, ArgAction::SetTrue) {
        json!(false)
    } else if matches!(action, ArgAction::SetFalse) {
        json!(true)
    } else {
        arg.get_default_values()
            .first()
            .map(|value| {
                let value = value.to_string_lossy();
                if ty == "int" {
                    value
                        .parse::<u64>()
                        .map(Value::from)
                        .unwrap_or_else(|_| Value::String(value.into_owned()))
                } else {
                    Value::String(value.into_owned())
                }
            })
            .unwrap_or(Value::Null)
    };
    if default.is_null() {
        default = match (command, id) {
            ("graph", "direction") => json!("outgoing"),
            ("lint", "fail_on") => json!("error"),
            ("scan" | "stale" | "affected" | "diff" | "stats" | "refresh", "fail_on") => {
                json!("never")
            }
            _ => Value::Null,
        };
    }
    let possible_values: Vec<String> = arg
        .get_possible_values()
        .iter()
        .map(|value| value.get_name().to_string())
        .collect();
    let resolution = if id == "bundle" {
        json!(["explicit", "env:OKF_BUNDLE", "config:okf.toml", "cwd"])
    } else {
        Value::Null
    };
    let stdin = matches!((command, id), ("affected", "changed"));
    let resolution =
        (name == "bundle").then(|| vec!["explicit", "env:OKF_BUNDLE", "config:okf.toml", "cwd"]);

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
        "possible_values": possible_values,
        "resolution": resolution,
        "stdin": stdin,
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
        "links" => ("query", false, "link"),
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

        "docs" => ("render", true, "docs,change"),

        _ => ("query", false, "concept"),
    }
}
