---
type: Command
title: okf source-scan
description: Inventories every regular file in a source directory without parsing it as an OKF bundle.
group: check
mutates: false
implemented_by:
- /components/check
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: bab34f363d28125adac33f33fefc2c9e04329d73
last_modified: 2026-10-04T20:39:29Z
---
# okf source-scan

Maps a repository or directory for ingest workflows. Unlike [okf scan](scan.md), it includes non-Markdown source artifacts and performs no concept parsing.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<directory>` | positional | yes | Source directory to inventory; it need not be an OKF bundle |

Global `--json` requests NDJSON.

Output stream: `source-file`.
