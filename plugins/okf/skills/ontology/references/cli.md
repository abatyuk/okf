# okf CLI — ontology command reference

> **Generated** by `cargo xtask docs` from `okf schema --json` and curated usage notes (tool 0.2.7, OKF spec 0.2). Do not hand-edit; regenerate instead.

This focused reference contains only commands selected for the `ontology` workflow. Consult it when exact arguments or output shapes are needed. If the installed `okf` version differs from the generated tool version above, or rejects documented syntax, use that command's `--help` output as the runtime authority.

Commands with a human form accept global `--json` for NDJSON; `schema` is always NDJSON. Bundle-aware commands take an optional trailing `bundle` positional resolved as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd. Meta commands have no bundle, and `source-scan` takes an explicit arbitrary directory.

Exit codes: 0 means success under the selected failure threshold, not necessarily no findings; 1 means findings or an unsuccessful resolution; 2 means usage errors; 3 means environment/I/O/YAML errors; 4 means an internal error. Inspect findings even with `--fail-on never`. NDJSON is one record per line, not a JSON array.

## query

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
| `<resource>` | positional | yes | Local artifact path to retrieve |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--from <value>` | string | no | Resolve a relative resource against this declaring concept id |
| `--lines <value>` | string | no | Retrieve only an inclusive, one-based START:END line range |
| `--max-bytes <value>` | int | no | Maximum bytes read into output (default: `65536`) |
| `--fetch` | bool | no | Explicitly request remote retrieval (requires a network-enabled build and policy) (default: `false`) |

Output stream: `artifact-content`.

For document-relative paths, pass the same `--from` used during resolution. JSON includes `text`, `binary`, `truncated`, `size`, `sha256`, and `path`. Binary files provide metadata only. Inspect `truncated` before treating a read as complete; human output alone does not expose this flag. Use bounded line windows and a sufficient byte budget for the needed range; do not infer absence from a truncated result. Use `show` for concepts and this command for opaque or reserved files.

Local artifact reads remain inside the canonical bundle after symlinks. For a URL, `--fetch` requires a network-enabled build and authorization covering that source. A user request to inspect a named source can supply that authorization; no separate OKF policy file is specified by this command. If unavailable, use an authorized external retrieval tool or report the evidence gap. Do not send ambient credentials. Reading computation, executor, or attester code never authorizes execution.

### `okf ontology list`

List the defined concept types.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Output stream: `ontology_type`.

### `okf ontology show`

Show one concept type and its rules.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | positional | yes | Concept type name |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Output stream: `ontology_type`.

### `okf show`

Show one concept's content, heading outline, or selected line range.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--outline` | bool | no | Show only the Markdown heading outline with 1-based document line numbers (default: `false`) |
| `--lines <value>` | string | no | Show only an inclusive 1-based document line range, `START:END` (or one line, `N`) |

Output stream: `concept`.

Without `--json`, show includes the serialized frontmatter and full Markdown body. Plain `show --json` returns metadata only: frontmatter plus `id`, `trust_tier`, `effective_status`, `effective_generated_at`, `latest_verified_at`, and `verification_current`. It does not include the body. `--outline --json` returns `headings` with `line`, `level`, and `text`; `--lines START:END --json` returns `start`, actual `end`, and `lines` containing `line` and `text`. Line numbers refer to the serialized document, including frontmatter. An outline or selected slice does not establish complete document-review coverage.

## check

### `okf lint`

Advisory checks (broken links, missing fields, orphans, ontology violations).

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--fix` | bool | no | Apply auto-fixable findings (v1: none are auto-fixable — reports what it would do) (default: `false`) |
| `--fail-on <value>` | string | no | Severity threshold that makes the run fail (exit 1): never|info|warn|error|any (default: `error`) |

Output stream: `finding`.

### `okf validate`

Conformance validation — the spec's three hard rules only.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Output stream: `violation`.

## mutate

### `okf ontology add` · _mutates_

Define a new concept type with its fields and reference rules.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | positional | yes | Concept type name |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--description <value>` | string | no | Description of the concept type |
| `--field <value>` | list<string> | no | A typed field, `key:type[:required][:v1|v2|...]` (repeatable) |
| `--ref <value>` | list<string> | no | A reference rule, `key:Target[|Target2]:cardinality` (repeatable) |
| `--remove-field <value>` | list<string> | no | Remove a typed field (update only; repeatable) |
| `--remove-ref <value>` | list<string> | no | Remove a reference rule (update only; repeatable) |
| `--attested` | bool | no | Mark the exact `Attested Computation` type as standard attested (default: `false`) |

Output stream: `change`.

Built-in field types: `string`, `text`, `int`, `bool`, `date`, `datetime`, `uri`, `enum`, `list`, `object`; custom type names must resolve in the existing sidecar's `field_types`. Reference cardinalities are `0..1` (optional one), `1..1` (exactly one), `0..n` (optional many), and `1..n` (at least one). For example, `--field "stage:enum:draft|active"` declares choices and `--ref "depends_on:Service:0..n"` permits zero or more Service links. Observed presence alone does not justify a required rule. These flags describe advisory local rules, not portable OKF conformance requirements.

### `okf ontology remove` · _mutates_

Remove a concept type.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | positional | yes | Concept type name to remove |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Output stream: `change`.

### `okf ontology update` · _mutates_

Modify fields/references of an existing concept type.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | positional | yes | Concept type name |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--description <value>` | string | no | Description of the concept type |
| `--field <value>` | list<string> | no | A typed field, `key:type[:required][:v1|v2|...]` (repeatable) |
| `--ref <value>` | list<string> | no | A reference rule, `key:Target[|Target2]:cardinality` (repeatable) |
| `--remove-field <value>` | list<string> | no | Remove a typed field (update only; repeatable) |
| `--remove-ref <value>` | list<string> | no | Remove a reference rule (update only; repeatable) |
| `--attested` | bool | no | Mark the exact `Attested Computation` type as standard attested (default: `false`) |

Output stream: `change`.

Built-in field types: `string`, `text`, `int`, `bool`, `date`, `datetime`, `uri`, `enum`, `list`, `object`; custom type names must resolve in the existing sidecar's `field_types`. Reference cardinalities are `0..1` (optional one), `1..1` (exactly one), `0..n` (optional many), and `1..n` (at least one). For example, `--field "stage:enum:draft|active"` declares choices and `--ref "depends_on:Service:0..n"` permits zero or more Service links. Observed presence alone does not justify a required rule. These flags describe advisory local rules, not portable OKF conformance requirements.
