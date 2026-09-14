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
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:06Z
---
# okf refresh

Re-records extension source fingerprints after a change has been acknowledged. It does not overwrite standard source modification time or clear an expired `stale_after`.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to refresh |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--fail-on <value>` | string | no | Fail (exit 1) when any source is skipped: never (default) | skipped | any (default: `never`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `change`.
