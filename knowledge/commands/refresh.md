---
type: Command
title: okf refresh
description: Re-records source fingerprints after a change has been acknowledged, clearing the drift that stale would report.
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
# okf refresh

Re-records source fingerprints after a change has been acknowledged, clearing the drift that stale would report.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to refresh |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--fail-on <value>` | string | no | Fail (exit 1) when any source is skipped: never (default) | skipped | any |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `change`.
