---
type: Command
title: okf docs
description: 'Generates documentation from a bundle: progressive-disclosure index.md files, or html, md, pdf, graphml, or obsidian output via --format.'
group: render
mutates: true
implemented_by:
- /components/render
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: bab34f363d28125adac33f33fefc2c9e04329d73
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 6938cba0788280234436282cdce1e4a880b25d22
last_modified: 2026-10-04T20:39:29Z
---
# okf docs

Generates documentation from a bundle: progressive-disclosure index.md files, or html, md, pdf, graphml, or obsidian output via --format.
Only `--format index` mutates the bundle. Other formats write an artifact to stdout; with
`--json`, that artifact is wrapped in one `docs` record containing its format and content.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--format <value>` | string | no | Output format: md|html|graphml|obsidian|index; pdf is retained but unavailable (default: `md`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `docs,change`.
