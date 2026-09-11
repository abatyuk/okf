---
type: Command
title: okf mv
description: Moves or renames a concept and rewrites every inbound link.
group: mutate
mutates: true
implemented_by:
- /components/mutate
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 7a22138be071fc20ede774d812f9f98d018e7341
last_modified: 2026-09-07T18:22:06Z
---
# okf mv

Moves or renames a concept and rewrites body links, ontology references, and standard internal `sources[].resource` paths. It also rebases the moved concept's relative standard paths.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<old>` | positional | yes | Existing concept id |
| `<new>` | positional | yes | New concept id |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `change`.
