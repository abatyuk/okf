//! clap command tree, groups, global `--json`. `okf schema` is derived from this tree.
use clap::{Args, Parser, Subcommand};

/// Open Knowledge Format harness — deterministic hands over markdown + YAML bundles.
#[derive(Debug, Parser)]
#[command(name = "okf", version, about, long_about = None)]
pub struct Cli {
    /// Emit NDJSON (one JSON object per line) instead of human text.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    // ---- META ----
    /// Print machine-readable CLI metadata (all commands, args, output shapes) as NDJSON.
    Schema,
    /// Print the CLI version and the OKF spec version(s) it supports.
    Version,

    // ---- QUERY ----
    /// List all concepts (search with no filter).
    List(BundleArgs),
    /// Search concepts by type, tag, text, and/or frontmatter field.
    Search(SearchArgs),
    /// Show one concept's content, heading outline, or selected line range.
    Show(ShowArgs),
    /// Show a directory's index.md, synthesizing it when absent.
    Browse(BrowseArgs),
    /// Concepts that link to a given concept.
    Backlinks(IdArgs),
    /// Render the link graph (or a subtree) as mermaid/dot/graphml.
    Graph(GraphArgs),
    /// Resolve a link/concept-id to a concrete bundle-relative file path.
    Resolve(ResolveArgs),

    // ---- CHECK ----
    /// Walk a bundle and report the candidate files that would be analyzed.
    Scan(BundleArgs),
    /// Conformance validation — the spec's three hard rules only.
    Validate(BundleArgs),
    /// Advisory checks (broken links, missing fields, orphans, ontology violations).
    Lint(LintArgs),
    /// Drift detection: recorded vs. recomputed source fingerprints (+ `stale_after`).
    Stale(FailOnArgs),
    /// Impact query: concepts needing review given a set of changed links.
    Affected(AffectedArgs),
    /// Concept-level diff of the working tree vs a git ref.
    Diff(DiffArgs),
    /// Bundle summary: counts by type, trust distribution, orphans.
    Stats(FailOnArgs),

    // ---- MUTATE ----
    /// Create a new empty OKF bundle.
    Init(InitArgs),
    /// Add a new concept document, scaffolded from the ontology.
    Add(AddArgs),
    /// Edit a concept losslessly and invalidate its prior verification.
    Edit(EditArgs),
    /// Move/rename a concept and rewrite every inbound link.
    Mv(MvArgs),
    /// Remove a concept; refuse if backlinks would dangle unless `--force`.
    Rm(RmArgs),
    /// Append a `verified` entry (the write-side of trust).
    Verify(VerifyArgs),
    /// Re-record source fingerprints after a change is acknowledged.
    Refresh(RefreshArgs),

    // ---- MUTATE / QUERY: ontology ----
    /// Inspect or edit the tool-local `ontology.yaml`.
    #[command(subcommand)]
    Ontology(OntologyCmd),

    // ---- RENDER ----
    /// Generate documentation from a bundle.
    Docs(DocsArgs),
}

#[derive(Debug, Subcommand)]
pub enum OntologyCmd {
    /// List the defined concept types.
    List(BundleArgs),
    /// Show one concept type and its rules.
    Show(OntShowArgs),
    /// Define a new concept type with its fields and reference rules.
    Add(OntEditArgs),
    /// Modify fields/references of an existing concept type.
    Update(OntEditArgs),
    /// Remove a concept type.
    Remove(OntRemoveArgs),
}

/// A bare bundle positional, shared by commands that take no other argument.
#[derive(Debug, Args)]
pub struct BundleArgs {
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Filter by exact concept `type`.
    #[arg(long = "type")]
    pub type_: Option<String>,
    /// Filter by membership in the concept's `tags`.
    #[arg(long = "tag")]
    pub tag: Option<String>,
    /// Filter by case-insensitive substring across id/title/description/body.
    #[arg(long = "text")]
    pub text: Option<String>,
    /// Filter by a frontmatter field, `key=value` (repeatable; AND).
    #[arg(long = "field")]
    pub field: Vec<String>,
}

#[derive(Debug, Args)]
pub struct IdArgs {
    /// Concept id (leading slash optional), e.g. `tables/customers`.
    pub concept: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    /// Concept id (leading slash optional), e.g. `tables/customers`.
    pub concept: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Show only the Markdown heading outline with 1-based document line numbers.
    #[arg(long, conflicts_with = "lines")]
    pub outline: bool,
    /// Show only an inclusive 1-based document line range, `START:END` (or one line, `N`).
    #[arg(long, value_name = "START:END", conflicts_with = "outline")]
    pub lines: Option<String>,
}

#[derive(Debug, Args)]
pub struct BrowseArgs {
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Bundle-relative directory to browse (default: root `/`).
    #[arg(long, default_value = "/")]
    pub directory: String,
}

#[derive(Debug, Args)]
pub struct GraphArgs {
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Optional subtree root: only the subgraph forward-reachable from this concept.
    pub subtree: Option<String>,
    /// Output format: mermaid (default), dot, or graphml.
    #[arg(long, default_value = "mermaid")]
    pub format: String,
}

#[derive(Debug, Args)]
pub struct ResolveArgs {
    /// Link or concept id to resolve.
    pub link: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Resolve a relative link against this containing concept id.
    #[arg(long)]
    pub from: Option<String>,
}

#[derive(Debug, Args)]
pub struct LintArgs {
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Apply auto-fixable findings (v1: none are auto-fixable — reports what it would do).
    #[arg(long)]
    pub fix: bool,
    /// Severity threshold that makes the run fail (exit 1): never|info|warn|error|any.
    #[arg(long = "fail-on")]
    pub fail_on: Option<String>,
}

#[derive(Debug, Args)]
pub struct FailOnArgs {
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Fail (exit 1) on any result: never (default) | info | warn | error | any.
    #[arg(long = "fail-on")]
    pub fail_on: Option<String>,
}

#[derive(Debug, Args)]
pub struct AffectedArgs {
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// A changed link/concept-id/resource (repeatable; also read from stdin lines).
    #[arg(long = "changed")]
    pub changed: Vec<String>,
    /// Follow the cascade past direct dependents.
    #[arg(long)]
    pub transitive: bool,
    /// Cap the number of hops when `--transitive`.
    #[arg(long)]
    pub depth: Option<usize>,
    /// Fail (exit 1) on any affected concept: never (default) | info | warn | error | any.
    #[arg(long = "fail-on")]
    pub fail_on: Option<String>,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    /// Git ref to diff against (e.g. `HEAD`, a branch, or a commit).
    pub git_ref: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Fail (exit 1) on any change: never (default) | info | warn | error | any.
    #[arg(long = "fail-on")]
    pub fail_on: Option<String>,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Bundle directory to create (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Title for the scaffolded root `index.md`.
    #[arg(long)]
    pub title: Option<String>,
    /// Do not scaffold a root `index.md`.
    #[arg(long = "no-index")]
    pub no_index: bool,
    /// Do not scaffold an `ontology.yaml`.
    #[arg(long = "no-ontology")]
    pub no_ontology: bool,
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Bundle-relative path of the new concept (with or without `.md`).
    pub path: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Concept `type` (an ontology concept-type key). Optional with `--attested`.
    #[arg(long = "type")]
    pub type_: Option<String>,
    /// Concept title.
    #[arg(long)]
    pub title: Option<String>,
    /// Concept description.
    #[arg(long)]
    pub description: Option<String>,
    /// Scaffold an OKF Attested Computation (computation/executor/attester).
    #[arg(long)]
    pub attested: bool,
    /// Set a custom scalar field at creation, `key=value` (repeatable).
    #[arg(long = "set")]
    pub set: Vec<String>,
    /// Set a declared reference at creation, `key=link` (repeatable).
    #[arg(long = "ref")]
    pub reference: Vec<String>,
    /// Add a structured source, `resource=<path-or-uri>,kind=<kind>` (repeatable).
    #[arg(long = "add-source")]
    pub add_source: Vec<String>,
}

#[derive(Debug, Args)]
pub struct EditArgs {
    /// Concept id to edit.
    pub concept: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Set/update a scalar field, `key=value` (repeatable).
    #[arg(long = "set")]
    pub set: Vec<String>,
    /// Remove a field entirely, `key` (repeatable).
    #[arg(long = "unset")]
    pub unset: Vec<String>,
    /// Append an item to a list field, `key=value` (repeatable, idempotent).
    #[arg(long = "add")]
    pub add: Vec<String>,
    /// Remove matching item(s) from a list field, `key=value` (repeatable).
    #[arg(long = "remove")]
    pub remove: Vec<String>,
    /// Add a structured source, `resource=<path-or-uri>,kind=<kind>` (repeatable).
    #[arg(long = "add-source")]
    pub add_source: Vec<String>,
    /// Replace the whole body. Use `@file` to read a file or `-` for stdin.
    #[arg(long = "set-body")]
    pub set_body: Option<String>,
    /// Append a block to the body. Use `@file` or `-` (stdin).
    #[arg(long = "append-body")]
    pub append_body: Option<String>,
    /// Empty the body.
    #[arg(long = "clear-body")]
    pub clear_body: bool,
    /// Replace a section's content, `<heading> <text>` (repeatable). Text accepts `@file`/`-`.
    #[arg(long = "set-section", num_args = 2, value_names = ["HEADING", "TEXT"])]
    pub set_section: Vec<String>,
    /// Append to a section, `<heading> <text>` (repeatable). Text accepts `@file`/`-`.
    #[arg(long = "append-section", num_args = 2, value_names = ["HEADING", "TEXT"])]
    pub append_section: Vec<String>,
    /// Remove a section (heading + content), `<heading>` (repeatable).
    #[arg(long = "remove-section")]
    pub remove_section: Vec<String>,
}

#[derive(Debug, Args)]
pub struct RefreshArgs {
    /// Concept id to refresh.
    pub concept: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Fail (exit 1) when any source is skipped: never (default) | skipped | any.
    #[arg(long = "fail-on")]
    pub fail_on: Option<String>,
}

#[derive(Debug, Args)]
pub struct MvArgs {
    /// Existing concept id.
    pub old: String,
    /// New concept id.
    pub new: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
}

#[derive(Debug, Args)]
pub struct RmArgs {
    /// Concept id to remove.
    pub concept: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Remove even if backlinks would dangle.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Concept id to verify.
    pub concept: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// The reviewing actor (e.g. `human:andrey` or `process:ci`).
    #[arg(long = "by")]
    pub by: String,
}

#[derive(Debug, Args)]
pub struct DocsArgs {
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Output format: md|html|pdf|graphml|obsidian|index.
    #[arg(long, default_value = "md")]
    pub format: String,
}

#[derive(Debug, Args)]
pub struct OntShowArgs {
    /// Concept type name.
    pub name: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
}

#[derive(Debug, Args)]
pub struct OntRemoveArgs {
    /// Concept type name to remove.
    pub name: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
}

#[derive(Debug, Args)]
pub struct OntEditArgs {
    /// Concept type name.
    pub name: String,
    /// Bundle directory (defaults to $OKF_BUNDLE, then the current directory).
    pub bundle: Option<String>,
    /// Description of the concept type.
    #[arg(long)]
    pub description: Option<String>,
    /// A typed field, `key:type[:required][:v1|v2|...]` (repeatable).
    #[arg(long = "field")]
    pub field: Vec<String>,
    /// A reference rule, `key:Target[|Target2]:cardinality` (repeatable).
    #[arg(long = "ref")]
    pub reference: Vec<String>,
    /// Remove a typed field (update only; repeatable).
    #[arg(long = "remove-field")]
    pub remove_field: Vec<String>,
    /// Remove a reference rule (update only; repeatable).
    #[arg(long = "remove-ref")]
    pub remove_reference: Vec<String>,
    /// Mark the concept type as an attested computation.
    #[arg(long)]
    pub attested: bool,
}
