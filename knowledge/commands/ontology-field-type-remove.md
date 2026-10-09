---
type: Command
title: okf ontology field-type remove
description: Removes a reusable structured field type from the ontology.
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
# okf ontology field-type remove

Removes a reusable structured field type from the ontology.

Removal validates the resulting ontology and refuses to strand unresolved type references.
Use bulk apply to update dependent definitions in the same write. `--dry-run` previews removal.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<name>` | positional | yes | Reusable field-type name to remove |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--dry-run` | bool | no | Validate and preview the change without writing (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `change`.
