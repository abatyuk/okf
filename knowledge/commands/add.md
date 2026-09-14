---
type: Command
title: okf add
description: Adds a new concept document, scaffolded from the ontology so required fields and reference keys start present.
group: mutate
mutates: true
implemented_by:
- /components/mutate
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:06Z
---
# okf add

Adds a new concept document, scaffolded from the ontology so required fields and reference keys start present. `--attested` creates exact `type: Attested Computation` and requires a runtime plus either `--computation` or `--inline-computation`.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<path>` | positional | yes | Bundle-relative path of the new concept (with or without `.md`) |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--type <value>` | string | no | Concept `type` (an ontology concept-type key). Optional with `--attested` |
| `--title <value>` | string | no | Concept title |
| `--description <value>` | string | no | Concept description |
| `--attested` | bool | no | Scaffold exact `type: Attested Computation`; requires `--runtime` (default: `false`) |
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

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `change`.
