---
type: Command
title: okf schema
description: 'Prints machine-readable CLI metadata as NDJSON: commands, mutation conditions, typed arguments, resolution, and output shapes.'
group: meta
mutates: false
implemented_by:
- /components/okf-cli
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
# okf schema

Prints machine-readable CLI metadata as NDJSON: every command with its group, mutation
capability and conditions, typed arguments, possible values, bundle-resolution metadata, and
output shape. Derived from the clap command tree, never hand-maintained, so an agent can discover
the whole surface without hard-coding it.

## Arguments

_No arguments._

This command is always NDJSON; `--json` is accepted but unnecessary.

Output stream: `schema`.
