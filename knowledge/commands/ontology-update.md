---
type: Command
title: okf ontology update
description: Modifies the fields or references of an existing concept type.
group: mutate
mutates: true
implemented_by:
- /components/ontology
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:06Z
---
# okf ontology update

Modifies the fields or references of an existing concept type.

## Arguments

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

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `change`.
