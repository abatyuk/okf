---
type: Command
title: okf verify
description: Appends a verified entry, the write side of trust, raising a concept's derived tier (unverified to machine-confirmed to human-reviewed).
group: mutate
mutates: true
implemented_by:
- /components/mutate
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:07Z
---
# okf verify

Appends a valid document-level `{by, at}` verification event after content has been checked against its sources or resource. This is distinct from per-run computation attestation.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to verify |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--by <value>` | string | yes | The reviewing actor (e.g. `human:andrey` or `process:ci`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `change`.
