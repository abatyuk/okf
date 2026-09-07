---
type: Command
title: okf ontology update
description: Modifies the fields or references of an existing concept type.
group: mutate
mutates: true
implemented_by:
- /components/ontology
sources:
- resource: ../crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: a1bdcb78fa383c159cc63689e31430beb78d30b4
last_modified: 2026-09-07T18:22:06Z
---
# okf ontology update

Modifies the fields or references of an existing concept type.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | positional | yes | Concept type name |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--description <value>` | string | no | Description of the concept type |
| `--field <value>` | list<string> | no | A typed field, `key:type[:required][:v1|v2|...]` (repeatable) |
| `--reference <value>` | list<string> | no | A reference rule, `key:Target[|Target2]:cardinality` (repeatable) |
| `--attested` | bool | no | Mark the concept type as an attested computation |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `change`.
