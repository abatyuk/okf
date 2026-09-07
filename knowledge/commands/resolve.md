---
type: Command
title: okf resolve
description: Resolves a link or concept id to a concrete bundle-relative file path.
group: query
mutates: false
implemented_by:
- /components/query
sources:
- resource: ../crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: a1bdcb78fa383c159cc63689e31430beb78d30b4
last_modified: 2026-09-07T18:22:06Z
---
# okf resolve

Resolves a link or concept id to a concrete bundle-relative file path.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<link>` | positional | yes | Link or concept id to resolve |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--from <value>` | string | no | Resolve a relative link against this containing concept id |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `resolved`.
