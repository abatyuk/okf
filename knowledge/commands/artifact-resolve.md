---
type: Command
title: okf artifact resolve
description: Resolves an OKF path-valued resource with declaring-concept context and containment checks.
group: query
mutates: false
implemented_by:
- /components/query
---
# okf artifact resolve

Classifies a resource as a concept, reserved file, opaque artifact, external URL, scope descriptor, missing path, or blocked path. Relative resources resolve from their declaring concept.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<resource>` | positional | yes | Resource path, URL, or scope descriptor |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--from <value>` | string | no | Resolve a relative resource against this declaring concept id |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `artifact-resolution`.
