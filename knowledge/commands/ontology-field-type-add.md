---
type: Command
title: okf ontology field-type add
description: Adds a reusable structured field type to the ontology.
group: mutate
mutates: true
implemented_by:
- /components/ontology
sources:
- resource: crates/okf-core/src/ontology/edit.rs
  kind: git-path
  fingerprint:
    blob_sha: 885cd3dedad75883309028b70b863cc31931c19b
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 6938cba0788280234436282cdce1e4a880b25d22
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: bab34f363d28125adac33f33fefc2c9e04329d73
last_modified: 2026-10-05T10:16:06Z
---
# okf ontology field-type add

Adds a reusable structured field type to the ontology.

Requires a full definition via `--from`. Existing names are rejected. The resulting ontology
validates before persistence; `--dry-run` previews without writing.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | positional | yes | Reusable field-type name |
| `<bundle>` | positional | no | Bundle directory |
| `--from <value>` | string | yes | Complete field-type definition: inline YAML, @file, a file path, or stdin (-) |
| `--dry-run` | bool | no | Validate and preview the change without writing (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `change`.
