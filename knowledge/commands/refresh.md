---
type: Command
title: okf refresh
description: Re-records source fingerprints after a change has been acknowledged, clearing the drift that stale would report.
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
# okf refresh

Re-records extension source fingerprints after a change has been acknowledged. It does not overwrite standard source modification time or clear an expired `stale_after`.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to refresh |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--fail-on <value>` | string | no | Fail (exit 1) when any source is skipped: never (default) | skipped | any |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `change`.
