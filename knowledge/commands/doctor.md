---
type: Command
title: okf doctor
description: Diagnoses conformance and compatibility changes before upgrading an existing bundle, with conservative safe repairs.
group: check
mutates: true
implemented_by:
- /components/check
---
# okf doctor

Inventories an existing bundle with a tolerant raw-byte walker, reports hard conformance blockers and behavior changes, and can apply only allow-listed safe repairs when explicitly confirmed.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--target <value>` | string | no | Target OKF version (default: `0.2`) |
| `--fix-safe` | bool | no | Enable the allow-listed safe repair set |
| `--dry-run` | bool | no | Show safe repairs without writing (the default without --yes) |
| `--yes` | bool | no | Confirm applying --fix-safe changes non-interactively |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `doctor-finding,doctor-summary`.
