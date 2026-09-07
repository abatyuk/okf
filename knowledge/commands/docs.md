---
type: Command
title: okf docs
description: 'Generates documentation from a bundle: progressive-disclosure index.md files, or html, md, pdf, graphml, or obsidian output via --format.'
group: render
mutates: false
implemented_by:
- /components/render
sources:
- resource: ../crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: a1bdcb78fa383c159cc63689e31430beb78d30b4
last_modified: 2026-09-07T18:22:06Z
---
# okf docs

Generates documentation from a bundle: progressive-disclosure index.md files, or html, md, pdf, graphml, or obsidian output via --format.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--format <value>` | string | no | Output format: md|html|pdf|graphml|obsidian|index (default: `md`) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `docs`.
