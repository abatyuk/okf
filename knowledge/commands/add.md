---
type: Command
title: okf add
description: Adds a new concept document, scaffolded from the ontology so required fields and reference keys start present.
group: mutate
mutates: true
implemented_by:
- /components/mutate
sources:
- resource: ../crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: a1bdcb78fa383c159cc63689e31430beb78d30b4
last_modified: 2026-09-07T18:22:06Z
---
# okf add

Adds a new concept document, scaffolded from the ontology so required fields and reference keys start present. --attested scaffolds an OKF Attested Computation.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<path>` | positional | yes | Bundle-relative path of the new concept (with or without `.md`) |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--type <value>` | string | no | Concept `type` (an ontology concept-type key). Optional with `--attested` |
| `--title <value>` | string | no | Concept title |
| `--description <value>` | string | no | Concept description |
| `--attested` | bool | no | Scaffold an OKF Attested Computation (computation/executor/attester) |
| `--set <value>` | list<string> | no | Set a custom scalar field at creation, `key=value` (repeatable) |
| `--ref <value>` | list<string> | no | Set a declared reference at creation, `key=link` (repeatable) |
| `--add-source <value>` | list<string> | no | Add a structured source, `resource=<path-or-uri>,kind=<kind>` (repeatable) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `change`.
