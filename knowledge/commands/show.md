---
type: Command
title: okf show
description: 'Shows one concept''s full content: frontmatter plus body, or the concept record under --json.'
group: query
mutates: false
implemented_by:
- /components/query
sources:
- resource: ../crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: a1bdcb78fa383c159cc63689e31430beb78d30b4
last_modified: 2026-09-07T18:22:07Z
---
# okf show

Shows one concept's full content: frontmatter plus body, or the concept record under --json.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `concept`.
