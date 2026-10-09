# okf CLI — review-verify command reference

> **Generated** by `cargo xtask docs` from `okf schema --json` and curated usage notes (tool 0.4.0, OKF spec 0.2). Do not hand-edit; regenerate instead.

This focused reference contains only commands selected for the `review-verify` workflow. First pass the skill CLI compatibility preflight. Consult this reference when exact arguments or output shapes are needed. For a compatible CLI that rejects documented syntax, use that command's `--help` output as the runtime authority.

Commands with a human form accept global `--json` for NDJSON; `schema` is always NDJSON. Bundle-aware commands take an optional trailing `bundle` path or a separate `--bundle-id` selector. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Catalog registration alone does not extend examination scope. Meta commands have no bundle, and `source-scan` takes an explicit arbitrary directory.

Exit codes: 0 means success under the selected failure threshold, not necessarily no findings; 1 means findings or an unsuccessful resolution; 2 means usage errors; 3 means environment/I/O/YAML errors; 4 means an internal error. Inspect findings even with `--fail-on never`. NDJSON is one record per line, not a JSON array.

## Global arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `--json` | bool | no | Emit NDJSON instead of human text or a bare artifact; schema is always NDJSON (default: `false`) |
| `--bundle-id <value>` | string | no | Select an explicitly registered bundle identity (paths keep separate meanings) |
| `--scope-bundle <value>` | list<string> | no | Add a registered bundle to the examination scope (repeatable) |
| `--catalog-scope` | bool | no | Examine every locally available registered bundle (default: `false`) |
| `--revision <value>` | string | no | Inspect a locally available Git revision; never fetches missing objects |

Scope defaults to the selected bundle. Use scope options only for graph, links, backlinks, resolve, affected, lint, search, and list; other commands reject additional scope. `--revision` supports graph, links, backlinks, resolve, affected, search, and list. Scope and effective settings are interpretation context, not authored concept metadata.

## Extension record contracts

Authored concept metadata remains open. These computed records describe scope, identity, interpretation, and explicit query extensions. Missing and null remain distinct.

```json
{
  "bundle-registration": {
    "available": "boolean",
    "configured_root": "path",
    "id": "string",
    "overridden": "boolean",
    "root": "path"
  },
  "concept-identity": {
    "bundle": "string",
    "id": "string",
    "schema_version": 1,
    "version": "string"
  },
  "effective-settings": {
    "bundle": "string",
    "settings": "interpretation-settings"
  },
  "expansion-summary": {
    "bounds": "{edges_per_hit:integer,targets:integer,bytes:integer}",
    "emitted_bytes": "integer",
    "emitted_edges": "integer",
    "emitted_targets": "integer",
    "reasons": "array<string>",
    "schema_version": 1,
    "total_edges": "integer|null",
    "truncated": "boolean"
  },
  "facet": {
    "basis": "all-matches|observed-matches",
    "complete": "boolean",
    "diagnostics": "array<object>",
    "field": "string",
    "omitted_values": "integer",
    "schema_version": 1,
    "truncated": "boolean",
    "values": "array<{value:scalar,count:integer}>"
  },
  "facet-excluded": {
    "complete": "boolean",
    "distinct_values": "integer",
    "field": "string",
    "reason": "string",
    "schema_version": 1,
    "threshold": "integer"
  },
  "projection": {
    "bundle": "string|null",
    "fields": "array<{field:string,present:boolean,value?:any,occurrences?:array<{path:string,value:any}>,empty_lists?:integer}>",
    "id": "string",
    "schema_version": 1,
    "version": "string"
  },
  "qualified_identity": {
    "bundle": "string",
    "id": "string",
    "version": "string"
  },
  "query-summary": {
    "examined_documents": "integer",
    "filters": "array<string>",
    "has_more": "boolean",
    "interpretation": "array<interpretation-settings>",
    "limit": "integer|null",
    "next_offset": "integer|null",
    "observed_matches": "integer",
    "offset": "integer",
    "partial": "boolean",
    "returned": "integer",
    "scan_complete": "boolean",
    "scan_limit": "integer|null",
    "schema_version": 1,
    "scope": "array<{bundle:string,root:path,version:string,mutable:boolean}>",
    "total_matches": "integer|null"
  },
  "related-concept": {
    "bundle": "string|null",
    "fields": "projection.fields",
    "id": "string",
    "identity": "string",
    "schema_version": 1,
    "version": "string"
  },
  "relationship": {
    "bundle": "string",
    "edge": "object",
    "primary": "string",
    "schema_version": 1,
    "target_identity": "string|null",
    "target_payload_status": "string (optional)",
    "version": "string"
  },
  "scope": {
    "examined": "array<{id:string,root:path,version:string,interpretation:interpretation-settings}>",
    "requested": "array<string>",
    "snapshot_examined": "array<qualified_identity> (optional)",
    "unavailable": "array<string>"
  },
  "warning": {
    "limit": "integer|null",
    "message": "string",
    "omitted": "integer|null",
    "reason": "string",
    "schema_version": 1
  }
}
```

## meta

### `okf catalog`

List effective catalog registrations, locations, and availability.

_No arguments._

Output stream: `bundle-registration,effective-settings`.

Supported global options: `--json`.

## query

### `okf artifact resolve`

Resolve any OKF path-valued resource with document context.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<resource>` | positional | yes | Resource path, URL, or scope descriptor |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--from <value>` | string | no | Resolve a relative resource against this declaring concept id |

Output stream: `artifact-resolution`.

Supported global options: `--bundle-id`, `--json`.

**Path namespaces:** a leading `/` means bundle-root-relative, not an operating-system absolute path. Other local paths resolve against the declaring concept's directory when `--from` is supplied, otherwise the bundle root. Keep `--from` on the subsequent read too. JSON uses `artifact_kind` (concept, artifact, reserved, external, scope, missing, or blocked), `path`, `exists`, `size`, and `message`. Missing/blocked resolution exits 1. A scope descriptor is provenance, not a missing file.

**Repository sources:** with repo `/work/app`, bundle `/work/app/knowledge`, and declaring concept `notes/service`, `/references/spec.txt` resolves to `/work/app/knowledge/references/spec.txt`. A `kind=git-path` or `git-commit` source `src/service.rs` is instead fingerprinted from the Git worktree root as `/work/app/src/service.rs`; the artifact resolver does not reinterpret paths by source kind. File/line-range/markdown-heading fingerprints use bundle-relative paths. Record the actual source location and convention; do not assume fingerprint and artifact paths coincide. Inspect external repository evidence with normal source tools within existing read authorization. Do not rewrite provenance, bypass containment, or mirror files merely to make artifact resolution succeed.

### `okf artifact show`

Retrieve a bounded local text artifact; binary files return metadata only.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<resource>` | positional | yes | Artifact path relative to the bundle root, or to --from when provided |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--from <value>` | string | no | Resolve a relative resource against this declaring concept id |
| `--lines <value>` | string | no | Retrieve only an inclusive, one-based START:END line range |
| `--max-bytes <value>` | int | no | Maximum bytes read into output (default: `65536`) |
| `--fetch` | bool | no | Request remote retrieval; unavailable unless built with url-sources and allowed by policy (default: `false`) |

Output stream: `artifact-content`.

Supported global options: `--bundle-id`, `--json`.

For document-relative paths, pass the same `--from` used during resolution. JSON includes `text`, `binary`, `truncated`, `size`, `sha256`, and `path`. Binary files provide metadata only. Inspect `truncated` before treating a read as complete; human output alone does not expose this flag. Use bounded line windows and a sufficient byte budget for the needed range; do not infer absence from a truncated result. Use `show` for concepts and this command for opaque or reserved files.

Local artifact reads remain inside the canonical bundle after symlinks. For a URL, `--fetch` requires a network-enabled build and authorization covering that source. A user request to inspect a named source can supply that authorization; no separate OKF policy file is specified by this command. If unavailable, use an authorized external retrieval tool or report the evidence gap. Do not send ambient credentials. Reading computation, executor, or attester code never authorizes execution.

### `okf list`

List all concepts (search with no filter).

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--type <value>` | string | no | Filter by exact concept `type` |
| `--tag <value>` | string | no | Filter by membership in the concept's `tags` |
| `--text <value>` | list<string> | no | Text query (repeatable). Repeated phrases use AND semantics by default |
| `--match <value>` | string | no | Text matching: phrase (default), all tokens, any token, or literal source text (default: `phrase`) |
| `--in <value>` | list<string> | no | Text fields to search (comma-separated or repeatable) (default: `id,title,description,body`) |
| `--sort <value>` | string | no | Result order: deterministic relevance (default) or concept id (default: `relevance`) |
| `--limit <value>` | int | no | Return at most this many results after all filters and sorting |
| `--field <value>` | list<string> | no | Filter by a frontmatter field, `key=value` (repeatable; AND) |
| `--facet-filter <value>` | list<string> | no | Typed nested selector condition (repeatable; AND) |
| `--facets` | bool | no | Emit JSON facets over all matches, before pagination (default: `false`) |
| `--facet <value>` | list<string> | no | Facet selector (repeatable); does not bypass high cardinality guard |
| `--offset <value>` | int | no | Start page at this nonnegative offset; requires a positive limit |
| `--scan-limit <value>` | int | no | Maximum examined documents across scope (positive) |
| `--full-scan` | bool | no | Examine the entire selected scope without a document scan budget (default: `false`) |
| `--project <value>` | list<string> | no | Explicit metadata projection selector (repeatable) |
| `--columns <value>` | list<string> | no | Human output column selector (repeatable) |
| `--view <value>` | string | no | Named bundle display view |
| `--expand <value>` | list<string> | no | Expand selected outbound relationship rule (repeatable) |
| `--no-expand` | bool | no | Disable configured human view expansion (default: `false`) |
| `--target-field <value>` | list<string> | no | Metadata selector for related targets (repeatable) |
| `--expansion-edges <value>` | int | no | Maximum edge occurrences per primary hit (1..100) (default: `10`) |
| `--expansion-targets <value>` | int | no | Maximum distinct target payloads per query (1..1000) (default: `100`) |
| `--expansion-bytes <value>` | int | no | Maximum serialized expansion bytes (1..1048576) (default: `262144`) |

Output stream: `concept,concept-identity,scope,projection,query-summary,facet,facet-excluded,relationship,related-concept,expansion-summary,warning`.

Supported global options: `--bundle-id`, `--catalog-scope`, `--json`, `--revision`, `--scope-bundle`.

JSON records contain frontmatter and computed lifecycle/trust metadata, not bodies. Aggregate field occurrence counts only after checking scan completeness and output warnings. The default scan budget is 1,000 eligible documents across scope; --limit bounds output, not scan work. For exhaustive inventory use a sufficient --scan-limit or deliberate --full-scan, and aggregate metadata locally rather than loading all records into context. Each offset invocation rescans; an incomplete scan has no authoritative continuation.

### `okf show`

Show one concept's content, heading outline, or selected line range.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--outline` | bool | no | Show only the Markdown heading outline with 1-based document line numbers (default: `false`) |
| `--lines <value>` | string | no | Show only an inclusive 1-based document line range, `START:END` (or one line, `N`) |
| `--numbered` | bool | no | Print document line numbers (including frontmatter); excludes the display header (default: `false`) |
| `--body` | bool | no | Print only the raw Markdown body, without frontmatter or display headers (default: `false`) |
| `--project <value>` | list<string> | no | Explicit metadata projection selector (repeatable) |

Output stream: `concept,projection`.

Supported global options: `--bundle-id`, `--json`.

Without `--json`, show includes the serialized frontmatter and full Markdown body. Plain `show --json` returns metadata only: frontmatter plus `id`, `trust_tier`, `effective_status`, `effective_generated_at`, `latest_verified_at`, and `verification_current`. It does not include the body. `--outline --json` returns `headings` with `line`, `level`, and `text`; `--lines START:END --json` returns `start`, actual `end`, and `lines` containing `line` and `text`. Line numbers refer to the serialized document, including frontmatter, excluding the three-line display header. `show -n` (or `--numbered`) prints the full document with these numbers and no header; its JSON uses the same line-range record. `show --body` prints only raw Markdown; with `--json` it returns a body record containing `id` and `body`. An outline or selected slice does not establish complete document-review coverage.

## check

### `okf computation check`

Check and display a computation contract; never executes code.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--details` | bool | no | Compatibility option; detailed output is not implemented for this command (default: `false`) |

Output stream: `computation-contract`.

Supported global options: `--bundle-id`, `--json`.

Inspect-only: `execution: not-run` is not a passing runtime attestation. Document verification and inspection of executable resources never establish a run verdict.

### `okf stale`

Drift detection: recorded vs. recomputed source fingerprints (+ `stale_after`).

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--fail-on <value>` | string | no | Fail on any result: never (default) or any; info/warn/error are compatibility aliases (default: `never`) |

Output stream: `drift`.

Supported global options: `--bundle-id`, `--json`.

Sources with fingerprint kinds but no recorded fingerprint are reported as `unrecorded`; unreadable sources are reported even without a baseline. Standard sources without a fingerprint kind are provenance only. Use `--fail-on any` for a failing exit status when findings exist. `validate` checks conformance, not source health.

### `okf stats`

Bundle summary: counts by type, trust distribution, orphans.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--fail-on <value>` | string | no | Fail on any result: never (default) or any; info/warn/error are compatibility aliases (default: `never`) |

Output stream: `stats`.

Supported global options: `--bundle-id`, `--json`.

## mutate

### `okf verify` · _mutates_

Append a `verified` entry (the write-side of trust).

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to verify |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--by <value>` | string | yes | The reviewing actor (e.g. `human:andrey` or `process:ci`) |

Output stream: `change`.

Supported global options: `--bundle-id`, `--json`.
