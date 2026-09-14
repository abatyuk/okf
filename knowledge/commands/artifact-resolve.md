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
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--from <value>` | string | no | Resolve a relative resource against this declaring concept id |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `artifact-resolution`.
