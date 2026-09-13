//! scan, validate, lint, stale, affected, diff, stats.
use std::io::{BufRead, IsTerminal};

use crate::cli::{
    AffectedArgs, BundleArgs, DiffArgs, DoctorArgs, FailOnArgs, LintArgs, SourceScanArgs,
};
use crate::output;
use okf_core::bundle::loader::{load_bundle, load_bundle_metadata};
use okf_core::bundle::resolve::resolve_bundle;
use okf_core::bundle::walk::{walk_files, walk_markdown};
use okf_core::check::doctor::doctor;
use okf_core::check::lint::{lint_bundle, meets_threshold, FailOn, LintConfig};
use okf_core::check::stale::check_stale;
use okf_core::check::validate::validate_bundle;
use okf_core::error::{OkfError, Result};
use okf_core::graph::affected::{affected, AffectedOptions};
use okf_core::graph::build::build_graph;
use okf_core::ontology::load::try_load;
use okf_core::ports::git::RealGit;
use okf_core::query::diff::diff;
use okf_core::query::stats::stats;
use serde_json::json;
use std::path::Path;
use std::str::FromStr;

/// `okf scan [bundle]` — report the candidate concept files the loader would analyze.
pub fn run_scan(args: &BundleArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let paths = walk_markdown(&root)?;
    if json {
        for path in &paths {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            output::print_line(&json!({
                "kind": "scan",
                "file": path.to_string_lossy(),
                "reserved": matches!(name, "index.md" | "log.md"),
            }))?;
        }
    } else {
        output::print_text_line(format_args!("{} Markdown file(s):", paths.len()))?;
        for path in &paths {
            output::print_text_line(format_args!("  {}", path.display()))?;
        }
    }
    Ok(0)
}

pub fn run_source_scan(args: &SourceScanArgs, json_output: bool) -> Result<i32> {
    let root = Path::new(&args.directory);
    let files = walk_files(root)?;
    if json_output {
        for path in files {
            let abs = root.join(&path);
            let size = std::fs::metadata(&abs)
                .map_err(|e| OkfError::Io(format!("{}: {e}", abs.display())))?
                .len();
            output::print_line(&json!({
                "kind": "source-file",
                "file": path.to_string_lossy(),
                "extension": path.extension().and_then(|s| s.to_str()),
                "size": size,
            }))?;
        }
    } else {
        for path in files {
            output::print_text_line(format_args!("{}", path.display()))?;
        }
    }
    Ok(0)
}

/// `okf validate [bundle]` — conformance (three hard rules). Exit 1 on nonconformance.
pub fn run_validate(args: &BundleArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let report = validate_bundle(&root)?;
    if json {
        for v in &report.violations {
            output::print_line(&json!({
                "kind": "violation",
                "file": v.file,
                "rule": v.rule.as_str(),
                "message": v.message,
            }))?;
        }
    } else if report.is_conformant() {
        output::print_text_line(format_args!("conformant: no violations"))?;
    } else {
        for v in &report.violations {
            output::print_text_line(format_args!(
                "{}\t{}\t{}",
                v.file,
                v.rule.as_str(),
                v.message
            ))?;
        }
        eprintln!("{} violation(s)", report.violations.len());
    }
    Ok(if report.is_conformant() { 0 } else { 1 })
}

/// `okf lint [bundle] [--fix] [--fail-on <sev>]`.
pub fn run_lint(args: &LintArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let ontology = try_load(&root)?;
    let config = LintConfig::default();
    let findings = lint_bundle(&bundle, ontology.as_ref(), &config);

    if json {
        for f in &findings {
            output::print_line(&json!({
                "kind": "finding",
                "rule": f.rule,
                "severity": f.severity.as_str(),
                "concept": f.concept,
                "message": f.message,
            }))?;
        }
    } else {
        if findings.is_empty() {
            output::print_text_line(format_args!("clean: no findings"))?;
        }
        for f in &findings {
            output::print_text_line(format_args!(
                "{}\t{}\t{}\t{}",
                f.severity.as_str(),
                f.rule,
                f.concept.as_deref().unwrap_or("-"),
                f.message
            ))?;
        }
    }

    // `--fix` is a documented v1 no-op: no lint rule is auto-fixable yet (core has no fixer).
    if args.fix && !json {
        eprintln!("--fix: 0 findings auto-fixable in v1 (no fixer implemented)");
    }

    let fail_on = match &args.fail_on {
        Some(s) => FailOn::from_str(s)?,
        None => FailOn::default(), // error
    };
    Ok(if meets_threshold(&findings, fail_on) {
        1
    } else {
        0
    })
}

/// `okf stale [bundle] [--fail-on <sev>]` — informational unless `--fail-on` is set.
pub fn run_stale(args: &FailOnArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle_metadata(&root)?;
    let report = check_stale(&bundle);

    if json {
        for cd in &report.concepts {
            let sources: Vec<_> = cd
                .sources
                .iter()
                .map(|s| json!({"resource": s.resource, "kind": s.kind, "drift": s.drift.as_str(), "message": s.message}))
                .collect();
            output::print_line(&json!({
                "kind": "drift",
                "concept": cd.concept,
                "expired": cd.expired,
                "sources": sources,
            }))?;
        }
    } else if report.is_empty() {
        output::print_text_line(format_args!("in sync: nothing drifted"))?;
    } else {
        for cd in &report.concepts {
            if let Some(exp) = &cd.expired {
                output::print_text_line(format_args!(
                    "{}\texpired\tstale_after {exp}",
                    cd.concept
                ))?;
            }
            for s in &cd.sources {
                output::print_text_line(format_args!(
                    "{}\t{}\t{}\t{}",
                    cd.concept,
                    s.drift.as_str(),
                    s.resource,
                    s.message
                ))?;
            }
        }
    }
    discovery_exit(&args.fail_on, report.is_empty())
}

/// `okf affected [bundle] --changed <..> [--transitive] [--depth N]` (also reads stdin).
pub fn run_affected(args: &AffectedArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let ontology = try_load(&root)?;

    let mut changed = args.changed.clone();
    // Also accept changed links piped on stdin (one per line), combined with `--changed`.
    if !std::io::stdin().is_terminal() {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let line = line.map_err(|e| OkfError::Io(e.to_string()))?;
            let t = line.trim();
            if !t.is_empty() {
                changed.push(t.to_string());
            }
        }
    }
    if changed.is_empty() {
        return Err(OkfError::Usage(
            "affected: provide at least one --changed link (or pipe them on stdin)".to_string(),
        ));
    }

    let graph = build_graph(&bundle, ontology.as_ref());
    let opts = AffectedOptions {
        transitive: args.transitive,
        depth: args.depth,
    };
    let ids = affected(&graph, &changed, &opts);

    if json {
        for id in &ids {
            output::print_line(&json!({"kind": "affected", "concept": id.0}))?;
        }
    } else if ids.is_empty() {
        output::print_text_line(format_args!("no affected concepts"))?;
    } else {
        for id in &ids {
            output::print_text_line(format_args!("{}", id.0))?;
        }
    }
    discovery_exit(&args.fail_on, ids.is_empty())
}

/// `okf diff <git-ref> [bundle] [--fail-on <sev>]`.
pub fn run_diff(args: &DiffArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let git = RealGit;
    let d = diff(&bundle, &git, &args.git_ref)?;

    if json {
        for (change, ids) in [
            ("added", &d.added),
            ("removed", &d.removed),
            ("modified", &d.modified),
        ] {
            for id in ids {
                output::print_line(&json!({"kind": "diff", "change": change, "concept": id.0}))?;
            }
        }
    } else if d.is_empty() {
        output::print_text_line(format_args!("no changes vs {}", args.git_ref))?;
    } else {
        for id in &d.added {
            output::print_text_line(format_args!("added\t{}", id.0))?;
        }
        for id in &d.removed {
            output::print_text_line(format_args!("removed\t{}", id.0))?;
        }
        for id in &d.modified {
            output::print_text_line(format_args!("modified\t{}", id.0))?;
        }
    }
    discovery_exit(&args.fail_on, d.is_empty())
}

/// `okf stats [bundle] [--fail-on <sev>]`.
pub fn run_stats(args: &FailOnArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let ontology = try_load(&root)?;
    let s = stats(&bundle, ontology.as_ref());

    if json {
        output::print_line(&json!({
            "kind": "stats",
            "total": s.total,
            "by_type": serde_json::to_value(&s.by_type).unwrap_or(serde_json::Value::Null),
            "trust": {
                "unverified": s.trust.unverified,
                "machine_confirmed": s.trust.machine_confirmed,
                "human_reviewed": s.trust.human_reviewed,
            },
            "stale": s.stale,
            "orphans": s.orphans,
        }))?;
    } else {
        println!("total: {}", s.total);
        println!("by type:");
        for (ty, n) in &s.by_type {
            println!("  {ty}: {n}");
        }
        println!("trust:");
        println!("  unverified: {}", s.trust.unverified);
        println!("  machine-confirmed: {}", s.trust.machine_confirmed);
        println!("  human-reviewed: {}", s.trust.human_reviewed);
        println!("orphans: {}", s.orphans);
        if let Some(stale) = s.stale {
            println!("stale: {stale}");
        }
    }
    // Stats is purely informational; only `--fail-on` (non-never) with a non-empty bundle fails.
    discovery_exit(&args.fail_on, s.total == 0)
}

/// Exit code for the informational discovery commands: `0` unless `--fail-on` is set to
/// something other than `never` and there is at least one result.
fn discovery_exit(fail_on: &Option<String>, empty: bool) -> Result<i32> {
    match fail_on {
        None => Ok(0),
        Some(s) => {
            let fo = FailOn::from_str(s)?;
            Ok(if !matches!(fo, FailOn::Never) && !empty {
                1
            } else {
                0
            })
        }
    }
}

/// Compatibility-first preflight and conservative repair for existing bundles.
pub fn run_doctor(args: &DoctorArgs, json_output: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let apply = args.fix_safe && args.yes && !args.dry_run;
    let report = doctor(&root, &args.target, args.fix_safe, apply)?;
    if json_output {
        for finding in &report.findings {
            output::print_line(&json!({
                "kind": "doctor-finding",
                "id": finding.id,
                "rule": finding.rule,
                "path": finding.path,
                "severity": finding.severity.as_str(),
                "repair": finding.repair.as_str(),
                "message": finding.message,
                "current": finding.current,
                "target": finding.target,
                "proposed_action": finding.proposed_action,
                "applied": finding.applied,
            }))?;
        }
        output::print_line(&json!({
            "kind": "doctor-summary",
            "target": report.target,
            "ready": report.ready(),
            "files_inspected": report.files_inspected,
            "concepts_inspected": report.concepts_inspected,
            "findings": report.findings.len(),
            "applied": apply,
        }))?;
    } else {
        for finding in &report.findings {
            output::print_text_line(format_args!(
                "{}\t{}\t{}\t{}\t{}",
                finding.severity.as_str(),
                finding.id,
                finding.repair.as_str(),
                finding.path.as_deref().unwrap_or("-"),
                finding.message
            ))?;
        }
        output::print_text_line(format_args!(
            "doctor: {} file(s), {} concept(s), {} finding(s), ready={}",
            report.files_inspected,
            report.concepts_inspected,
            report.findings.len(),
            report.ready()
        ))?;
        if args.fix_safe && !apply {
            eprintln!("safe fixes were dry-run only; pass --fix-safe --yes to apply");
        }
    }
    Ok(if report.ready() { 0 } else { 1 })
}
