---
type: Command
title: okf edit
description: Losslessly edits a concept's frontmatter or body and invalidates its prior verification.
group: mutate
mutates: true
implemented_by:
- /components/mutate
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: bab34f363d28125adac33f33fefc2c9e04329d73
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 6938cba0788280234436282cdce1e4a880b25d22
- resource: crates/okf-cli/src/structured.rs
  kind: git-path
  fingerprint:
    blob_sha: 889819283f7bd1c986e0ce78deb39f77fd481633
- resource: crates/okf-core/src/mutate/structured.rs
  kind: git-path
  fingerprint:
    blob_sha: 6c504c52f0f9f23b83fde8b312aae82dafcc8c78
last_modified: 2026-10-05T10:16:06Z
---
# okf edit

Losslessly edits a concept in place. Frontmatter: `--set key=value` (scalar), `--unset key`,
`--add key=value` (append a list item, idempotent), `--remove key=value` (drop list items).
Body: `--set-body` / `--append-body` / `--clear-body`, or the section-aware `--set-section` /
`--append-section` / `--remove-section` (heading matched by GitHub slug). Body/section text
accepts `@file` or `-` (stdin). Unrelated keys and their values are preserved; YAML presentation may normalize.
Every meaningful edit removes the concept's `verified` entries, so its derived trust tier returns
to `unverified` until the revised document is reviewed and verified again.

Structured edits use `--set-yaml key=value` to replace a complete value, `--set-path path=value`
to set an object property, and `--unset-path path` to remove one. Dotted scalar `--set` keys remain
literal. `--patch` accepts an RFC 6902 YAML/JSON array for concrete JSON Pointer operations,
including list edits, move/copy and test guards. Failed patches leave the file unchanged.
`--dry-run` previews the resulting change without persistence. Meaningful mutations clear current
verification and update an existing generation timestamp; previews do neither. Unrelated values
and body text survive, while YAML presentation may normalize.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to edit |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--set <value>` | list<string> | no | Set/update a scalar field, `key=value` (repeatable) |
| `--set-yaml <value>` | list<string> | no | Replace a field with YAML: `key=value`, `key=@file`, or `key=-` (repeatable) |
| `--set-path <value>` | list<string> | no | Set an object path with a YAML value, e.g. `deadline.within=72` (repeatable) |
| `--dry-run` | bool | no | Validate and show the resulting diff without writing (default: `false`) |
| `--unset <value>` | list<string> | no | Remove a field entirely, `key` (repeatable) |
| `--unset-path <value>` | list<string> | no | Delete an object path (repeatable); absent keys are a no-op |
| `--patch <value>` | string | no | Apply an RFC 6902 JSON Patch to frontmatter, supplied as YAML, `@file`, or `-` |
| `--add <value>` | list<string> | no | Append an item to a list field, `key=value` (repeatable, idempotent) |
| `--remove <value>` | list<string> | no | Remove matching item(s) from a list field, `key=value` (repeatable) |
| `--add-source <value>` | list<string> | no | Add a standard source, `resource=<path-or-uri>[,kind=<extension>][,id=...,...]` |
| `--add-source-json <value>` | list<string> | no | Add a full source mapping as JSON/YAML or `@file` (repeatable) |
| `--remove-source <value>` | list<string> | no | Remove sources matching `<path-or-uri>` or `resource=<path-or-uri>[,kind=<kind>]` (repeatable) |
| `--set-body <value>` | string | no | Replace the whole body. Use `@file` to read a file or `-` for stdin |
| `--append-body <value>` | string | no | Append a block to the body. Use `@file` or `-` (stdin) |
| `--replace <OLD> <NEW>` | list<string> | no | Replace body text, `<old> <new>` (repeatable; defaults to exactly one match) |
| `--all` | bool | no | Replace every occurrence matched by `--replace` (default: `false`) |
| `--clear-body` | bool | no | Empty the body (default: `false`) |
| `--set-section <HEADING> <TEXT>` | list<string> | no | Replace a section's content, `<heading> <text>` (repeatable). Text accepts `@file`/`-` |
| `--append-section <HEADING> <TEXT>` | list<string> | no | Append to a section, `<heading> <text>` (repeatable). Text accepts `@file`/`-` |
| `--remove-section <value>` | list<string> | no | Remove a section (heading + content), `<heading>` (repeatable) |
| `--rename-section <OLD> <NEW>` | list<string> | no | Rename a section heading, `<old> <new>` (repeatable) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `change`.
