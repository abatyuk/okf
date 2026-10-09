---
type: Command
title: okf computation check
description: Inspects an Attested Computation contract and its artifacts without executing the computation.
group: check
mutates: false
implemented_by:
- /components/check
- /components/query
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: bab34f363d28125adac33f33fefc2c9e04329d73
last_modified: 2026-10-04T20:39:29Z
---
# okf computation check

Checks the exact type, required runtime, parameters, inline-or-file computation choice, executor receipt, and attester resources. Its execution state is always `not-run`.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--details` | bool | no | Compatibility option; detailed output is not implemented for this command (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `computation-contract`.
