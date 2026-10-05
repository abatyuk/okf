---
type: Command
title: okf show
description: Shows a concept in full, as a heading outline with line numbers, or as a selected line range.
group: query
mutates: false
implemented_by:
- /components/query
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
# okf show

Shows one concept's full content by default. `--outline` returns only Markdown headings with
their 1-based physical document line numbers; use those numbers with `--lines START:END` to
retrieve an inclusive slice instead of loading a large document in full. Both focused modes
have structured `--json` representations.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--outline` | bool | no | Show only the Markdown heading outline with 1-based document line numbers (default: `false`) |
| `--lines <value>` | string | no | Show only an inclusive 1-based document line range, `START:END` (or one line, `N`) |
| `--numbered` | bool | no | Print document line numbers (including frontmatter); excludes the display header (default: `false`) |
| `--body` | bool | no | Print only the raw Markdown body, without frontmatter or display headers (default: `false`) |
| `--project <value>` | list<string> | no | Explicit metadata projection selector (repeatable) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `concept,projection`.
