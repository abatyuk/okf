//! clap command tree, groups, global `--json`. `okf schema` is derived from this tree.
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};

/// Open Knowledge Format harness — deterministic hands over markdown + YAML bundles.
#[derive(Debug, Parser)]
#[command(name = "okf", version, about, long_about = None)]
pub struct Cli {
    /// Emit NDJSON instead of human text or a bare artifact; schema is always NDJSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Select an explicitly registered bundle identity (paths keep separate meanings).
    #[arg(long, global = true)]
    pub bundle_id: Option<String>,
    /// Add a registered bundle to the examination scope (repeatable).
    #[arg(long = "scope-bundle", global = true)]
    pub scope_bundle: Vec<String>,
    /// Examine every locally available registered bundle.
    #[arg(long, global = true, conflicts_with = "scope_bundle")]
    pub catalog_scope: bool,
    /// Inspect a locally available Git revision; never fetches missing objects.
    #[arg(long, global = true)]
    pub revision: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    // ---- META ----
    /// Print machine-readable CLI metadata (all commands, args, output shapes) as NDJSON.
    Schema,
    /// List effective catalog registrations, locations, and availability.
    Catalog,
    /// Print the CLI version and the OKF spec version(s) it supports.
    Version,

    // ---- QUERY ----
    /// List all concepts (search with no filter).
    List(SearchArgs),
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
    /// List, resolve, retrieve, or write path-valued bundle artifacts without executing them.
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

    /// Preview, apply, or recover coordinated single-bundle changes.
    #[command(subcommand)]
    Changeset(ChangeSetCmd),

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
    /// Define a new concept type with structured fields, references, and relationships.
    Add(OntEditArgs),
    /// Modify an existing concept type and its named declarations.
    Update(OntEditArgs),
    /// Remove a concept type.
    Remove(OntRemoveArgs),
    /// Add, replace, or remove reusable field-type definitions.
    #[command(subcommand)]
    FieldType(OntFieldTypeCmd),
    /// Apply coordinated concept and reusable field-type changes atomically.
    Apply(OntApplyArgs),
}

#[derive(Debug, Subcommand)]
pub enum OntFieldTypeCmd {
    /// List reusable field-type definitions.
    List(BundleArgs),
    /// Show a reusable field-type definition and its effective fields.
    Show(OntShowArgs),
    /// Define a new reusable field type.
    Add(OntFieldTypeEditArgs),
    /// Replace an existing reusable field type completely.
    Update(OntFieldTypeEditArgs),
    /// Remove a reusable field type if the resulting ontology remains valid.
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
    /// Create or replace a local artifact; citing source fingerprints remain unchanged.
    Put(ArtifactPutArgs),
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
    /// Typed nested selector condition (repeatable; AND).
    #[arg(long)]
    pub facet_filter: Vec<String>,
    /// Emit JSON facets over all matches, before pagination.
    #[arg(long)]
    pub facets: bool,
    /// Facet selector (repeatable); does not bypass high cardinality guard.
    #[arg(long)]
    pub facet: Vec<String>,
    /// Start page at this nonnegative offset; requires a positive limit.
    #[arg(long, requires = "limit")]
    pub offset: Option<usize>,
    /// Maximum examined documents across scope (positive).
    #[arg(long, conflicts_with = "full_scan")]
    pub scan_limit: Option<usize>,
    /// Examine the entire selected scope without a document scan budget.
    #[arg(long, conflicts_with = "scan_limit")]
    pub full_scan: bool,
    /// Explicit metadata projection selector (repeatable).
    #[arg(long)]
    pub project: Vec<String>,
    /// Human output column selector (repeatable).
    #[arg(long = "columns", alias = "column", value_delimiter = ',')]
    pub column: Vec<String>,
    /// Named bundle display view.
    #[arg(long)]
    pub view: Option<String>,
    /// Expand selected outbound relationship rule (repeatable).
    #[arg(long, conflicts_with = "no_expand")]
    pub expand: Vec<String>,
    /// Disable configured human view expansion.
    #[arg(long)]
    pub no_expand: bool,
    /// Metadata selector for related targets (repeatable).
    #[arg(long)]
    pub target_field: Vec<String>,
    /// Maximum edge occurrences per primary hit (1..100).
    #[arg(long, default_value_t = 10)]
    pub expansion_edges: usize,
    /// Maximum distinct target payloads per query (1..1000).
    #[arg(long, default_value_t = 100)]
    pub expansion_targets: usize,
    /// Maximum serialized expansion bytes (1..1048576).
    #[arg(long, default_value_t = 262144)]
    pub expansion_bytes: usize,
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
    /// Show individual semantic incoming occurrences and configured inverse labels.
    #[arg(long)]
    pub details: bool,
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
    /// Print document line numbers (including frontmatter); excludes the display header.
    #[arg(short = 'n', long, conflicts_with_all = ["outline", "body"])]
    pub numbered: bool,
    /// Print only the raw Markdown body, without frontmatter or display headers.
    #[arg(long, conflicts_with_all = ["outline", "lines", "numbered"])]
    pub body: bool,
    /// Explicit metadata projection selector (repeatable).
    #[arg(long, conflicts_with_all = ["body", "outline", "lines", "numbered"])]
    pub project: Vec<String>,
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
    /// Bundle root directory; use --directory to select a directory within the bundle.
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
    /// Artifact path relative to the bundle root, or to --from when provided.
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
    /// Request remote retrieval; unavailable unless built with url-sources and allowed by policy.
    #[arg(long)]
    pub fetch: bool,
}

#[derive(Debug, Args)]
pub struct ArtifactPutArgs {
    /// Destination path relative to the bundle root (opaque or reserved artifacts only).
    pub resource: String,
    /// Read artifact bytes from @file (file path is relative to the current directory).
    #[arg(value_name = "@FILE")]
    pub input: String,
    /// Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory).
    pub bundle: Option<String>,
    /// Refuse to overwrite an existing file (the default).
    #[arg(long, conflicts_with = "replace")]
    pub create_only: bool,
    /// Allow replacement of an existing artifact, leaving source fingerprints for drift checks.
    #[arg(long, conflicts_with = "create_only")]
    pub replace: bool,
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
    /// Fail on any result: never (default) or any; info/warn/error are compatibility aliases.
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
    /// Cap hops with --transitive; otherwise ignored with a diagnostic.
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
    /// Replace the generated Markdown body. Use `@file` or `-` for stdin.
    #[arg(long)]
    pub body: Option<String>,
    /// Scaffold exact `type: Attested Computation`; requires `--runtime`.
    #[arg(long)]
    pub attested: bool,
    /// Set a custom scalar field at creation, `key=value` (repeatable).
    #[arg(long = "set")]
    pub set: Vec<String>,
    /// Replace a field with YAML: `key=value`, `key=@file`, or `key=-` (repeatable).
    #[arg(long = "set-yaml")]
    pub set_yaml: Vec<String>,
    /// Set an object path with a YAML value, e.g. `deadline.within=72` (repeatable).
    #[arg(long = "set-path")]
    pub set_path: Vec<String>,
    /// Validate and show the resulting diff without writing.
    #[arg(long)]
    pub dry_run: bool,
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
    /// Replace a field with YAML: `key=value`, `key=@file`, or `key=-` (repeatable).
    #[arg(long = "set-yaml")]
    pub set_yaml: Vec<String>,
    /// Set an object path with a YAML value, e.g. `deadline.within=72` (repeatable).
    #[arg(long = "set-path")]
    pub set_path: Vec<String>,
    /// Validate and show the resulting diff without writing.
    #[arg(long)]
    pub dry_run: bool,
    /// Remove a field entirely, `key` (repeatable).
    #[arg(long = "unset")]
    pub unset: Vec<String>,
    /// Delete an object path (repeatable); absent keys are a no-op.
    #[arg(long = "unset-path")]
    pub unset_path: Vec<String>,
    /// Apply an RFC 6902 JSON Patch to frontmatter, supplied as YAML, `@file`, or `-`.
    #[arg(long, allow_hyphen_values = true)]
    pub patch: Option<String>,
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
    /// Replace body text, `<old> <new>` (repeatable; defaults to exactly one match).
    #[arg(long = "replace", num_args = 2, value_names = ["OLD", "NEW"])]
    pub replace: Vec<String>,
    /// Replace every occurrence matched by `--replace`.
    #[arg(long, requires = "replace")]
    pub all: bool,
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
    /// Rename a section heading, `<old> <new>` (repeatable).
    #[arg(long = "rename-section", num_args = 2, value_names = ["OLD", "NEW"])]
    pub rename_section: Vec<String>,
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
    /// Output format: md|html|graphml|obsidian|index; pdf is retained but unavailable.
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
    /// Validate and preview the change without writing.
    #[arg(long)]
    pub dry_run: bool,
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
    /// A complete field declaration, key=YAML|@file|- (repeatable).
    #[arg(long)]
    pub field_yaml: Vec<String>,
    /// A complete reference declaration, key=YAML|@file|- (repeatable).
    #[arg(long)]
    pub ref_yaml: Vec<String>,
    /// A complete relationship declaration, key=YAML|@file|- (repeatable).
    #[arg(long)]
    pub relationship_yaml: Vec<String>,
    /// Remove a relationship (update only; repeatable).
    #[arg(long)]
    pub remove_relationship: Vec<String>,
    /// Merge a concept-type definition from inline YAML, @file, a file path, or stdin (-).
    #[arg(long, allow_hyphen_values = true)]
    pub from: Option<String>,
    /// Validate and preview the change without writing.
    #[arg(long)]
    pub dry_run: bool,
    /// Mark the exact `Attested Computation` type as standard attested.
    #[arg(long)]
    pub attested: bool,
}

#[derive(Debug, Args)]
pub struct OntFieldTypeEditArgs {
    /// Reusable field-type name.
    pub name: String,
    /// Bundle directory.
    pub bundle: Option<String>,
    /// Complete field-type definition: inline YAML, @file, a file path, or stdin (-).
    #[arg(long, allow_hyphen_values = true)]
    pub from: String,
    /// Validate and preview the change without writing.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct OntApplyArgs {
    /// Bundle directory.
    pub bundle: Option<String>,
    /// Coordinated change document: inline YAML, @file, a file path, or stdin (-).
    #[arg(long, allow_hyphen_values = true)]
    pub from: String,
    /// Validate and preview the change without writing.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Subcommand)]
pub enum ChangeSetCmd {
    /// Validate and preview a coordinated change document without writing bundle files.
    Plan(ChangeSetArgs),
    /// Validate then publish coordinated changes with a recoverable rollback journal.
    Apply(ChangeSetArgs),
    /// Recover an interrupted publication without overwriting intervening edits.
    Recover(BundleArgs),
}

#[derive(Debug, Args)]
pub struct ChangeSetArgs {
    /// Bundle directory.
    pub bundle: Option<String>,
    /// Version 1 change document: inline YAML/JSON, @file, or stdin (-).
    #[arg(long, allow_hyphen_values = true)]
    pub from: String,
    /// Require this base digest from a previous plan before applying.
    #[arg(long)]
    pub expect: Option<String>,
    /// Validate and preview without writing (also the behavior of changeset plan).
    #[arg(long)]
    pub dry_run: bool,
}

/// The same capability table drives runtime admission, help and machine discovery.
pub fn supports_scope(name: &str) -> bool {
    matches!(
        name,
        "graph" | "backlinks" | "links" | "resolve" | "affected" | "lint" | "search" | "list"
    )
}
pub fn supports_revision(name: &str) -> bool {
    supports_scope(name) && name != "lint"
}
pub fn supports_bundle(name: &str) -> bool {
    !matches!(name, "schema" | "catalog" | "version" | "source-scan")
}

pub fn command_tree() -> clap::Command {
    let mut command = Cli::command();
    let globals = command
        .get_arguments()
        .filter(|a| a.is_global_set())
        .cloned()
        .collect::<Vec<_>>();
    fn decorate(command: &mut clap::Command, prefix: &str, globals: &[clap::Arg]) {
        let name = if prefix.is_empty() {
            command.get_name().to_string()
        } else {
            format!("{prefix} {}", command.get_name())
        };
        if command.has_subcommands() {
            for child in command.get_subcommands_mut() {
                decorate(child, &name, globals);
            }
            return;
        }
        for (arg, supported) in [
            ("revision", supports_revision(&name)),
            ("scope_bundle", supports_scope(&name)),
            ("catalog_scope", supports_scope(&name)),
            ("bundle_id", supports_bundle(&name)),
        ] {
            if !supported {
                if let Some(global) = globals.iter().find(|a| a.get_id() == arg) {
                    *command = command.clone().arg(global.clone().hide(true));
                }
            }
        }
        if name == "ontology field-type remove" {
            *command = command.clone().mut_args(|a| {
                if a.get_id() == "name" {
                    a.help("Reusable field-type name to remove")
                } else {
                    a
                }
            });
        }
        if matches!(name.as_str(), "links" | "computation check") {
            *command = command.clone().mut_arg("details", |a| {
                a.help("Compatibility option; detailed output is not implemented for this command")
            });
        }
    }
    for child in command.get_subcommands_mut() {
        decorate(child, "", &globals);
    }
    command.build();
    command
}
