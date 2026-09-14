//! clap command tree, groups, global `--json`. `okf schema` is derived from this tree.
use clap::{Args, Parser, Subcommand, ValueEnum};

/// Open Knowledge Format harness — deterministic hands over markdown + YAML bundles.
#[derive(Debug, Parser)]
#[command(name = "okf", version, about, long_about = None)]
pub struct Cli {
    /// Emit NDJSON instead of human text or a bare artifact; schema is always NDJSON.
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
    /// List the direct concept links defined by one concept.
    Links(IdArgs),
    /// Render the link graph (or a bounded rooted neighborhood) as mermaid/dot/graphml.
    Graph(GraphArgs),
    /// Resolve a link/concept-id to a concrete bundle-relative file path.
    Resolve(ResolveArgs),
    /// List, resolve, or retrieve path-valued bundle artifacts without executing them.
    #[command(subcommand)]
    Artifact(ArtifactCmd),
    /// Inspect Attested Computation contracts without executing them.
    #[command(subcommand)]
    Computation(ComputationCmd),

    // ---- CHECK ----
    /// Walk a bundle and report the candidate files that would be analyzed.
    Scan(FailOnArgs),
    /// Inventory every regular source file without parsing it as an OKF concept.
    SourceScan(SourceScanArgs),
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
    /// Diagnose compatibility and safely repair an existing bundle for OKF v0.2.
    Doctor(DoctorArgs),

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

#[derive(Debug, Subcommand)]
pub enum ArtifactCmd {
    /// List local artifacts and concepts under a bundle directory.
    List(ArtifactListArgs),
    /// Resolve any OKF path-valued resource with document context.
    Resolve(ArtifactResolveArgs),
    /// Retrieve a bounded local text artifact; binary files return metadata only.
    Show(ArtifactShowArgs),
}

#[derive(Debug, Subcommand)]
pub enum ComputationCmd {
    /// Check and display a computation contract; never executes code.
    Check(IdArgs),
}

/// A bare bundle positional, shared by commands that take no other argument.
#[derive(Debug, Args)]
pub struct BundleArgs {
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
}

#[derive(Debug, Args)]
pub struct SourceScanArgs {
    /// Source directory to inventory; it need not be an OKF bundle.
    pub directory: String,
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Filter by exact concept `type`.
    #[arg(long = "type")]
    pub type_: Option<String>,
    /// Filter by membership in the concept's `tags`.
    #[arg(long = "tag")]
    pub tag: Option<String>,
    /// Text query (repeatable). Repeated phrases use AND semantics by default.
    #[arg(long = "text")]
    pub text: Vec<String>,
    /// Text matching: phrase (default), all tokens, any token, or literal source text.
    #[arg(long = "match", value_enum, default_value_t = SearchMatchArg::Phrase)]
    pub match_mode: SearchMatchArg,
    /// Text fields to search (comma-separated or repeatable).
    #[arg(
        long = "in",
        value_enum,
        value_delimiter = ',',
        default_value = "id,title,description,body"
    )]
    pub in_: Vec<SearchFieldArg>,
    /// Result order: deterministic relevance (default) or concept id.
    #[arg(long, value_enum, default_value_t = SearchSortArg::Relevance)]
    pub sort: SearchSortArg,
    /// Return at most this many results after all filters and sorting.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Filter by a frontmatter field, `key=value` (repeatable; AND).
    #[arg(long = "field")]
    pub field: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SearchMatchArg {
    Phrase,
    All,
    Any,
    Literal,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SearchFieldArg {
    Id,
    Title,
    Description,
    Body,
    Frontmatter,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SearchSortArg {
    Relevance,
    Id,
}

#[derive(Debug, Args)]
pub struct IdArgs {
    /// Concept id (leading slash optional), e.g. `tables/customers`.
    pub concept: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    /// Concept id (leading slash optional), e.g. `tables/customers`.
    pub concept: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
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
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Bundle-relative directory to browse (default: root `/`).
    #[arg(long, default_value = "/")]
    pub directory: String,
}

#[derive(Debug, Args)]
pub struct GraphArgs {
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Optional concept at the neighborhood root; without one, render the entire graph.
    #[arg(long = "root")]
    pub concept: Option<String>,
    /// Output format: mermaid (default), dot, or graphml.
    #[arg(long, default_value = "mermaid", value_parser = ["mermaid", "dot", "graphml"])]
    pub format: String,
    /// Edges to follow from the root: outgoing (default), incoming, or both.
    #[arg(
        long,
        value_parser = ["outgoing", "incoming", "both"],
        default_value = "outgoing"
    )]
    pub direction: Option<String>,
    /// Maximum neighbor distance from the root (0 = root only; default: unbounded).
    #[arg(long, requires = "concept")]
    pub depth: Option<usize>,
}

#[derive(Debug, Args)]
pub struct ResolveArgs {
    /// Link or concept id to resolve.
    pub link: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Resolve a relative link against this containing concept id.
    #[arg(long)]
    pub from: Option<String>,
}

#[derive(Debug, Args)]
pub struct ArtifactListArgs {
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Bundle-relative directory to inventory.
    #[arg(long, default_value = "references")]
    pub directory: String,
    /// Compute SHA-256 digests (reads each file).
    #[arg(long)]
    pub digest: bool,
}

#[derive(Debug, Args)]
pub struct ArtifactResolveArgs {
    /// Resource path, URL, or scope descriptor.
    pub resource: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Resolve a relative resource against this declaring concept id.
    #[arg(long)]
    pub from: Option<String>,
}

#[derive(Debug, Args)]
pub struct ArtifactShowArgs {
    /// Local artifact path to retrieve.
    pub resource: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Resolve a relative resource against this declaring concept id.
    #[arg(long)]
    pub from: Option<String>,
    /// Retrieve only an inclusive, one-based START:END line range.
    #[arg(long, value_name = "START:END")]
    pub lines: Option<String>,
    /// Maximum bytes read into output.
    #[arg(long, default_value_t = 65_536)]
    pub max_bytes: usize,
    /// Explicitly request remote retrieval (requires a network-enabled build and policy).
    #[arg(long)]
    pub fetch: bool,
}

#[derive(Debug, Args)]
pub struct LintArgs {
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Apply auto-fixable findings (v1: none are auto-fixable — reports what it would do).
    #[arg(long)]
    pub fix: bool,
    /// Severity threshold that makes the run fail (exit 1): never|info|warn|error|any.
    #[arg(
        long = "fail-on",
        value_parser = ["never", "info", "warn", "error", "any"],
        default_value = "error"
    )]
    pub fail_on: Option<String>,
}

#[derive(Debug, Args)]
pub struct FailOnArgs {
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Fail (exit 1) on any result: never (default) | info | warn | error | any.
    #[arg(
        long = "fail-on",
        value_parser = ["never", "info", "warn", "error", "any"],
        default_value = "never"
    )]
    pub fail_on: Option<String>,
}

#[derive(Debug, Args)]
pub struct AffectedArgs {
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
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
    #[arg(
        long = "fail-on",
        value_parser = ["never", "info", "warn", "error", "any"],
        default_value = "never"
    )]
    pub fail_on: Option<String>,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    /// Git ref to diff against (e.g. `HEAD`, a branch, or a commit).
    pub git_ref: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Fail (exit 1) on any change: never (default) | info | warn | error | any.
    #[arg(
        long = "fail-on",
        value_parser = ["never", "info", "warn", "error", "any"],
        default_value = "never"
    )]
    pub fail_on: Option<String>,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Target OKF version.
    #[arg(long, default_value = "0.2", value_parser = ["0.2"])]
    pub target: String,
    /// Enable the allow-listed safe repair set.
    #[arg(long = "fix-safe")]
    pub fix_safe: bool,
    /// Show safe repairs without writing (the default without --yes).
    #[arg(long)]
    pub dry_run: bool,
    /// Confirm applying --fix-safe changes non-interactively.
    #[arg(long, requires = "fix_safe")]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Bundle directory to create (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
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
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
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
    /// Scaffold exact `type: Attested Computation`; requires `--runtime`.
    #[arg(long)]
    pub attested: bool,
    /// Set a custom scalar field at creation, `key=value` (repeatable).
    #[arg(long = "set")]
    pub set: Vec<String>,
    /// Set a declared reference at creation, `key=link` (repeatable).
    #[arg(long = "ref")]
    pub reference: Vec<String>,
    /// Add a standard source, `resource=<path-or-uri>[,kind=<extension>][,id=...,...]`.
    #[arg(long = "add-source")]
    pub add_source: Vec<String>,
    /// Add a full source mapping as JSON/YAML or `@file` (repeatable).
    #[arg(long = "add-source-json")]
    pub add_source_json: Vec<String>,
    /// Runtime for an exact `type: Attested Computation`.
    #[arg(long)]
    pub runtime: Option<String>,
    /// Declared parameter `name:type[:required]` (repeatable).
    #[arg(long = "parameter")]
    pub parameter: Vec<String>,
    /// Path to a computation file; omit to scaffold one inline computation fence.
    #[arg(long, conflicts_with = "inline_computation")]
    pub computation: Option<String>,
    /// Inline sanctioned computation text, literal, `@file`, or `-` for stdin.
    #[arg(long = "inline-computation", conflicts_with = "computation")]
    pub inline_computation: Option<String>,
    /// Executor instructions/code resource.
    #[arg(long = "executor-resource")]
    pub executor_resource: Option<String>,
    /// Required executor receipt field (repeatable).
    #[arg(long = "receipt")]
    pub receipt: Vec<String>,
    /// Deterministic attester code resource.
    #[arg(long = "attester-resource")]
    pub attester_resource: Option<String>,
    /// Actor that generated this content.
    #[arg(long = "generated-by")]
    pub generated_by: Option<String>,
}

#[derive(Debug, Args)]
pub struct EditArgs {
    /// Concept id to edit.
    pub concept: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
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
    /// Add a standard source, `resource=<path-or-uri>[,kind=<extension>][,id=...,...]`.
    #[arg(long = "add-source")]
    pub add_source: Vec<String>,
    /// Add a full source mapping as JSON/YAML or `@file` (repeatable).
    #[arg(long = "add-source-json")]
    pub add_source_json: Vec<String>,
    /// Remove sources matching `<path-or-uri>` or `resource=<path-or-uri>[,kind=<kind>]`
    /// (repeatable).
    #[arg(long = "remove-source")]
    pub remove_source: Vec<String>,
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
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Fail (exit 1) when any source is skipped: never (default) | skipped | any.
    #[arg(
        long = "fail-on",
        value_parser = ["never", "skipped", "any"],
        default_value = "never"
    )]
    pub fail_on: Option<String>,
}

#[derive(Debug, Args)]
pub struct MvArgs {
    /// Existing concept id.
    pub old: String,
    /// New concept id.
    pub new: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
}

#[derive(Debug, Args)]
pub struct RmArgs {
    /// Concept id to remove.
    pub concept: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Remove even if backlinks would dangle.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Concept id to verify.
    pub concept: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// The reviewing actor (e.g. `human:andrey` or `process:ci`).
    #[arg(long = "by")]
    pub by: String,
}

#[derive(Debug, Args)]
pub struct DocsArgs {
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Output format: md|html|pdf|graphml|obsidian|index.
    #[arg(
        long,
        default_value = "md",
        value_parser = ["md", "html", "pdf", "graphml", "obsidian", "index"]
    )]
    pub format: String,
}

#[derive(Debug, Args)]
pub struct OntShowArgs {
    /// Concept type name.
    pub name: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
}

#[derive(Debug, Args)]
pub struct OntRemoveArgs {
    /// Concept type name to remove.
    pub name: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
}

#[derive(Debug, Args)]
pub struct OntEditArgs {
    /// Concept type name.
    pub name: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
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
    /// Mark the exact `Attested Computation` type as standard attested.
    #[arg(long)]
    pub attested: bool,
}
