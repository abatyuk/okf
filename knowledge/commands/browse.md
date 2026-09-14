---
type: Command
title: okf browse
description: Reads a directory's index.md for progressive disclosure, synthesizing it when absent.
group: query
mutates: false
implemented_by:
- /components/query
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
- resource: crates/okf-cli/src/commands/query.rs
  kind: git-path
  fingerprint:
    blob_sha: 079709086641c8a57a21c3da0c2f2e027af390f3
last_modified: 2026-09-09T13:13:42Z
---
# okf browse

Reads a directory's checked-in `index.md` for specification-native progressive disclosure. If the file is absent, synthesizes the same directory listing in memory from bundle concepts without writing it.

Use the root view first, then pass `--directory <path>` to descend into a promising branch. JSON output identifies whether the content came from a file or was synthesized.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--directory <value>` | string | no | Bundle-relative directory to browse (default: root `/`) (default: `/`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `index`.
