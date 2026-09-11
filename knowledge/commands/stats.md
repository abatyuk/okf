---
type: Command
title: okf stats
description: 'Bundle summary: counts by type, trust-tier distribution, corrected stale count, and orphan count.'
group: check
mutates: false
implemented_by:
- /components/query
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 7a22138be071fc20ede774d812f9f98d018e7341
last_modified: 2026-09-07T18:22:07Z
---
# okf stats

Bundle summary: counts by type, trust-tier distribution, corrected stale count, and orphan count.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--fail-on <value>` | string | no | Fail (exit 1) on any result: never (default) | info | warn | error | any |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `stats`.
