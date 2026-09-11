---
type: Command
title: okf init
description: 'Creates a new empty OKF bundle: base structure plus a starter ontology.yaml.'
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
# okf init

Creates a new empty OKF bundle: base structure plus a starter ontology.yaml. Does not invent concepts.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory to create (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--title <value>` | string | no | Title for the scaffolded root `index.md` |
| `--no-index` | bool | no | Do not scaffold a root `index.md` |
| `--no-ontology` | bool | no | Do not scaffold an `ontology.yaml` |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `change`.
