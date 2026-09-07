---
type: Command
title: okf stale
description: 'Drift detection: compares recorded source fingerprints against recomputed ones (plus any stale_after date) to find concepts whose sources have moved.'
group: check
mutates: false
implemented_by:
- /components/check
sources:
- resource: ../crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: a1bdcb78fa383c159cc63689e31430beb78d30b4
last_modified: 2026-09-07T18:22:07Z
---
# okf stale

Drift detection: compares recorded source fingerprints against recomputed ones (plus any stale_after date) to find concepts whose sources have moved.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--fail-on <value>` | string | no | Fail (exit 1) on any result: never (default) | info | warn | error | any |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `drift`.
