//! init, add, edit, mv, rm, verify, refresh. Each emits a `change` record under `--json`.
use crate::cli::{AddArgs, EditArgs, InitArgs, MvArgs, RefreshArgs, RmArgs, VerifyArgs};
use crate::output;
use okf_core::bundle::resolve::resolve_bundle;
use okf_core::error::Result;
use okf_core::fingerprint::Engine;
use okf_core::model::source::{Fingerprint, Source, SourceKind};
use okf_core::mutate::add::{add, AddOptions};
use okf_core::mutate::edit::{edit, parse_kv, EditChange, EditSpec};
use okf_core::mutate::init::{init, InitOptions};
use okf_core::mutate::mv::mv;
use okf_core::mutate::refresh::refresh;
use okf_core::mutate::rm::rm;
use okf_core::mutate::verify::verify;
use okf_core::ontology::load::try_load;
use okf_core::ports::clock::SystemClock;
use okf_core::ports::fs::RealFs;
use okf_core::ports::git::RealGit;
use serde_json::json;
use std::path::PathBuf;

/// Resolve a bundle path that need not yet exist (for `init`): explicit arg, else $OKF_BUNDLE,
/// else the current directory.
fn init_target(arg: Option<&str>) -> Result<PathBuf> {
    if let Some(a) = arg {
        return Ok(PathBuf::from(a));
    }
    if let Ok(env) = std::env::var("OKF_BUNDLE") {
        if !env.is_empty() {
            return Ok(PathBuf::from(env));
        }
    }
    std::env::current_dir().map_err(|e| {
        okf_core::error::OkfError::Environment(format!("cannot read current directory: {e}"))
    })
}

/// `okf init [bundle] [--title] [--no-index] [--no-ontology]`.
pub fn run_init(args: &InitArgs, json: bool) -> Result<i32> {
    let root = init_target(args.bundle.as_deref())?;
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
        sources: parse_source_args(&args.add_source)?,
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
        add_sources: parse_source_args(&args.add_source)?,
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

/// Parse `resource=...,kind=...` into a typed source. The comma delimiter is intentionally
/// narrow; paths and URIs containing commas can use percent-encoding.
fn parse_source_args(args: &[String]) -> Result<Vec<Source>> {
    use okf_core::error::OkfError;
    args.iter()
        .map(|arg| {
            let mut resource = None;
            let mut kind = None;
            for part in arg.split(',') {
                let (key, value) = part.split_once('=').ok_or_else(|| {
                    OkfError::Usage(format!(
                        "invalid --add-source {arg:?}: expected resource=<value>,kind=<value>"
                    ))
                })?;
                match key.trim() {
                    "resource" if !value.trim().is_empty() => {
                        resource = Some(value.trim().to_string())
                    }
                    "kind" if !value.trim().is_empty() => kind = Some(value.trim().to_string()),
                    other => {
                        return Err(OkfError::Usage(format!(
                            "invalid --add-source key {other:?}: expected resource and kind"
                        )))
                    }
                }
            }
            let resource = resource.ok_or_else(|| {
                OkfError::Usage("--add-source requires resource=<value>".to_string())
            })?;
            let kind = kind
                .ok_or_else(|| OkfError::Usage("--add-source requires kind=<value>".to_string()))?;
            Ok(Source {
                resource,
                kind: SourceKind::from_kind_str(&kind),
                fingerprint: Fingerprint::default(),
                extra: Default::default(),
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
            "last_modified": r.last_modified,
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
