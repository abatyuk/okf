---
type: Command
title: okf links
description: Lists the normalized direct concept links defined by one concept and reports whether each target exists.
group: query
mutates: false
implemented_by:
- /components/graph
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: 126dddfab3fb8caa78cfd61396db89dbcc324a33
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 28db6285f18a0400ab4b08c79c46322e58c2d833
- resource: crates/okf-cli/src/commands/query.rs
  kind: git-path
  fingerprint:
    blob_sha: c9b643257e90f1e3e225cb4613be98f08f6abd02
- resource: crates/okf-core/src/model/link.rs
  kind: git-path
  fingerprint:
    blob_sha: 26329c9080146c41e206bfacbb7e6dfb93bcedde
last_modified: 2026-10-04T20:39:29Z
---
# okf links

Lists one concept's direct normalized outbound edges from ontology-declared frontmatter
references, internal Markdown `sources[].resource` values, and Markdown body links. External and
self links are excluded, duplicate targets are collapsed, and missing concept targets remain
visible. Use [backlinks](backlinks.md) for the inverse one-hop query or a bounded
[graph](graph.md) for multi-hop context.

Declared nested reference selectors extract individual metadata occurrences. Semantic rules pair
targets with kinds and attributes in the same list record; opaque path-like strings do not create
edges. Catalog graph inspection retains qualified identity and resource/snapshot evidence.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--details` | bool | no | Compatibility option; detailed output is not implemented for this command (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `link,relationship,scope,bundle-edge`.
