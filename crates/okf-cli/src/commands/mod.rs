//! One thin function per command: parse args -> call okf-core -> hand result to output.
//!
//! Handlers return the *success* exit code: `0` for ok/clean, `1` for reportable findings
//! at/above a command's threshold. Errors bubble as [`OkfError`] and are mapped to 2/3/4 by
//! [`crate::exit`]. Core never calls `process::exit`.
pub mod check;
pub mod mutate;
pub mod ontology;
pub mod query;
pub mod render;
pub mod schema;

use crate::cli::{Cli, Command, OntologyCmd};
use okf_core::error::Result;

/// Dispatch a parsed CLI invocation to its command handler, returning an exit code.
pub fn run(cli: Cli) -> Result<i32> {
    let json = cli.json;
    match &cli.command {
        // META
        Command::Schema => schema::run_schema(json),
        Command::Version => schema::run_version(json),
        // QUERY
        Command::List(a) => query::run_list(a, json),
        Command::Search(a) => query::run_search(a, json),
        Command::Show(a) => query::run_show(a, json),
        Command::Browse(a) => query::run_browse(a, json),
        Command::Backlinks(a) => query::run_backlinks(a, json),
        Command::Graph(a) => query::run_graph(a, json),
        Command::Resolve(a) => query::run_resolve(a, json),
        // CHECK
        Command::Scan(a) => check::run_scan(a, json),
        Command::Validate(a) => check::run_validate(a, json),
        Command::Lint(a) => check::run_lint(a, json),
        Command::Stale(a) => check::run_stale(a, json),
        Command::Affected(a) => check::run_affected(a, json),
        Command::Diff(a) => check::run_diff(a, json),
        Command::Stats(a) => check::run_stats(a, json),
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
        },
        // RENDER
        Command::Docs(a) => render::run_docs(a, json),
    }
}
