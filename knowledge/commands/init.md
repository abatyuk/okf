---
type: Command
title: okf init
description: 'Creates a new empty OKF bundle: base structure plus a starter ontology.yaml.'
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
last_modified: 2026-10-04T20:39:29Z
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

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `change`.
