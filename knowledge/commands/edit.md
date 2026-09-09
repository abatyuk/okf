---
type: Command
title: okf edit
description: Losslessly edits a concept's frontmatter or body and invalidates its prior verification.
group: mutate
mutates: true
implemented_by:
- /components/mutate
sources:
- resource: ../crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: a1bdcb78fa383c159cc63689e31430beb78d30b4
last_modified: 2026-09-07T18:22:06Z
---
# okf edit

Losslessly edits a concept in place. Frontmatter: `--set key=value` (scalar), `--unset key`,
`--add key=value` (append a list item, idempotent), `--remove key=value` (drop list items).
Body: `--set-body` / `--append-body` / `--clear-body`, or the section-aware `--set-section` /
`--append-section` / `--remove-section` (heading matched by GitHub slug). Body/section text
accepts `@file` or `-` (stdin). Key order and unknown keys are always preserved.
Every successful edit removes the concept's `verified` entries, so its derived trust tier returns
to `unverified` until the revised document is reviewed and verified again.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to edit |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--set <value>` | list<string> | no | Set/update a scalar field, `key=value` (repeatable) |
| `--unset <value>` | list<string> | no | Remove a field entirely, `key` (repeatable) |
| `--add <value>` | list<string> | no | Append an item to a list field, `key=value` (repeatable, idempotent) |
| `--remove <value>` | list<string> | no | Remove matching item(s) from a list field, `key=value` (repeatable) |
| `--add-source <value>` | list<string> | no | Add a structured source, `resource=<path-or-uri>,kind=<kind>` (repeatable) |
| `--remove-source <value>` | list<string> | no | Remove sources matching `<path-or-uri>` or `resource=<path-or-uri>[,kind=<kind>]` (repeatable) |
| `--set-body <value>` | string | no | Replace the whole body. Use `@file` to read a file or `-` for stdin |
| `--append-body <value>` | string | no | Append a block to the body. Use `@file` or `-` (stdin) |
| `--clear-body` | bool | no | Empty the body |
| `--set-section <HEADING> <TEXT>` | list<string> | no | Replace a section's content, `<heading> <text>` (repeatable). Text accepts `@file`/`-` |
| `--append-section <HEADING> <TEXT>` | list<string> | no | Append to a section, `<heading> <text>` (repeatable). Text accepts `@file`/`-` |
| `--remove-section <value>` | list<string> | no | Remove a section (heading + content), `<heading>` (repeatable) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `change`.
