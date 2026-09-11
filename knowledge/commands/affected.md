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
    blob_sha: 7a22138be071fc20ede774d812f9f98d018e7341
last_modified: 2026-09-07T18:22:06Z
---
# okf affected

Impact query: given a set of changed links, computes the blast radius of concepts that may need review via a reverse walk, direct by default or --transitive.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--changed <value>` | list<string> | no | A changed link/concept-id/resource (repeatable; also read from stdin lines) |
| `--transitive` | bool | no | Follow the cascade past direct dependents |
| `--depth <value>` | string | no | Cap the number of hops when `--transitive` |
| `--fail-on <value>` | string | no | Fail (exit 1) on any affected concept: never (default) | info | warn | error | any |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `affected`.
