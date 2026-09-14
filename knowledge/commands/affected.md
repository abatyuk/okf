---
type: Command
title: okf affected
description: 'Impact query: given a set of changed links, computes the blast radius of concepts that may need review via a reverse walk, direct by default or --transitive.'
group: check
mutates: false
implemented_by:
- /components/graph
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:06Z
---
# okf affected

Impact query: given a set of changed links, computes the blast radius of concepts that may need review via a reverse walk, direct by default or --transitive.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--changed <value>` | list<string> | no | A changed link/concept-id/resource (repeatable; also read from stdin lines) |
| `--transitive` | bool | no | Follow the cascade past direct dependents (default: `false`) |
| `--depth <value>` | int | no | Cap the number of hops when `--transitive` |
| `--fail-on <value>` | string | no | Fail (exit 1) on any affected concept: never (default) | info | warn | error | any (default: `never`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `affected`.
