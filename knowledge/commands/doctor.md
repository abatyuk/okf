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
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--target <value>` | string | no | Target OKF version (default: `0.2`) |
| `--fix-safe` | bool | no | Enable the allow-listed safe repair set (default: `false`) |
| `--dry-run` | bool | no | Show safe repairs without writing (the default without --yes) (default: `false`) |
| `--yes` | bool | no | Confirm applying --fix-safe changes non-interactively (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `doctor-finding,doctor-summary`.
