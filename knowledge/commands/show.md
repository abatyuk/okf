---
type: Command
title: okf show
description: 'Shows a concept in full, as a heading outline with line numbers, or as a selected line range.'
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
# okf show

Shows one concept's full content by default. `--outline` returns only Markdown headings with
their 1-based physical document line numbers; use those numbers with `--lines START:END` to
retrieve an inclusive slice instead of loading a large document in full. Both focused modes
have structured `--json` representations.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--outline` | bool | no | Show only the Markdown heading outline with 1-based document line numbers |
| `--lines <value>` | string | no | Show only an inclusive 1-based document line range, `START:END` (or one line, `N`) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `concept`.
