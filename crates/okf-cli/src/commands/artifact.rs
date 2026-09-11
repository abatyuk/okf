//! Read-only artifact inventory, resolution, and bounded retrieval.

use crate::cli::{ArtifactListArgs, ArtifactResolveArgs, ArtifactShowArgs, IdArgs};
use crate::output;
use okf_core::bundle::loader::load_bundle;
use okf_core::bundle::resolve::resolve_bundle;
use okf_core::error::{OkfError, Result};
use okf_core::model::standard::parse_timestamp;
use okf_core::ports::clock::{Clock, SystemClock};
use okf_core::query::artifact::{
    fetch_artifact, list_artifacts, resolve_artifact, show_artifact, ArtifactKind,
};
use okf_core::query::computation::inspect;
use serde_json::json;

pub fn run_list(args: &ArtifactListArgs, json_output: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let entries = list_artifacts(&root, Some(&args.directory), args.digest)?;
    if json_output {
        for entry in entries {
            output::print_line(&json!({
                "kind": "artifact",
                "path": entry.path.to_string_lossy(),
                "artifact_kind": entry.kind.as_str(),
                "size": entry.size,
                "sha256": entry.sha256,
            }))?;
        }
    } else if entries.is_empty() {
        println!("(no artifacts)");
    } else {
        for entry in entries {
            println!(
                "{}\t{}\t{}",
                entry.kind.as_str(),
                entry.size,
                entry.path.display()
            );
        }
    }
    Ok(0)
}

pub fn run_resolve(args: &ArtifactResolveArgs, json_output: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let result = resolve_artifact(&root, args.from.as_deref(), &args.resource);
    if json_output {
        output::print_line(&json!({
            "kind": "artifact-resolution",
            "resource": result.resource,
            "artifact_kind": result.kind.as_str(),
            "path": result.path.map(|p| p.to_string_lossy().into_owned()),
            "exists": result.exists,
            "size": result.size,
            "message": result.message,
        }))?;
    } else {
        println!(
            "{}\t{}\t{}",
            result.kind.as_str(),
            result
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| result.resource.clone()),
            result.message.unwrap_or_default()
        );
    }
    Ok(
        if matches!(result.kind, ArtifactKind::Missing | ArtifactKind::Blocked) {
            1
        } else {
            0
        },
    )
}

pub fn run_show(args: &ArtifactShowArgs, json_output: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let resolved = resolve_artifact(&root, args.from.as_deref(), &args.resource);
    if resolved.kind == ArtifactKind::External {
        if !args.fetch {
            return Err(OkfError::Environment(
                "remote resource requires explicit --fetch and a network-enabled policy"
                    .to_string(),
            ));
        }
        let content = fetch_artifact(&args.resource, args.max_bytes)?;
        return print_content(content, json_output);
    }
    let lines = args.lines.as_deref().map(parse_lines).transpose()?;
    let content = show_artifact(
        &root,
        args.from.as_deref(),
        &args.resource,
        args.max_bytes,
        lines,
    )?;
    print_content(content, json_output)
}

fn print_content(
    content: okf_core::query::artifact::ArtifactContent,
    json_output: bool,
) -> Result<i32> {
    if json_output {
        output::print_line(&json!({
            "kind": "artifact-content",
            "resource": content.resolved.resource,
            "path": content.resolved.path.map(|p| p.to_string_lossy().into_owned()),
            "size": content.resolved.size,
            "sha256": content.sha256,
            "binary": content.binary,
            "truncated": content.truncated,
            "text": content.text,
        }))?;
    } else if content.binary {
        println!(
            "binary\t{}\t{}",
            content.sha256,
            content.resolved.path.unwrap().display()
        );
    } else if let Some(text) = content.text {
        print!("{text}");
        if !text.ends_with('\n') {
            println!();
        }
    }
    Ok(0)
}

fn parse_lines(raw: &str) -> Result<(usize, usize)> {
    let (start, end) = raw.split_once(':').unwrap_or((raw, raw));
    let start = start.parse::<usize>().ok().filter(|n| *n > 0);
    let end = end.parse::<usize>().ok().filter(|n| *n > 0);
    match (start, end) {
        (Some(start), Some(end)) if start <= end => Ok((start, end)),
        _ => Err(OkfError::Usage(format!(
            "invalid --lines {raw:?}: expected positive START:END"
        ))),
    }
}

pub fn run_computation_check(args: &IdArgs, json_output: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let contract = inspect(&bundle, &args.concept)?;
    let resolve = |resource: &Option<String>| {
        resource
            .as_deref()
            .map(|r| resolve_artifact(&root, Some(&contract.concept.id.0), r))
    };
    let computation = resolve(&contract.computation);
    let executor = resolve(&contract.executor_resource);
    let attester = resolve(&contract.attester_resource);
    let mut issues = contract.issues.clone();
    for (name, resolved) in [
        ("computation", computation.as_ref()),
        ("executor", executor.as_ref()),
        ("attester", attester.as_ref()),
    ] {
        if resolved.is_some_and(|r| matches!(r.kind, ArtifactKind::Missing | ArtifactKind::Blocked))
        {
            issues.push(format!("{name} resource is missing or blocked"));
        }
    }
    let stale = contract
        .concept
        .frontmatter
        .get_str("stale_after")
        .and_then(parse_timestamp)
        .zip(parse_timestamp(&SystemClock.now_rfc3339()))
        .map(|(deadline, now)| now >= deadline);
    if json_output {
        let artifact_json = |r: Option<okf_core::query::artifact::ResolvedArtifact>| {
            r.map(|r| {
                json!({
                    "resource": r.resource,
                    "kind": r.kind.as_str(),
                    "path": r.path.map(|p| p.to_string_lossy().into_owned()),
                    "exists": r.exists,
                    "message": r.message,
                })
            })
        };
        output::print_line(&json!({
            "kind": "computation-contract",
            "id": contract.concept.id.0,
            "valid": issues.is_empty(),
            "effective_status": contract.concept.effective_status(),
            "trust_tier": contract.concept.trust_tier().as_str(),
            "stale": stale,
            "verification_current": contract.concept.verification_current(),
            "runtime": contract.runtime,
            "parameters": contract.parameters.iter().map(|(name, ty, required)| json!({"name":name,"type":ty,"required":required})).collect::<Vec<_>>(),
            "inline": contract.inline,
            "computation": artifact_json(computation),
            "executor": artifact_json(executor),
            "receipt": contract.receipt,
            "attester": artifact_json(attester),
            "issues": issues,
            "execution": "not-run",
        }))?;
    } else {
        println!(
            "{}\truntime={}\tvalid={}\tstatus={}\ttrust={}\tstale={}\texecution=not-run",
            contract.concept.id.0,
            contract.runtime.as_deref().unwrap_or("-"),
            issues.is_empty(),
            contract.concept.effective_status(),
            contract.concept.trust_tier().as_str(),
            stale.map(|v| v.to_string()).as_deref().unwrap_or("unknown")
        );
        for issue in &issues {
            println!("  issue: {issue}");
        }
    }
    Ok(if issues.is_empty() { 0 } else { 1 })
}
