---
type: Command
title: okf ontology add
description: Defines a new concept type with its typed fields and reference rules, written to ontology.yaml through the lossless ontology editor.
group: mutate
mutates: true
implemented_by:
- /components/ontology
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: bab34f363d28125adac33f33fefc2c9e04329d73
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 6938cba0788280234436282cdce1e4a880b25d22
- resource: crates/okf-core/src/ontology/edit.rs
  kind: git-path
  fingerprint:
    blob_sha: 885cd3dedad75883309028b70b863cc31931c19b
- resource: crates/okf-cli/src/structured.rs
  kind: git-path
  fingerprint:
    blob_sha: 889819283f7bd1c986e0ce78deb39f77fd481633
last_modified: 2026-10-05T10:16:06Z
---
# okf ontology add

Defines a new concept type with its typed fields and reference rules, written to ontology.yaml through the lossless ontology editor.

Structured declaration flags `--field-yaml`, `--ref-yaml`, and `--relationship-yaml` accept
`name=value` with inline YAML/JSON, `@file`, or stdin `-`. Each supplied declaration replaces its
whole named entry. `--from` accepts one concept-type definition; omitted named entries remain
on update. Overlapping file/flag operations fail. `--remove-relationship` is available on update.
The combined prospective ontology validates before writing; `--dry-run` previews without writes.

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
| `--field-yaml <value>` | list<string> | no | A complete field declaration, key=YAML|@file|- (repeatable) |
| `--ref-yaml <value>` | list<string> | no | A complete reference declaration, key=YAML|@file|- (repeatable) |
| `--relationship-yaml <value>` | list<string> | no | A complete relationship declaration, key=YAML|@file|- (repeatable) |
| `--remove-relationship <value>` | list<string> | no | Remove a relationship (update only; repeatable) |
| `--from <value>` | string | no | Merge a concept-type definition from inline YAML, @file, a file path, or stdin (-) |
| `--dry-run` | bool | no | Validate and preview the change without writing (default: `false`) |
| `--attested` | bool | no | Mark the exact `Attested Computation` type as standard attested (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `change`.
