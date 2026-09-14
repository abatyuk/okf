---
type: Command
title: okf artifact list
description: Inventories concepts, reserved files, and opaque artifacts under a bundle directory such as references/.
group: query
mutates: false
implemented_by:
- /components/query
---
# okf artifact list

Lists bundle files with their artifact classification and size, optionally computing a SHA-256 digest. The `references/` directory is supported as an optional OKF convention, not required.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--directory <value>` | string | no | Bundle-relative directory to inventory (default: `references`) |
| `--digest` | bool | no | Compute SHA-256 digests (reads each file) (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `artifact`.
