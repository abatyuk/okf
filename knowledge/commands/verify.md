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
    blob_sha: 7a22138be071fc20ede774d812f9f98d018e7341
last_modified: 2026-09-07T18:22:07Z
---
# okf verify

Appends a valid document-level `{by, at}` verification event after content has been checked against its sources or resource. This is distinct from per-run computation attestation.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id to verify |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--by <value>` | string | yes | The reviewing actor (e.g. `human:andrey` or `process:ci`) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `change`.
