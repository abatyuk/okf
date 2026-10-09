---
type: Command
title: okf ontology field-type list
description: List authored reusable field types and their effective definitions.
group: query
mutates: false
implemented_by:
- /components/ontology
sources:
- resource: crates/okf-cli/src/commands/ontology.rs
  kind: git-path
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
---
# okf ontology field-type list

List authored reusable field types and their effective definitions.

See the [complete CLI reference](../../docs/okf-cli-reference.md) for operation schemas and guarantees.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `ontology_field_type`.
