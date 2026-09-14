---
type: Command
title: okf stale
description: Detects extension fingerprint drift and lifecycle expiry using offset-aware, inclusive stale_after comparison.
group: check
mutates: false
implemented_by:
- /components/check
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:07Z
---
# okf stale

Compares recorded source fingerprint extensions against recomputed values and applies exact `now >= stale_after` lifecycle semantics with explicit-offset instant parsing.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--fail-on <value>` | string | no | Fail (exit 1) on any result: never (default) | info | warn | error | any (default: `never`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `drift`.
