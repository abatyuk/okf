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
    let cli = crate::cli::command_tree();
    // Header line describing the tool/contract.
    output::print_line(&json!({
        "kind": "schema",
        "tool": "okf",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "okf_spec": OKF_SPEC,
        "ndjson_schema": "2",
        "global_args": cli.get_arguments()
            .filter(|arg| arg.is_global_set() && arg.get_id() != "help" && arg.get_id() != "version")
            .map(|arg| arg_record("", arg)).collect::<Vec<_>>(),
        "bundle_resolution": ["explicit:path-or-id", "env:OKF_BUNDLE:path", "catalog:cwd-containing-root", "config:default_bundle", "catalog:sole-entry", "cwd:without-catalog"],
        "scope": {"default": "selected-bundle", "additional_bundles": "explicit", "registration_authorizes_traversal": false},
        "output_records": output_record_contracts(),
    }))?;

    fn emit_leaves(prefix: &str, cmd: &ClapCommand) -> Result<()> {
        if cmd.get_name() == "help" {
            return Ok(());
        }
        let name = if prefix.is_empty() {
            cmd.get_name().to_owned()
        } else {
            format!("{prefix} {}", cmd.get_name())
        };
        if cmd.has_subcommands() {
            for child in cmd.get_subcommands() {
                emit_leaves(&name, child)?;
            }
        } else {
            output::print_line(&command_record(&name, cmd))?;
        }
        Ok(())
    }
    for sub in cli.get_subcommands() {
        emit_leaves("", sub)?;
    }
    Ok(0)
}

/// New extension records are separate from authored concept output. The field contracts
/// identify typed identities and statuses without imposing metadata schemas on concepts.
fn output_record_contracts() -> Value {
    json!({
        "changeset-file": {"path":"path","operation":"create|replace|remove","before_bytes":"integer|null","after_bytes":"integer|null","diff":"string|null"},
        "changeset-summary": {"base_digest":"string","files":"integer","applied":"boolean","publication":"journaled-per-file"},
        "changeset-recovery": {"paths":"array<path>"},
        "ontology_field_type": {"name":"string","definition":"object","effective":"object"},
        "qualified_identity": {"bundle": "string", "id": "string", "version": "string"},
        "scope": {"requested": "array<string>", "examined": "array<{id:string,root:path,version:string,interpretation:interpretation-settings}>", "unavailable": "array<string>", "snapshot_examined": "array<qualified_identity> (optional)"},
        "interpretation-settings": {"bundle": "string|null", "ontology_path": "path|null", "ontology_digest": "string|null", "settings": "object"},
        "effective-settings": {"bundle": "string", "settings": "interpretation-settings"},
        "concept-identity": {"schema_version": 1, "id": "string", "bundle": "string", "version": "string"},
        "query-summary": {
            "schema_version": 1, "total_matches": "integer|null", "observed_matches": "integer", "returned": "integer",
            "offset": "integer", "limit": "integer|null", "has_more": "boolean", "next_offset": "integer|null",
            "scan_complete": "boolean", "partial": "boolean", "examined_documents": "integer", "scan_limit": "integer|null",
            "filters": "array<string>", "scope": "array<{bundle:string,root:path,version:string,mutable:boolean}>", "interpretation": "array<interpretation-settings>"
        },
        "projection": {
            "schema_version": 1, "id": "string", "bundle": "string|null", "version": "string",
            "fields": "array<{field:string,present:boolean,value?:any,occurrences?:array<{path:string,value:any}>,empty_lists?:integer}>"
        },
        "facet": {
            "schema_version": 1, "field": "string", "basis": "all-matches|observed-matches", "complete": "boolean",
            "truncated": "boolean", "omitted_values": "integer", "values": "array<{value:scalar,count:integer}>", "diagnostics": "array<object>"
        },
        "facet-excluded": {"schema_version": 1, "field": "string", "reason": "string", "threshold": "integer", "distinct_values": "integer", "complete": "boolean"},
        "relationship": {"schema_version": 1, "primary": "string", "bundle": "string", "version": "string", "edge": "object", "target_identity": "string|null", "target_payload_status": "string (optional)"},
        "related-concept": {"schema_version": 1, "identity": "string", "id": "string", "bundle": "string|null", "version": "string", "fields": "projection.fields"},
        "expansion-summary": {"schema_version": 1, "emitted_edges": "integer", "emitted_targets": "integer", "emitted_bytes": "integer", "bounds": "{edges_per_hit:integer,targets:integer,bytes:integer}", "truncated": "boolean", "reasons": "array<string>", "total_edges": "integer|null"},
        "warning": {"schema_version": 1, "reason": "string", "limit": "integer|null", "omitted": "integer|null", "message": "string"},
        "bundle-registration": {"id": "string", "configured_root": "path", "root": "path", "overridden": "boolean", "available": "boolean"},
        "bundle-backlink": {"bundle": "string", "id": "string", "version": "string"},
        "bundle-affected": {"bundle": "string", "id": "string", "version": "string"},
        "bundle-node": {"bundle": "string", "id": "string", "version": "string"},
        "bundle-edge": {
            "source": "qualified_identity", "target": "qualified_identity|null",
            "resource": "string", "location": "string", "status": "string", "evidence": "string",
            "snapshot": "{requested:object,status:string,evidence:string,resolved:qualified_identity|null,candidate:qualified_identity|null}|null",
            "fingerprint_status": "string|null",
            "relationship": "{source:string,rule:string,field_path:string,raw_reference:string,target:string,authored_kind:string|null,inverse:string|null,attributes:object,status:string}|null"
        }
    })
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
        .map(|arg| {
            let mut record = arg_record(full_name, arg);
            record["conflicts_with"] = json!(cmd
                .get_arg_conflicts_with(arg)
                .iter()
                .map(|a| a.get_long().unwrap_or(a.get_id().as_str()))
                .collect::<Vec<_>>());
            record
        })
        .collect();

    let mutates_when = match full_name {
        "doctor" => json!({"fix-safe": true, "yes": true, "dry-run": false}),
        "docs" => json!({"format": "index"}),
        _ if cmd
            .get_arguments()
            .any(|arg| arg.get_long() == Some("dry-run")) =>
        {
            json!({"dry-run": false})
        }
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
        "mutates_when": if mutates {mutates_when} else {Value::Null},
        "supported_globals": {"json":true,"bundle-id":crate::cli::supports_bundle(full_name),"scope-bundle":crate::cli::supports_scope(full_name),"catalog-scope":crate::cli::supports_scope(full_name),"revision":crate::cli::supports_revision(full_name)},
        "availability": {"remote_fetch": cfg!(feature="url-sources"), "pdf":false},
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
        (
            "search" | "list",
            "limit"
                | "offset"
                | "scan_limit"
                | "expansion_edges"
                | "expansion_targets"
                | "expansion_bytes"
        ) | ("graph", "depth")
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
    let stdin = matches!((command, id), ("affected", "changed"))
        || matches!(
            id,
            "set_yaml"
                | "set_path"
                | "field_yaml"
                | "ref_yaml"
                | "relationship_yaml"
                | "patch"
                | "add_source_json"
                | "inline_computation"
                | "set_body"
                | "append_body"
                | "set_section"
                | "append_section"
        )
        || (command == "add" && id == "body")
        || (id == "from"
            && (command.starts_with("ontology ") || command.starts_with("changeset ")));
    let requires: &[&str] = match (command, id) {
        ("search" | "list", "offset") => &["limit"],
        ("graph", "depth") => &["root"],
        ("doctor", "yes") => &["fix-safe"],
        ("edit", "all") => &["replace"],
        _ => &[],
    };
    let resolution = (name == "bundle").then(|| {
        vec![
            "explicit:path",
            "env:OKF_BUNDLE:path",
            "catalog:cwd-containing-root",
            "config:default_bundle",
            "catalog:sole-entry",
            "cwd:without-catalog",
        ]
    });

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
        "requires": requires,
        "aliases": arg.get_all_aliases().unwrap_or_default(),
        "value_delimiter": arg.get_value_delimiter().map(|c| c.to_string()),
        "arity": arg.get_num_args().map(|n| json!({"min":n.min_values(),"max":n.max_values()})),
    })
}

/// Static per-command metadata clap cannot model: (group, mutates, output stream).
fn meta(name: &str) -> (&'static str, bool, &'static str) {
    match name {
        "schema" => ("meta", false, "schema"),
        "version" => ("meta", false, "version"),
        "catalog" => ("meta", false, "bundle-registration,effective-settings"),

        "list" | "search" => ("query", false, "concept,concept-identity,scope,projection,query-summary,facet,facet-excluded,relationship,related-concept,expansion-summary,warning"),
        "show" => ("query", false, "concept,projection"),
        "browse" => ("query", false, "index"),
        "backlinks" => ("query", false, "concept,relationship,scope,bundle-backlink,bundle-edge"),
        "links" => ("query", false, "link,relationship,scope,bundle-edge"),
        "graph" => ("query", false, "graph,scope,bundle-node,bundle-edge"),
        "resolve" => ("query", false, "resolved,scope,bundle-edge"),
        "artifact list" => ("query", false, "artifact"),
        "artifact resolve" => ("query", false, "artifact-resolution"),
        "artifact show" => ("query", false, "artifact-content"),
        "artifact put" => ("mutate", true, "artifact-write"),
        "computation check" => ("check", false, "computation-contract"),
        "ontology list" => ("query", false, "ontology_type"),
        "ontology show" => ("query", false, "ontology_type"),
        "ontology field-type list" | "ontology field-type show" => ("query", false, "ontology_field_type"),
        "changeset plan" => ("query", false, "changeset-file,changeset-summary"),
        "changeset apply" => ("mutate", true, "changeset-file,changeset-summary"),
        "changeset recover" => ("mutate", true, "changeset-recovery"),

        "scan" => ("check", false, "scan"),
        "source-scan" => ("check", false, "source-file"),
        "validate" => ("check", false, "violation"),
        "lint" => ("check", false, "finding,scope"),
        "stale" => ("check", false, "drift"),
        "affected" => ("check", false, "affected,scope,bundle-affected"),
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
        "ontology apply" | "ontology field-type add" | "ontology field-type update" | "ontology field-type remove" => ("mutate", true, "change"),

        "docs" => ("render", true, "docs,change"),

        _ => ("query", false, "concept"),
    }
}
