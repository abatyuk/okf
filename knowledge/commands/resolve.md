---
type: Command
title: okf resolve
description: Resolves a link or concept id to a concrete bundle-relative file path.
group: query
mutates: false
implemented_by:
- /components/query
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: bab34f363d28125adac33f33fefc2c9e04329d73
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 6938cba0788280234436282cdce1e4a880b25d22
last_modified: 2026-10-04T20:39:29Z
---
# okf resolve

Resolves a concept [Link](../types/Link.md) to a concrete bundle-relative Markdown path using standard document-relative semantics. Use [okf artifact resolve](artifact-resolve.md) for arbitrary path-valued resources.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<link>` | positional | yes | Link or concept id to resolve |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--from <value>` | string | no | Resolve a relative link against this containing concept id |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `resolved,scope,bundle-edge`.
