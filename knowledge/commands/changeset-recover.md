---
type: Command
title: okf changeset recover
description: Recover interrupted publication without overwriting intervening edits.
group: mutate
mutates: true
implemented_by:
- /components/mutate
sources:
- resource: crates/okf-core/src/mutate/transaction.rs
  kind: git-path
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
---
# okf changeset recover

Recover interrupted publication without overwriting intervening edits.

See the [complete CLI reference](../../docs/okf-cli-reference.md) for operation schemas and guarantees.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `changeset-recovery`.
