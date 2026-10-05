---
type: Command
title: okf ontology apply
description: Applies dependent ontology declarations and explicit removals in one validated write.
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
# okf ontology apply

Applies dependent ontology declarations and explicit removals in one validated write.

Input `--from` accepts inline YAML/JSON, `@file`, stdin `-`, or a definition file path. The change
envelope has `field_types` and `concepts` maps, preserving omitted named declarations. Top-level
`remove` lists `field_types` and `concepts`; per-concept `remove` lists `fields`, `references`, and
`relationships`. Supplied declarations replace completely, and conflicting operations fail.
The final ontology validates before one write. `--dry-run` previews without persistence.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory |
| `--from <value>` | string | yes | Coordinated change document: inline YAML, @file, a file path, or stdin (-) |
| `--dry-run` | bool | no | Validate and preview the change without writing (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `change`.
