# okf CLI — repair command reference

> **Generated** by `cargo xtask docs` from `okf schema --json` and curated usage notes (tool 0.3.2, OKF spec 0.2). Do not hand-edit; regenerate instead.

This focused reference contains only commands selected for the `repair` workflow. First pass the skill CLI compatibility preflight. Consult this reference when exact arguments or output shapes are needed. For a compatible CLI that rejects documented syntax, use that command's `--help` output as the runtime authority.

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
  "bundle-backlink": {
    "bundle": "string",
    "id": "string",
    "version": "string"
  },
  "bundle-edge": {
    "evidence": "string",
    "fingerprint_status": "string|null",
    "location": "string",
    "relationship": "{source:string,rule:string,field_path:string,raw_reference:string,target:string,authored_kind:string|null,inverse:string|null,attributes:object,status:string}|null",
    "resource": "string",
    "snapshot": "{requested:object,status:string,evidence:string,resolved:qualified_identity|null,candidate:qualified_identity|null}|null",
    "source": "qualified_identity",
    "status": "string",
    "target": "qualified_identity|null"
  },
  "bundle-node": {
    "bundle": "string",
    "id": "string",
    "version": "string"
  },
  "bundle-registration": {
    "available": "boolean",
    "configured_root": "path",
    "id": "string",
    "overridden": "boolean",
    "root": "path"
  },
  "effective-settings": {
    "bundle": "string",
    "settings": "interpretation-settings"
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
  }
}
```

## meta

### `okf catalog`

List effective catalog registrations, locations, and availability.

_No arguments._

Output stream: `bundle-registration,effective-settings`.

## query

### `okf artifact list`

List local artifacts and concepts under a bundle directory.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle root directory; use --directory to select a directory within the bundle |
| `--directory <value>` | string | no | Bundle-relative directory to inventory (default: `references`) |
| `--digest` | bool | no | Compute SHA-256 digests (reads each file) (default: `false`) |

Output stream: `artifact`.

The positional path selects a bundle root relative to the current directory. To filter within the selected bundle, use `--directory contracts/x`; printed paths remain relative to the bundle root.

### `okf artifact resolve`

Resolve any OKF path-valued resource with document context.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<resource>` | positional | yes | Resource path, URL, or scope descriptor |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--from <value>` | string | no | Resolve a relative resource against this declaring concept id |

Output stream: `artifact-resolution`.

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
| `--fetch` | bool | no | Explicitly request remote retrieval (requires a network-enabled build and policy) (default: `false`) |

Output stream: `artifact-content`.

For document-relative paths, pass the same `--from` used during resolution. JSON includes `text`, `binary`, `truncated`, `size`, `sha256`, and `path`. Binary files provide metadata only. Inspect `truncated` before treating a read as complete; human output alone does not expose this flag. Use bounded line windows and a sufficient byte budget for the needed range; do not infer absence from a truncated result. Use `show` for concepts and this command for opaque or reserved files.

Local artifact reads remain inside the canonical bundle after symlinks. For a URL, `--fetch` requires a network-enabled build and authorization covering that source. A user request to inspect a named source can supply that authorization; no separate OKF policy file is specified by this command. If unavailable, use an authorized external retrieval tool or report the evidence gap. Do not send ambient credentials. Reading computation, executor, or attester code never authorizes execution.

### `okf backlinks`

Concepts that link to a given concept.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--details` | bool | no | Show individual semantic incoming occurrences and configured inverse labels (default: `false`) |

Output stream: `concept,relationship,scope,bundle-backlink,bundle-edge`.

### `okf graph`

Render the link graph (or a bounded rooted neighborhood) as mermaid/dot/graphml.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--root <value>` | string | no | Optional concept at the neighborhood root; without one, render the entire graph |
| `--format <value>` | string | no | Output format: mermaid (default), dot, or graphml (default: `mermaid`) |
| `--direction <value>` | string | no | Edges to follow from the root: outgoing (default), incoming, or both (default: `outgoing`) |
| `--depth <value>` | int | no | Maximum neighbor distance from the root (0 = root only; default: unbounded) |

Output stream: `graph,scope,bundle-node,bundle-edge`.

### `okf links`

List the direct concept links defined by one concept.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--details` | bool | no | Show individual semantic incoming occurrences and configured inverse labels (default: `false`) |

Output stream: `link,relationship,scope,bundle-edge`.

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

Without `--json`, show includes the serialized frontmatter and full Markdown body. Plain `show --json` returns metadata only: frontmatter plus `id`, `trust_tier`, `effective_status`, `effective_generated_at`, `latest_verified_at`, and `verification_current`. It does not include the body. `--outline --json` returns `headings` with `line`, `level`, and `text`; `--lines START:END --json` returns `start`, actual `end`, and `lines` containing `line` and `text`. Line numbers refer to the serialized document, including frontmatter, excluding the three-line display header. `show -n` (or `--numbered`) prints the full document with these numbers and no header; its JSON uses the same line-range record. `show --body` prints only raw Markdown; with `--json` it returns a body record containing `id` and `body`. An outline or selected slice does not establish complete document-review coverage.

## check

### `okf doctor` · _conditionally mutates_

Diagnose compatibility and safely repair an existing bundle for OKF v0.2.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--target <value>` | string | no | Target OKF version (default: `0.2`) |
| `--fix-safe` | bool | no | Enable the allow-listed safe repair set (default: `false`) |
| `--dry-run` | bool | no | Show safe repairs without writing (the default without --yes) (default: `false`) |
| `--yes` | bool | no | Confirm applying --fix-safe changes non-interactively (default: `false`) |

Output stream: `doctor-finding,doctor-summary`.

### `okf lint`

Advisory checks (broken links, missing fields, orphans, ontology violations).

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--fix` | bool | no | Apply auto-fixable findings (v1: none are auto-fixable — reports what it would do) (default: `false`) |
| `--fail-on <value>` | string | no | Severity threshold that makes the run fail (exit 1): never|info|warn|error|any (default: `error`) |

Output stream: `finding,scope`.

For sources with a fingerprint kind, `source-unrecorded` warns when no baseline exists. `source-missing` errors when file, line-range, or markdown-heading sources cannot be fingerprinted, even without a baseline. Lint does not fetch URLs or inspect Git sources. Configure `source_unrecorded = "off"` (or `info`, `warn`, `error`; default `warn`) in `[bundle_settings.default.lint]` in `okf.toml`, or `[bundle_settings."<id>".lint]` for a named bundle. Disabling this rule leaves `source-missing` and `stale` checks active. All lint settings accept `off`, `info`, `warn`, or `error`. Defaults: `broken_link` and `source_missing` are `error`; `missing_title`, `spec_v02` (finding rule `okf-v02`), `ontology_violation`, `source_unrecorded`, and `index_coverage` are `warn`; `missing_description` and `orphan` are `info`. Omitted settings keep these defaults. `off` suppresses the selected lint rule only; validation and stale checks remain independent. `index_exclude` controls index coverage exclusions and `finding_budget` limits ontology findings (positive, default 1,000 per concept). Use `--fail-on warn` to fail on warnings as well as errors.

### `okf validate`

Conformance validation — the spec's three hard rules only.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Output stream: `violation`.

## mutate

### `okf artifact put` · _mutates_

Create or replace a local artifact; citing source fingerprints remain unchanged.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<resource>` | positional | yes | Destination path relative to the bundle root (opaque or reserved artifacts only) |
| `<input>` | positional | yes | Read artifact bytes from @file (file path is relative to the current directory) |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--create-only` | bool | no | Refuse to overwrite an existing file (the default) (default: `false`) |
| `--replace` | bool | no | Allow replacement of an existing artifact, leaving source fingerprints for drift checks (default: `false`) |

Output stream: `artifact-write`.

Write bytes from `@file` to a bundle-relative opaque or reserved artifact. The default (and `--create-only`) refuses an existing destination; `--replace` permits replacement. Parent directories are created as needed. Concept and configuration files are protected, and paths must remain inside the bundle. Recorded source fingerprints are preserved so `stale` detects changes; review citing concepts before `refresh`.

### `okf edit` · _conditionally mutates_

Edit a concept losslessly and invalidate its prior verification.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to edit |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--set <value>` | list<string> | no | Set/update a scalar field, `key=value` (repeatable) |
| `--set-yaml <value>` | list<string> | no | Replace a field with YAML: `key=value`, `key=@file`, or `key=-` (repeatable) |
| `--set-path <value>` | list<string> | no | Set an object path with a YAML value, e.g. `deadline.within=72` (repeatable) |
| `--dry-run` | bool | no | Validate and show the resulting diff without writing (default: `false`) |
| `--unset <value>` | list<string> | no | Remove a field entirely, `key` (repeatable) |
| `--unset-path <value>` | list<string> | no | Delete an object path (repeatable); absent keys are a no-op |
| `--patch <value>` | string | no | Apply an RFC 6902 JSON Patch to frontmatter, supplied as YAML, `@file`, or `-` |
| `--add <value>` | list<string> | no | Append an item to a list field, `key=value` (repeatable, idempotent) |
| `--remove <value>` | list<string> | no | Remove matching item(s) from a list field, `key=value` (repeatable) |
| `--add-source <value>` | list<string> | no | Add a standard source, `resource=<path-or-uri>[,kind=<extension>][,id=...,...]` |
| `--add-source-json <value>` | list<string> | no | Add a full source mapping as JSON/YAML or `@file` (repeatable) |
| `--remove-source <value>` | list<string> | no | Remove sources matching `<path-or-uri>` or `resource=<path-or-uri>[,kind=<kind>]` (repeatable) |
| `--set-body <value>` | string | no | Replace the whole body. Use `@file` to read a file or `-` for stdin |
| `--append-body <value>` | string | no | Append a block to the body. Use `@file` or `-` (stdin) |
| `--replace <OLD> <NEW>` | list<string> | no | Replace body text, `<old> <new>` (repeatable; defaults to exactly one match) |
| `--all` | bool | no | Replace every occurrence matched by `--replace` (default: `false`) |
| `--clear-body` | bool | no | Empty the body (default: `false`) |
| `--set-section <HEADING> <TEXT>` | list<string> | no | Replace a section's content, `<heading> <text>` (repeatable). Text accepts `@file`/`-` |
| `--append-section <HEADING> <TEXT>` | list<string> | no | Append to a section, `<heading> <text>` (repeatable). Text accepts `@file`/`-` |
| `--remove-section <value>` | list<string> | no | Remove a section (heading + content), `<heading>` (repeatable) |
| `--rename-section <OLD> <NEW>` | list<string> | no | Rename a section heading, `<old> <new>` (repeatable) |

Output stream: `change`.

`--set` keeps literal scalar keys; a dotted key is not traversal. `--set-yaml` replaces a complete named value. `--set-path` parses YAML and creates intermediate maps for object-only paths; `--unset-path` removes an object property. `--patch` accepts an RFC 6902 array with concrete JSON Pointer paths, including list edits and `test` guards. Inputs accept inline YAML/JSON, `@file`, or `-`; only one stdin consumer is permitted. Duplicate/non-string keys, tags, anchors, aliases, merge keys, nonfinite numbers and multiple documents fail. `--dry-run` previews without writes. Use `--add-source-json @file` for source mappings. Do not fabricate verification. Meaningful edits remove active `verified` events and update existing `generated.at`; preserve needed historical evidence separately. Body files contain Markdown only, without frontmatter. Section flags take heading and text as separate values. `--replace OLD NEW` replaces exactly one literal body match; `--all` replaces every match and still fails if none exist. `--rename-section OLD NEW` changes a uniquely matched heading without replacing its content. Malformed YAML must be repaired before this command can load it.

### `okf verify` · _mutates_

Append a `verified` entry (the write-side of trust).

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to verify |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--by <value>` | string | yes | The reviewing actor (e.g. `human:andrey` or `process:ci`) |

Output stream: `change`.

## render

### `okf docs` · _conditionally mutates_

Generate documentation from a bundle.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--format <value>` | string | no | Output format: md|html|pdf|graphml|obsidian|index (default: `md`) |

Output stream: `docs,change`.

`--format index` writes indexes throughout the bundle, replacing their bodies; it does not merge curated prose. Use it only when all affected index bodies are generated or replacement is already authorized. Preserve curated indexes and edit only necessary links otherwise. Validate after writes. Other formats emit output rather than updating indexes; the default is `md`.
