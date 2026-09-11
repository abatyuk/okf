---
type: Command
title: okf search
description: Searches concepts by type, tag, free text, and/or frontmatter field, returning matching concept records.
group: query
mutates: false
implemented_by:
- /components/query
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 7a22138be071fc20ede774d812f9f98d018e7341
last_modified: 2026-09-07T18:22:07Z
---
# okf search

Searches concepts by type, tag, free text, and/or frontmatter field, returning matching concept records.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--type <value>` | string | no | Filter by exact concept `type` |
| `--tag <value>` | string | no | Filter by membership in the concept's `tags` |
| `--text <value>` | string | no | Filter by case-insensitive substring across id/title/description/body |
| `--field <value>` | list<string> | no | Filter by a frontmatter field, `key=value` (repeatable; AND) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `concept`.
