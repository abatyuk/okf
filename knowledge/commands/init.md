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
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:06Z
---
# okf init

Creates a new empty OKF bundle: base structure plus a starter ontology.yaml. Does not invent concepts.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory to create (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--title <value>` | string | no | Title for the scaffolded root `index.md` |
| `--no-index` | bool | no | Do not scaffold a root `index.md` (default: `false`) |
| `--no-ontology` | bool | no | Do not scaffold an `ontology.yaml` (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `change`.
