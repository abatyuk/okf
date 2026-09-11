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
    blob_sha: 7a22138be071fc20ede774d812f9f98d018e7341
- resource: crates/okf-cli/src/commands/query.rs
  kind: git-path
  fingerprint:
    blob_sha: 98fcf5b787e69c801489823ffea093f747e64d22
last_modified: 2026-09-09T13:13:42Z
---
# okf browse

Reads a directory's checked-in `index.md` for specification-native progressive disclosure. If the file is absent, synthesizes the same directory listing in memory from bundle concepts without writing it.

Use the root view first, then pass `--directory <path>` to descend into a promising branch. JSON output identifies whether the content came from a file or was synthesized.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--directory <value>` | string | no | Bundle-relative directory to browse (default: root `/`) (default: `/`) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `index`.
