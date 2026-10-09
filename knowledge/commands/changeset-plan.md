---
type: Command
title: okf changeset plan
description: Validate and preview coordinated single-bundle changes without publishing files.
group: query
mutates: false
implemented_by:
- /components/mutate
sources:
- resource: crates/okf-core/src/mutate/changeset.rs
  kind: git-path
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
---
# okf changeset plan

Validate and preview coordinated single-bundle changes without publishing files.

See the [complete CLI reference](../../docs/okf-cli-reference.md) for operation schemas and guarantees.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory |
| `--from <value>` | string | yes | Version 1 change document: inline YAML/JSON, @file, or stdin (-) |
| `--expect <value>` | string | no | Require this base digest from a previous plan before applying |
| `--dry-run` | bool | no | Validate and preview without writing (also the behavior of changeset plan) (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `changeset-file,changeset-summary`.
