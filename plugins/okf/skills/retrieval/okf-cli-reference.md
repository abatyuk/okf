# okf CLI — argument reference

> **Generated** by `cargo xtask docs` from `okf schema --json` (tool 0.2.2, OKF spec 0.2). Do not hand-edit; regenerate instead.

**For skills:** consult this file to learn a command's arguments. **Do not** run `okf <cmd> --help` or `okf schema` first just to discover flags — they are all listed here. Every command also accepts the global `--json` flag (NDJSON output) and takes an optional trailing `bundle` positional that falls back to `$OKF_BUNDLE`, then the cwd.

## meta

### `okf schema`

Print machine-readable CLI metadata (all commands, args, output shapes) as NDJSON.

_No arguments._

Output stream: `schema`.

### `okf version`

Print the CLI version and the OKF spec version(s) it supports.

_No arguments._

Output stream: `version`.

## query

### `okf artifact list`

List local artifacts and concepts under a bundle directory.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--directory <value>` | string | no | Bundle-relative directory to inventory (default: `references`) |
| `--digest` | bool | no | Compute SHA-256 digests (reads each file) |

Output stream: `artifact`.

### `okf artifact resolve`

Resolve any OKF path-valued resource with document context.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<resource>` | positional | yes | Resource path, URL, or scope descriptor |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--from <value>` | string | no | Resolve a relative resource against this declaring concept id |

Output stream: `artifact-resolution`.

### `okf artifact show`

Retrieve a bounded local text artifact; binary files return metadata only.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<resource>` | positional | yes | Local artifact path to retrieve |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--from <value>` | string | no | Resolve a relative resource against this declaring concept id |
| `--lines <value>` | string | no | Retrieve only an inclusive, one-based START:END line range |
| `--max-bytes <value>` | string | no | Maximum bytes read into output (default: `65536`) |
| `--fetch` | bool | no | Explicitly request remote retrieval (requires a network-enabled build and policy) |

Output stream: `artifact-content`.

### `okf backlinks`

Concepts that link to a given concept.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Output stream: `concept`.

### `okf browse`

Show a directory's index.md, synthesizing it when absent.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--directory <value>` | string | no | Bundle-relative directory to browse (default: root `/`) (default: `/`) |

Output stream: `index`.

### `okf graph`

Render the link graph (or a bounded rooted neighborhood) as mermaid/dot/graphml.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `<concept>` | positional | no | Optional concept at the neighborhood root; without one, render the entire graph |
| `--format <value>` | string | no | Output format: mermaid (default), dot, or graphml (default: `mermaid`) |
| `--direction <value>` | string | no | Edges to follow from the root: outgoing (default), incoming, or both |
| `--depth <value>` | string | no | Maximum neighbor distance from the root (0 = root only; default: unbounded) |

Output stream: `graph`.

### `okf links`

List the direct concept links defined by one concept.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Output stream: `link`.

### `okf list`

List all concepts (search with no filter).

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Output stream: `concept`.

### `okf ontology list`

List the defined concept types.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Output stream: `ontology_type`.

### `okf ontology show`

Show one concept type and its rules.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | positional | yes | Concept type name |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Output stream: `ontology_type`.

### `okf resolve`

Resolve a link/concept-id to a concrete bundle-relative file path.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<link>` | positional | yes | Link or concept id to resolve |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--from <value>` | string | no | Resolve a relative link against this containing concept id |

Output stream: `resolved`.

### `okf search`

Search concepts by type, tag, text, and/or frontmatter field.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--type <value>` | string | no | Filter by exact concept `type` |
| `--tag <value>` | string | no | Filter by membership in the concept's `tags` |
| `--text <value>` | string | no | Filter by case-insensitive substring across id/title/description/body |
| `--field <value>` | list<string> | no | Filter by a frontmatter field, `key=value` (repeatable; AND) |

Output stream: `concept`.

### `okf show`

Show one concept's content, heading outline, or selected line range.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--outline` | bool | no | Show only the Markdown heading outline with 1-based document line numbers |
| `--lines <value>` | string | no | Show only an inclusive 1-based document line range, `START:END` (or one line, `N`) |

Output stream: `concept`.

## check

### `okf affected`

Impact query: concepts needing review given a set of changed links.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--changed <value>` | list<string> | no | A changed link/concept-id/resource (repeatable; also read from stdin lines) |
| `--transitive` | bool | no | Follow the cascade past direct dependents |
| `--depth <value>` | string | no | Cap the number of hops when `--transitive` |
| `--fail-on <value>` | string | no | Fail (exit 1) on any affected concept: never (default) | info | warn | error | any |

Output stream: `affected`.

### `okf computation check`

Check and display a computation contract; never executes code.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Output stream: `computation-contract`.

### `okf diff`

Concept-level diff of the working tree vs a git ref.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<git_ref>` | positional | yes | Git ref to diff against (e.g. `HEAD`, a branch, or a commit) |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--fail-on <value>` | string | no | Fail (exit 1) on any change: never (default) | info | warn | error | any |

Output stream: `diff`.

### `okf doctor` · _mutates_

Diagnose compatibility and safely repair an existing bundle for OKF v0.2.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--target <value>` | string | no | Target OKF version (default: `0.2`) |
| `--fix-safe` | bool | no | Enable the allow-listed safe repair set |
| `--dry-run` | bool | no | Show safe repairs without writing (the default without --yes) |
| `--yes` | bool | no | Confirm applying --fix-safe changes non-interactively |

Output stream: `doctor-finding,doctor-summary`.

### `okf lint`

Advisory checks (broken links, missing fields, orphans, ontology violations).

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--fix` | bool | no | Apply auto-fixable findings (v1: none are auto-fixable — reports what it would do) |
| `--fail-on <value>` | string | no | Severity threshold that makes the run fail (exit 1): never|info|warn|error|any |

Output stream: `finding`.

### `okf scan`

Walk a bundle and report the candidate files that would be analyzed.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Output stream: `scan`.

### `okf source-scan`

Inventory every regular source file without parsing it as an OKF concept.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<directory>` | positional | yes | Source directory to inventory; it need not be an OKF bundle |

Output stream: `source-file`.

### `okf stale`

Drift detection: recorded vs. recomputed source fingerprints (+ `stale_after`).

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--fail-on <value>` | string | no | Fail (exit 1) on any result: never (default) | info | warn | error | any |

Output stream: `drift`.

### `okf stats`

Bundle summary: counts by type, trust distribution, orphans.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--fail-on <value>` | string | no | Fail (exit 1) on any result: never (default) | info | warn | error | any |

Output stream: `stats`.

### `okf validate`

Conformance validation — the spec's three hard rules only.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Output stream: `violation`.

## mutate

### `okf add` · _mutates_

Add a new concept document, scaffolded from the ontology.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<path>` | positional | yes | Bundle-relative path of the new concept (with or without `.md`) |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--type <value>` | string | no | Concept `type` (an ontology concept-type key). Optional with `--attested` |
| `--title <value>` | string | no | Concept title |
| `--description <value>` | string | no | Concept description |
| `--attested` | bool | no | Scaffold exact `type: Attested Computation`; requires `--runtime` |
| `--set <value>` | list<string> | no | Set a custom scalar field at creation, `key=value` (repeatable) |
| `--ref <value>` | list<string> | no | Set a declared reference at creation, `key=link` (repeatable) |
| `--add-source <value>` | list<string> | no | Add a standard source, `resource=<path-or-uri>[,kind=<extension>][,id=...,...]` |
| `--add-source-json <value>` | list<string> | no | Add a full source mapping as JSON/YAML or `@file` (repeatable) |
| `--runtime <value>` | string | no | Runtime for an exact `type: Attested Computation` |
| `--parameter <value>` | list<string> | no | Declared parameter `name:type[:required]` (repeatable) |
| `--computation <value>` | string | no | Path to a computation file; omit to scaffold one inline computation fence |
| `--inline-computation <value>` | string | no | Inline sanctioned computation text, literal, `@file`, or `-` for stdin |
| `--executor-resource <value>` | string | no | Executor instructions/code resource |
| `--receipt <value>` | list<string> | no | Required executor receipt field (repeatable) |
| `--attester-resource <value>` | string | no | Deterministic attester code resource |
| `--generated-by <value>` | string | no | Actor that generated this content |

Output stream: `change`.

### `okf edit` · _mutates_

Edit a concept losslessly and invalidate its prior verification.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to edit |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--set <value>` | list<string> | no | Set/update a scalar field, `key=value` (repeatable) |
| `--unset <value>` | list<string> | no | Remove a field entirely, `key` (repeatable) |
| `--add <value>` | list<string> | no | Append an item to a list field, `key=value` (repeatable, idempotent) |
| `--remove <value>` | list<string> | no | Remove matching item(s) from a list field, `key=value` (repeatable) |
| `--add-source <value>` | list<string> | no | Add a standard source, `resource=<path-or-uri>[,kind=<extension>][,id=...,...]` |
| `--add-source-json <value>` | list<string> | no | Add a full source mapping as JSON/YAML or `@file` (repeatable) |
| `--remove-source <value>` | list<string> | no | Remove sources matching `<path-or-uri>` or `resource=<path-or-uri>[,kind=<kind>]` (repeatable) |
| `--set-body <value>` | string | no | Replace the whole body. Use `@file` to read a file or `-` for stdin |
| `--append-body <value>` | string | no | Append a block to the body. Use `@file` or `-` (stdin) |
| `--clear-body` | bool | no | Empty the body |
| `--set-section <HEADING> <TEXT>` | list<string> | no | Replace a section's content, `<heading> <text>` (repeatable). Text accepts `@file`/`-` |
| `--append-section <HEADING> <TEXT>` | list<string> | no | Append to a section, `<heading> <text>` (repeatable). Text accepts `@file`/`-` |
| `--remove-section <value>` | list<string> | no | Remove a section (heading + content), `<heading>` (repeatable) |

Output stream: `change`.

### `okf init` · _mutates_

Create a new empty OKF bundle.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory to create (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--title <value>` | string | no | Title for the scaffolded root `index.md` |
| `--no-index` | bool | no | Do not scaffold a root `index.md` |
| `--no-ontology` | bool | no | Do not scaffold an `ontology.yaml` |

Output stream: `change`.

### `okf mv` · _mutates_

Move/rename a concept and rewrite every inbound link.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<old>` | positional | yes | Existing concept id |
| `<new>` | positional | yes | New concept id |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Output stream: `change`.

### `okf ontology add` · _mutates_

Define a new concept type with its fields and reference rules.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | positional | yes | Concept type name |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--description <value>` | string | no | Description of the concept type |
| `--field <value>` | list<string> | no | A typed field, `key:type[:required][:v1|v2|...]` (repeatable) |
| `--ref <value>` | list<string> | no | A reference rule, `key:Target[|Target2]:cardinality` (repeatable) |
| `--remove-field <value>` | list<string> | no | Remove a typed field (update only; repeatable) |
| `--remove-ref <value>` | list<string> | no | Remove a reference rule (update only; repeatable) |
| `--attested` | bool | no | Mark the exact `Attested Computation` type as standard attested |

Output stream: `change`.

### `okf ontology remove` · _mutates_

Remove a concept type.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | positional | yes | Concept type name to remove |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Output stream: `change`.

### `okf ontology update` · _mutates_

Modify fields/references of an existing concept type.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | positional | yes | Concept type name |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--description <value>` | string | no | Description of the concept type |
| `--field <value>` | list<string> | no | A typed field, `key:type[:required][:v1|v2|...]` (repeatable) |
| `--ref <value>` | list<string> | no | A reference rule, `key:Target[|Target2]:cardinality` (repeatable) |
| `--remove-field <value>` | list<string> | no | Remove a typed field (update only; repeatable) |
| `--remove-ref <value>` | list<string> | no | Remove a reference rule (update only; repeatable) |
| `--attested` | bool | no | Mark the exact `Attested Computation` type as standard attested |

Output stream: `change`.

### `okf refresh` · _mutates_

Re-record source fingerprints after a change is acknowledged.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to refresh |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--fail-on <value>` | string | no | Fail (exit 1) when any source is skipped: never (default) | skipped | any |

Output stream: `change`.

### `okf rm` · _mutates_

Remove a concept; refuse if backlinks would dangle unless `--force`.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to remove |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--force` | bool | no | Remove even if backlinks would dangle |

Output stream: `change`.

### `okf verify` · _mutates_

Append a `verified` entry (the write-side of trust).

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to verify |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--by <value>` | string | yes | The reviewing actor (e.g. `human:andrey` or `process:ci`) |

Output stream: `change`.

## render

### `okf docs`

Generate documentation from a bundle.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--format <value>` | string | no | Output format: md|html|pdf|graphml|obsidian|index (default: `md`) |

Output stream: `docs`.
