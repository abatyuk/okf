//! Artifact inventory, resolution, bounded retrieval, and explicit writes.

use crate::cli::{
    ArtifactListArgs, ArtifactPutArgs, ArtifactResolveArgs, ArtifactShowArgs, IdArgs,
};
use crate::output;
use okf_core::bundle::loader::load_concept;
use okf_core::bundle::resolve::resolve_bundle;
use okf_core::error::{OkfError, Result};
use okf_core::model::standard::parse_timestamp;
use okf_core::ports::clock::{Clock, SystemClock};
use okf_core::query::artifact::{
    fetch_artifact, list_artifacts, put_artifact, resolve_artifact, show_artifact, ArtifactKind,
    ArtifactResolver,
};
use okf_core::query::computation::inspect_concept;
use serde_json::json;

pub fn run_list(args: &ArtifactListArgs, json_output: bool) -> Result<i32> {
    check_list_bundle_argument(args.bundle.as_deref())?;
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
            output::print_text_line(format_args!(
                "{}\t{}\t{}",
                entry.kind.as_str(),
                entry.size,
                entry.path.display()
            ))?;
        }
    }
    Ok(0)
}

fn check_list_bundle_argument(bundle: Option<&str>) -> Result<()> {
    let Some(bundle) = bundle else {
        return Ok(());
    };
    let Ok(default_root) = resolve_bundle(None) else {
        return Ok(());
    };
    let cwd = std::env::current_dir()?;
    let configured = std::env::var("OKF_BUNDLE").is_ok_and(|s| !s.is_empty())
        || okf_core::bundle::config::config_bundle(&cwd)
            .ok()
            .flatten()
            .is_some()
        || default_root.join("index.md").is_file()
        || default_root.join("ontology.yaml").is_file();
    if !configured {
        return Ok(());
    }
    let default_root = default_root.canonicalize()?;
    let candidate = std::path::Path::new(bundle)
        .canonicalize()
        .ok()
        .or_else(|| default_root.join(bundle).canonicalize().ok());
    if let Some(candidate) = candidate {
        if candidate != default_root {
            if let Ok(directory) = candidate.strip_prefix(&default_root) {
                return Err(OkfError::Usage(format!(
                    "artifact list positional argument selects a bundle root; {} is inside bundle {}; use --directory {}",
                    bundle, default_root.display(), directory.display()
                )));
            }
        }
    }
    Ok(())
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
        output::print_text_line(format_args!(
            "{}\t{}\t{}",
            result.kind.as_str(),
            result
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| result.resource.clone()),
            result.message.unwrap_or_default()
        ))?;
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
    if resolved.kind == ArtifactKind::Missing {
        let canonical_root = root.canonicalize()?;
        let suggestion = if args.from.is_none() {
            std::path::Path::new(&args.resource)
                .canonicalize()
                .ok()
                .and_then(|path| {
                    path.strip_prefix(&canonical_root)
                        .ok()
                        .map(|rel| rel.to_path_buf())
                })
                .map(|rel| format!("; did you mean {}?", rel.display()))
                .unwrap_or_default()
        } else {
            String::new()
        };
        return Err(OkfError::Usage(format!(
            "artifact not found: {}; paths are resolved relative to the bundle root {} (or the declaring concept with --from){}",
            args.resource, canonical_root.display(), suggestion
        )));
    }
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

pub fn run_put(args: &ArtifactPutArgs, json_output: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let input = args
        .input
        .strip_prefix('@')
        .filter(|path| !path.is_empty())
        .ok_or_else(|| OkfError::Usage("artifact put input must be @file".to_string()))?;
    let bytes = std::fs::read(input).map_err(|e| OkfError::Io(format!("{input}: {e}")))?;
    let written = put_artifact(&root, &args.resource, &bytes, args.replace)?;
    if json_output {
        output::print_line(&json!({
            "kind": "artifact-write",
            "path": written.path.to_string_lossy(),
            "artifact_kind": written.kind.as_str(),
            "size": written.size,
            "sha256": written.sha256,
            "replaced": written.replaced,
            "source_fingerprints": "unchanged",
        }))?;
    } else {
        output::print_text_line(format_args!(
            "{} {} ({} bytes; source fingerprints unchanged)",
            if written.replaced {
                "replaced"
            } else {
                "created"
            },
            written.path.display(),
            written.size,
        ))?;
    }
    Ok(0)
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
        output::print_text_line(format_args!(
            "binary\t{}\t{}",
            content.sha256,
            content.resolved.path.unwrap().display()
        ))?;
    } else if let Some(text) = content.text {
        output::print_text(format_args!("{text}"))?;
        if !text.ends_with('\n') {
            output::print_text_line(format_args!(""))?;
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
    let concept = load_concept(&root, &args.concept)?
        .ok_or_else(|| OkfError::Usage(format!("concept not found: {}", args.concept)))?;
    let contract = inspect_concept(&concept);
    let resolver = ArtifactResolver::new(&root);
    let resolve = |resource: &Option<String>| {
        resource
            .as_deref()
            .map(|r| resolver.resolve(Some(&contract.concept.id.0), r))
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
        output::print_text_line(format_args!(
            "{}\truntime={}\tvalid={}\tstatus={}\ttrust={}\tstale={}\texecution=not-run",
            contract.concept.id.0,
            contract.runtime.as_deref().unwrap_or("-"),
            issues.is_empty(),
            contract.concept.effective_status(),
            contract.concept.trust_tier().as_str(),
            stale.map(|v| v.to_string()).as_deref().unwrap_or("unknown")
        ))?;
        for issue in &issues {
            output::print_text_line(format_args!("  issue: {issue}"))?;
        }
    }
    Ok(if issues.is_empty() { 0 } else { 1 })
}
