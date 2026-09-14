//! init, add, edit, mv, rm, verify, refresh. Each emits a `change` record under `--json`.
use crate::cli::{AddArgs, EditArgs, InitArgs, MvArgs, RefreshArgs, RmArgs, VerifyArgs};
use crate::output;
use okf_core::bundle::resolve::{resolve_bundle, resolve_bundle_target};
use okf_core::error::Result;
use okf_core::fingerprint::Engine;
use okf_core::model::source::{Fingerprint, Source, SourceKind};
use okf_core::mutate::add::{add, AddOptions};
use okf_core::mutate::edit::{edit, parse_kv, EditChange, EditSpec, SourceSelector};
use okf_core::mutate::init::{init, InitOptions};
use okf_core::mutate::mv::mv;
use okf_core::mutate::refresh::refresh;
use okf_core::mutate::rm::rm;
use okf_core::mutate::verify::verify;
use okf_core::ontology::load::try_load;
use okf_core::ports::clock::Clock;
use okf_core::ports::clock::SystemClock;
use okf_core::ports::fs::RealFs;
use okf_core::ports::git::RealGit;
use serde_json::json;

/// `okf init [bundle] [--title] [--no-index] [--no-ontology]`.
pub fn run_init(args: &InitArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle_target(args.bundle.as_deref())?;
    let opts = InitOptions {
        index: !args.no_index,
        ontology: !args.no_ontology,
        title: args.title.clone(),
    };
    let r = init(&root, &opts)?;
    if json {
        output::print_line(&json!({
            "kind": "change",
            "op": "init",
            "root": r.root.to_string_lossy(),
            "created_dir": r.created_dir,
            "index": r.index_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "ontology": r.ontology_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
        }))?;
    } else {
        println!("initialized bundle at {}", r.root.display());
        if let Some(p) = &r.index_path {
            println!("  wrote {}", p.display());
        }
        if let Some(p) = &r.ontology_path {
            println!("  wrote {}", p.display());
        }
    }
    Ok(0)
}

/// `okf add <path> [bundle] [--type] [--title] [--description] [--attested]`.
pub fn run_add(args: &AddArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let ontology = try_load(&root)?;
    let clock = SystemClock;
    let mut stdin_cache = None;
    let opts = AddOptions {
        concept_type: args.type_.clone(),
        title: args.title.clone(),
        description: args.description.clone(),
        attested: args.attested,
        sets: parse_pairs("--set", &args.set)?
            .into_iter()
            .map(|(k, v)| (k, okf_core::mutate::edit::parse_scalar(&v)))
            .collect(),
        references: parse_pairs("--ref", &args.reference)?,
        sources: combined_sources(&args.add_source, &args.add_source_json)?,
        runtime: args.runtime.clone(),
        parameters: parse_parameters(&args.parameter)?,
        computation: args.computation.clone(),
        inline_computation: resolve_opt(&args.inline_computation, &mut stdin_cache)?,
        executor_resource: args.executor_resource.clone(),
        receipt: args.receipt.clone(),
        attester_resource: args.attester_resource.clone(),
        generated_by: args.generated_by.clone(),
        generated_at: args.generated_by.as_ref().map(|_| clock.now_rfc3339()),
    };
    let r = add(&root, &args.path, ontology.as_ref(), &opts)?;
    if json {
        output::print_line(&json!({
            "kind": "change",
            "op": "add",
            "id": r.id.0,
            "path": r.path.to_string_lossy(),
            "type": r.concept_type,
            "attested": r.attested,
        }))?;
    } else {
        println!(
            "added {} ({}) at {}",
            r.id.0,
            r.concept_type,
            r.path.display()
        );
    }
    Ok(0)
}

fn parse_parameters(args: &[String]) -> Result<Vec<(String, String, bool)>> {
    use okf_core::error::OkfError;
    args.iter()
        .map(|raw| {
            let parts: Vec<&str> = raw.split(':').collect();
            if !(2..=3).contains(&parts.len())
                || parts[0].trim().is_empty()
                || parts[1].trim().is_empty()
                || (parts.len() == 3 && parts[2] != "required")
            {
                return Err(OkfError::Usage(format!(
                    "invalid --parameter {raw:?}: expected name:type[:required]"
                )));
            }
            Ok((parts[0].to_string(), parts[1].to_string(), parts.len() == 3))
        })
        .collect()
}

/// `okf edit <concept> [bundle]` with frontmatter (`--set/--unset/--add/--remove`) and body
/// (`--set-body/--append-body/--clear-body`, `--*-section`) operations.
pub fn run_edit(args: &EditArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let mut stdin_cache: Option<String> = None;

    let spec = EditSpec {
        sets: parse_pairs("--set", &args.set)?,
        unsets: args.unset.clone(),
        adds: parse_pairs("--add", &args.add)?,
        removes: parse_pairs("--remove", &args.remove)?,
        add_sources: combined_sources(&args.add_source, &args.add_source_json)?,
        remove_sources: parse_source_selector_args(&args.remove_source)?,
        clear_body: args.clear_body,
        set_body: resolve_opt(&args.set_body, &mut stdin_cache)?,
        append_body: resolve_opt(&args.append_body, &mut stdin_cache)?,
        set_sections: section_pairs(&args.set_section, &mut stdin_cache)?,
        append_sections: section_pairs(&args.append_section, &mut stdin_cache)?,
        remove_sections: args.remove_section.clone(),
    };

    let r = edit(&root, &args.concept, &spec)?;
    if json {
        let changes: Vec<_> = r.changes.iter().map(change_json).collect();
        output::print_line(&json!({
            "kind": "change",
            "op": "edit",
            "id": r.id.0,
            "path": r.path.to_string_lossy(),
            "changes": changes,
        }))?;
    } else {
        println!("edited {} ({} change(s))", r.id.0, r.changes.len());
    }
    Ok(0)
}

/// Split a list of `key=value` args for a given flag.
fn parse_pairs(flag: &str, args: &[String]) -> Result<Vec<(String, String)>> {
    args.iter().map(|a| parse_kv(flag, a)).collect()
}

/// Parse a compact standard source plus optional extension/signals. Full mappings and values
/// containing commas should use `--add-source-json`.
fn parse_source_args(args: &[String]) -> Result<Vec<Source>> {
    use okf_core::error::OkfError;
    args.iter()
        .map(|arg| {
            let mut resource = None;
            let mut kind = None;
            let mut extra = indexmap::IndexMap::new();
            for part in arg.split(',') {
                let (key, value) = part.split_once('=').ok_or_else(|| {
                    OkfError::Usage(format!(
                        "invalid --add-source {arg:?}: expected comma-separated key=value fields"
                    ))
                })?;
                match key.trim() {
                    "resource" if !value.trim().is_empty() => {
                        resource = Some(value.trim().to_string())
                    }
                    "kind" if !value.trim().is_empty() => kind = Some(value.trim().to_string()),
                    "id" | "title" | "author" | "last_modified" if !value.trim().is_empty() => {
                        extra.insert(
                            key.trim().to_string(),
                            serde_yaml::Value::String(value.trim().to_string()),
                        );
                    }
                    "usage_count" if value.trim().parse::<u64>().is_ok() => {
                        extra.insert(
                            "usage_count".to_string(),
                            serde_yaml::Value::Number(value.trim().parse::<u64>().unwrap().into()),
                        );
                    }
                    other => {
                        return Err(OkfError::Usage(format!(
                            "invalid --add-source key {other:?}: expected resource, kind, id, title, author, usage_count, or last_modified"
                        )))
                    }
                }
            }
            let resource = resource.ok_or_else(|| {
                OkfError::Usage("--add-source requires resource=<value>".to_string())
            })?;
            Ok(Source {
                resource,
                kind: kind
                    .as_deref()
                    .map(SourceKind::from_kind_str)
                    .unwrap_or_else(|| SourceKind::Other(String::new())),
                fingerprint: Fingerprint::default(),
                extra,
            })
        })
        .collect()
}

fn combined_sources(compact: &[String], structured: &[String]) -> Result<Vec<Source>> {
    let mut sources = parse_source_args(compact)?;
    sources.extend(parse_source_json_args(structured)?);
    Ok(sources)
}

fn parse_source_json_args(args: &[String]) -> Result<Vec<Source>> {
    use okf_core::error::OkfError;
    args.iter()
        .map(|raw| {
            let content = if let Some(path) = raw.strip_prefix('@') {
                std::fs::read_to_string(path)
                    .map_err(|e| OkfError::Environment(format!("cannot read {path}: {e}")))?
            } else {
                raw.clone()
            };
            let value: serde_yaml::Value = serde_yaml::from_str(&content)
                .map_err(|e| OkfError::Usage(format!("invalid --add-source-json: {e}")))?;
            let source = Source::from_value(&value).ok_or_else(|| {
                OkfError::Usage("--add-source-json must be a source mapping".to_string())
            })?;
            if source.resource.trim().is_empty() {
                return Err(OkfError::Usage(
                    "--add-source-json requires non-empty resource".to_string(),
                ));
            }
            Ok(source)
        })
        .collect()
}

/// Parse a `--remove-source` selector. A bare value matches every source with that resource;
/// the structured form can additionally restrict the match to one source kind.
fn parse_source_selector_args(args: &[String]) -> Result<Vec<SourceSelector>> {
    use okf_core::error::OkfError;
    args.iter()
        .map(|arg| {
            if !arg.contains('=') {
                let resource = arg.trim();
                if resource.is_empty() {
                    return Err(OkfError::Usage(
                        "--remove-source requires a non-empty resource".to_string(),
                    ));
                }
                return Ok(SourceSelector {
                    resource: resource.to_string(),
                    kind: None,
                });
            }

            let mut resource = None;
            let mut kind = None;
            for part in arg.split(',') {
                let (key, value) = part.split_once('=').ok_or_else(|| {
                    OkfError::Usage(format!(
                        "invalid --remove-source {arg:?}: expected resource=<value>[,kind=<value>]"
                    ))
                })?;
                match key.trim() {
                    "resource" if !value.trim().is_empty() => {
                        resource = Some(value.trim().to_string())
                    }
                    "kind" if !value.trim().is_empty() => {
                        kind = Some(SourceKind::from_kind_str(value.trim()))
                    }
                    other => {
                        return Err(OkfError::Usage(format!(
                            "invalid --remove-source key {other:?}: expected resource and optional kind"
                        )))
                    }
                }
            }
            Ok(SourceSelector {
                resource: resource.ok_or_else(|| {
                    OkfError::Usage("--remove-source requires resource=<value>".to_string())
                })?,
                kind,
            })
        })
        .collect()
}

/// Chunk a `num_args = 2` section flag (`<heading> <text>` pairs) into `(heading, text)`,
/// resolving each text through `@file`/`-`/literal.
fn section_pairs(
    flat: &[String],
    stdin_cache: &mut Option<String>,
) -> Result<Vec<(String, String)>> {
    let mut out = Vec::with_capacity(flat.len() / 2);
    for pair in flat.chunks(2) {
        // clap enforces num_args = 2, so every chunk has exactly two elements.
        let text = resolve_text(&pair[1], stdin_cache)?;
        out.push((pair[0].clone(), text));
    }
    Ok(out)
}

/// Resolve an optional body-text arg through the content resolver.
fn resolve_opt(arg: &Option<String>, stdin_cache: &mut Option<String>) -> Result<Option<String>> {
    match arg {
        Some(raw) => Ok(Some(resolve_text(raw, stdin_cache)?)),
        None => Ok(None),
    }
}

/// Resolve document text: `@path` reads a file, `-` reads stdin (once, cached), anything else
/// is literal. A literal `@`/`-` can be escaped by prefixing another `@` (e.g. `@@`).
fn resolve_text(raw: &str, stdin_cache: &mut Option<String>) -> Result<String> {
    use okf_core::error::OkfError;
    if raw == "-" {
        if stdin_cache.is_none() {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
                .map_err(|e| OkfError::Io(format!("reading stdin: {e}")))?;
            *stdin_cache = Some(buf);
        }
        return Ok(stdin_cache.clone().unwrap());
    }
    if let Some(path) = raw.strip_prefix('@') {
        if let Some(escaped) = path.strip_prefix('@') {
            return Ok(format!("@{escaped}")); // `@@foo` → literal `@foo`
        }
        return std::fs::read_to_string(path)
            .map_err(|e| OkfError::Environment(format!("cannot read {path}: {e}")));
    }
    Ok(raw.to_string())
}

/// Render one applied change as a `--json` object.
fn change_json(c: &EditChange) -> serde_json::Value {
    match c {
        EditChange::Set { key, existed } => json!({"op": "set", "key": key, "existed": existed}),
        EditChange::Unset { key, existed } => {
            json!({"op": "unset", "key": key, "existed": existed})
        }
        EditChange::Add { key, added } => json!({"op": "add", "key": key, "added": added}),
        EditChange::Remove { key, removed } => {
            json!({"op": "remove", "key": key, "removed": removed})
        }
        EditChange::SetBody => json!({"op": "set-body"}),
        EditChange::AppendBody => json!({"op": "append-body"}),
        EditChange::ClearBody => json!({"op": "clear-body"}),
        EditChange::SetSection { heading } => json!({"op": "set-section", "heading": heading}),
        EditChange::AppendSection { heading } => {
            json!({"op": "append-section", "heading": heading})
        }
        EditChange::RemoveSection { heading } => {
            json!({"op": "remove-section", "heading": heading})
        }
        EditChange::InvalidateVerification { removed } => {
            json!({"op": "invalidate-verification", "removed": removed})
        }
    }
}

/// `okf mv <old> <new> [bundle]`.
pub fn run_mv(args: &MvArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let r = mv(&root, &args.old, &args.new)?;
    if json {
        let rewritten: Vec<_> = r.rewritten.iter().map(|c| c.0.clone()).collect();
        output::print_line(&json!({
            "kind": "change",
            "op": "mv",
            "from": r.from.0,
            "to": r.to.0,
            "rewritten": rewritten,
        }))?;
    } else {
        println!(
            "moved {} -> {} ({} referrer(s) rewritten)",
            r.from.0,
            r.to.0,
            r.rewritten.len()
        );
    }
    Ok(0)
}

/// `okf rm <concept> [bundle] [--force]`.
pub fn run_rm(args: &RmArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let r = rm(&root, &args.concept, args.force)?;
    if json {
        let dangling: Vec<_> = r.dangling_referrers.iter().map(|c| c.0.clone()).collect();
        output::print_line(&json!({
            "kind": "change",
            "op": "rm",
            "id": r.id.0,
            "removed": r.removed,
            "dangling_referrers": dangling,
        }))?;
    } else {
        println!("removed {}", r.id.0);
        if !r.dangling_referrers.is_empty() {
            eprintln!(
                "warning: {} referrer(s) now dangle",
                r.dangling_referrers.len()
            );
        }
    }
    Ok(0)
}

/// `okf verify <concept> [bundle] --by <actor>`.
pub fn run_verify(args: &VerifyArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let clock = SystemClock;
    let r = verify(&root, &args.concept, &args.by, &clock)?;
    if json {
        output::print_line(&json!({
            "kind": "change",
            "op": "verify",
            "id": r.id.0,
            "actor": r.actor,
            "at": r.at,
        }))?;
    } else {
        println!("verified {} by {} at {}", r.id.0, r.actor, r.at);
    }
    Ok(0)
}

/// `okf refresh <concept> [bundle]`.
pub fn run_refresh(args: &RefreshArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let fs = RealFs;
    let git = RealGit;
    let clock = SystemClock;
    let engine = Engine::new(&root, &fs, &git, None);
    let r = refresh(&root, &args.concept, &engine, &clock)?;
    if json {
        let skipped: Vec<_> = r
            .skipped
            .iter()
            .map(|s| json!({"resource": s.resource, "reason": s.reason}))
            .collect();
        output::print_line(&json!({
            "kind": "change",
            "op": "refresh",
            "id": r.id.0,
            "updated": r.updated,
            "unchanged": r.unchanged,
            "skipped": skipped,
            "refreshed_at": r.refreshed_at,
        }))?;
    } else {
        println!(
            "refreshed {} ({} updated, {} unchanged, {} skipped)",
            r.id.0,
            r.updated.len(),
            r.unchanged.len(),
            r.skipped.len()
        );
        for skipped in &r.skipped {
            eprintln!("  skipped {}: {}", skipped.resource, skipped.reason);
        }
    }
    let fail = match args.fail_on.as_deref() {
        None | Some("never") => false,
        Some("any") | Some("skipped") => !r.skipped.is_empty(),
        Some(other) => {
            return Err(okf_core::error::OkfError::Usage(format!(
                "invalid --fail-on {other:?}: expected never, skipped, or any"
            )))
        }
    };
    Ok(if fail { 1 } else { 0 })
}
