---
type: Command
title: okf doctor
description: Diagnoses conformance and compatibility changes before upgrading an existing bundle, with conservative safe repairs.
group: check
mutates: true
implemented_by:
- /components/check
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: bab34f363d28125adac33f33fefc2c9e04329d73
last_modified: 2026-10-04T20:39:29Z
---
# okf doctor

Inventories an existing bundle with a tolerant raw-byte walker, reports hard conformance blockers and behavior changes, and can apply only allow-listed safe repairs when explicitly confirmed.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--target <value>` | string | no | Target OKF version (default: `0.2`) |
| `--fix-safe` | bool | no | Enable the allow-listed safe repair set (default: `false`) |
| `--dry-run` | bool | no | Show safe repairs without writing (the default without --yes) (default: `false`) |
| `--yes` | bool | no | Confirm applying --fix-safe changes non-interactively (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `doctor-finding,doctor-summary`.
