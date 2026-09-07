---
type: Command
title: okf rm
description: Removes a concept, refusing if backlinks would dangle unless --force is given.
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
# okf rm

Removes a concept, refusing if backlinks would dangle unless --force is given.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to remove |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--force` | bool | no | Remove even if backlinks would dangle |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `change`.
