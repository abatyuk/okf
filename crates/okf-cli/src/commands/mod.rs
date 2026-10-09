//! One thin function per command: parse args -> call okf-core -> hand result to output.
//!
//! Handlers return the *success* exit code: `0` for ok/clean, `1` for reportable findings
//! at/above a command's threshold. Errors bubble as [`OkfError`] and are mapped to 2/3/4 by
//! [`crate::exit`]. Core never calls `process::exit`.
pub mod artifact;
pub mod catalog;
pub mod changeset;
pub mod check;
pub mod mutate;
pub mod ontology;
pub mod query;
pub mod render;
pub mod schema;

use crate::cli::{ArtifactCmd, Cli, Command, ComputationCmd, OntologyCmd};
use okf_core::error::Result;

/// Dispatch a parsed CLI invocation to its command handler, returning an exit code.
pub fn run(mut cli: Cli) -> Result<i32> {
    match &cli.command {
        Command::Links(a) | Command::Computation(ComputationCmd::Check(a)) if a.details => eprintln!("okf: --details is retained for compatibility and has no effect for this command"),
        Command::Lint(a) if a.fix => eprintln!("okf: --fix is retained for compatibility; no automatic lint fixes are implemented (use doctor --fix-safe for supported repairs)"),
        Command::Affected(a) if a.depth.is_some() && !a.transitive => eprintln!("okf: --depth has no effect without --transitive"),
        Command::Init(a) if a.title.is_some() && a.no_index => eprintln!("okf: --title has no effect with --no-index"),
        _ => {},
    }

    let failure_alias = match &cli.command {
        Command::Scan(a) | Command::Stale(a) | Command::Stats(a) => a.fail_on.as_deref(),
        Command::Affected(a) => a.fail_on.as_deref(),
        Command::Diff(a) => a.fail_on.as_deref(),
        _ => None,
    };
    if matches!(failure_alias, Some("info" | "warn" | "error")) {
        eprintln!("okf: discovery --fail-on {} is a compatibility alias for any; results have no severity", failure_alias.unwrap());
    }
    if let Some(code) = catalog::prepare(&mut cli)? {
        return Ok(code);
    }
    // Hold the bundle writer lock through ordinary read-modify-write handlers.
    // mv and changeset acquire it around their guarded transactional publication.
    let _writer = if catalog::writes_command(&cli.command) && !matches!(cli.command, Command::Mv(_))
    {
        let path = catalog::bundle_slot(&mut cli.command).and_then(|p| p.clone());
        let root = okf_core::bundle::resolve::resolve_bundle_target(path.as_deref())?;
        if root.is_dir() {
            Some(okf_core::mutate::transaction::writer_lock(&root)?)
        } else {
            None
        }
    } else {
        None
    };
    let json = cli.json;
    match &cli.command {
        // META
        Command::Schema => schema::run_schema(json),
        Command::Catalog => catalog::run_catalog(json),
        Command::Version => schema::run_version(json),
        // QUERY
        Command::List(a) => query::run_list(a, json),
        Command::Search(a) => query::run_search(a, json),
        Command::Show(a) => query::run_show(a, json),
        Command::Browse(a) => query::run_browse(a, json),
        Command::Backlinks(a) => query::run_backlinks(a, json),
        Command::Links(a) => query::run_links(a, json),
        Command::Graph(a) => query::run_graph(a, json),
        Command::Resolve(a) => query::run_resolve(a, json),
        Command::Artifact(cmd) => match cmd {
            ArtifactCmd::List(a) => artifact::run_list(a, json),
            ArtifactCmd::Resolve(a) => artifact::run_resolve(a, json),
            ArtifactCmd::Show(a) => artifact::run_show(a, json),
            ArtifactCmd::Put(a) => artifact::run_put(a, json),
        },
        Command::Computation(cmd) => match cmd {
            ComputationCmd::Check(a) => artifact::run_computation_check(a, json),
        },
        // CHECK
        Command::Scan(a) => check::run_scan(a, json),
        Command::SourceScan(a) => check::run_source_scan(a, json),
        Command::Validate(a) => check::run_validate(a, json),
        Command::Lint(a) => check::run_lint(a, json),
        Command::Stale(a) => check::run_stale(a, json),
        Command::Affected(a) => check::run_affected(a, json),
        Command::Diff(a) => check::run_diff(a, json),
        Command::Stats(a) => check::run_stats(a, json),
        Command::Doctor(a) => check::run_doctor(a, json),
        // MUTATE
        Command::Init(a) => mutate::run_init(a, json),
        Command::Add(a) => mutate::run_add(a, json),
        Command::Edit(a) => mutate::run_edit(a, json),
        Command::Mv(a) => mutate::run_mv(a, json),
        Command::Rm(a) => mutate::run_rm(a, json),
        Command::Verify(a) => mutate::run_verify(a, json),
        Command::Refresh(a) => mutate::run_refresh(a, json),
        // ontology (query + mutate)
        Command::Ontology(cmd) => match cmd {
            OntologyCmd::List(a) => ontology::run_list(a, json),
            OntologyCmd::Show(a) => ontology::run_show(a, json),
            OntologyCmd::Add(a) => ontology::run_add(a, json),
            OntologyCmd::Update(a) => ontology::run_update(a, json),
            OntologyCmd::Remove(a) => ontology::run_remove(a, json),
            OntologyCmd::FieldType(cmd) => ontology::run_field_type(cmd, json),
            OntologyCmd::Apply(a) => ontology::run_apply(a, json),
        },
        Command::Changeset(a) => changeset::run(a, json),
        // RENDER
        Command::Docs(a) => render::run_docs(a, json),
    }
}
