---
type: Command
title: okf schema
description: 'Prints machine-readable CLI metadata as NDJSON: every command with its group, mutates flag, args, and output shape.'
group: meta
mutates: false
implemented_by:
- /components/okf-cli
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 7a22138be071fc20ede774d812f9f98d018e7341
last_modified: 2026-09-07T18:22:07Z
---
# okf schema

Prints machine-readable CLI metadata as NDJSON: every command with its group, mutates flag, args, and output shape. Derived from the clap command tree, never hand-maintained, so an agent can discover the whole surface without hard-coding it.

## Arguments

_No arguments._

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `schema`.
