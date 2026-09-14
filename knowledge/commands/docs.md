---
type: Command
title: okf docs
description: 'Generates documentation from a bundle: progressive-disclosure index.md files, or html, md, pdf, graphml, or obsidian output via --format.'
group: render
mutates: true
implemented_by:
- /components/render
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:06Z
---
# okf docs

Generates documentation from a bundle: progressive-disclosure index.md files, or html, md, pdf, graphml, or obsidian output via --format.
Only `--format index` mutates the bundle. Other formats write an artifact to stdout; with
`--json`, that artifact is wrapped in one `docs` record containing its format and content.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--format <value>` | string | no | Output format: md|html|pdf|graphml|obsidian|index (default: `md`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `docs,change`.
