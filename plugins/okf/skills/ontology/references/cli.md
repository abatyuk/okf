# okf CLI — ontology command reference

> **Generated** by `cargo xtask docs` from `okf schema --json` (tool 0.2.4, OKF spec 0.2). Do not hand-edit; regenerate instead.

This focused reference contains only commands selected for the `ontology` workflow. Consult it when exact arguments or output shapes are needed. If the installed `okf` version differs from the generated tool version above, or rejects documented syntax, use that command's `--help` output as the runtime authority.

Commands with a human form accept global `--json` for NDJSON; `schema` is always NDJSON. Bundle-aware commands take an optional trailing `bundle` positional resolved as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd. Meta commands have no bundle, and `source-scan` takes an explicit arbitrary directory.

## query

### `okf artifact resolve`

Resolve any OKF path-valued resource with document context.

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<resource>` | positional | yes | Resource path, URL, or scope descriptor |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--from <value>` | string | no | Resolve a relative resource against this declaring concept id |

Output stream: `artifact-resolution`.

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
