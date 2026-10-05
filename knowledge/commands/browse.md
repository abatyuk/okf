---
type: Command
title: okf browse
description: Reads a directory's index.md for progressive disclosure, synthesizing it when absent.
group: query
mutates: false
implemented_by:
- /components/query
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: 126dddfab3fb8caa78cfd61396db89dbcc324a33
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 28db6285f18a0400ab4b08c79c46322e58c2d833
- resource: crates/okf-cli/src/commands/query.rs
  kind: git-path
  fingerprint:
    blob_sha: c9b643257e90f1e3e225cb4613be98f08f6abd02
last_modified: 2026-10-04T20:39:29Z
---
# okf browse

Reads a directory's checked-in `index.md` for specification-native progressive disclosure. If the file is absent, synthesizes the same directory listing in memory from bundle concepts without writing it.

Use the root view first, then pass `--directory <path>` to descend into a promising branch. JSON output identifies whether the content came from a file or was synthesized.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--directory <value>` | string | no | Bundle-relative directory to browse (default: root `/`) (default: `/`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `index`.
