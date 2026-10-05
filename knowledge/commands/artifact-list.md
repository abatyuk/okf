---
type: Command
title: okf artifact list
description: Inventories concepts, reserved files, and opaque artifacts under a bundle directory such as references/.
group: query
mutates: false
implemented_by:
- /components/query
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: bab34f363d28125adac33f33fefc2c9e04329d73
last_modified: 2026-10-04T20:39:29Z
---
# okf artifact list

Lists bundle files with their artifact classification and size, optionally computing a SHA-256 digest. The `references/` directory is supported as an optional OKF convention, not required.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle root directory; use --directory to select a directory within the bundle |
| `--directory <value>` | string | no | Bundle-relative directory to inventory (default: `references`) |
| `--digest` | bool | no | Compute SHA-256 digests (reads each file) (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `artifact`.
