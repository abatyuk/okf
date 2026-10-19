---
type: Command
group: query
mutates: false
implemented_by:
- /components/graph
title: okf links
description: Lists the normalized direct concept links defined by one concept and reports whether each target exists.
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: f78376fda84122c6210c1b68638c32cccc4f5c12
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 168b9a7f079813a55a9a99905970dfc182a91d67
- resource: crates/okf-cli/src/commands/query.rs
  kind: git-path
  fingerprint:
    blob_sha: 90e117f1601c95a58053c0f2a29355e740ba65f4
- resource: crates/okf-core/src/model/link.rs
  kind: git-path
  fingerprint:
    blob_sha: 26329c9080146c41e206bfacbb7e6dfb93bcedde
last_modified: 2026-10-19T03:58:59Z
---
# okf links

Lists one concept's direct normalized outbound edges from ontology-declared frontmatter
references, internal Markdown `sources[].resource` values, and Markdown body links. External and
self links are excluded, duplicate targets are collapsed, and missing concept targets remain
visible. Use [backlinks](backlinks.md) for the inverse one-hop query or a bounded
[graph](graph.md) for multi-hop context.

Use `okf links <concept> <bundle> --details` for authored semantic occurrences, including
multiple relationships to the same target. Each carries its kind, configured inverse label,
concrete field path, attributes, and status. Semantic self-edges and external or unknown-kind
occurrences remain inspectable; the compact adjacency rules above apply to ordinary links.
Inverse labels describe the incoming view and do not add outgoing edges.

Local `--details --json` emits `relationship` records with `schema_version: 1`,
`direction: outgoing`, bundle/version identity and an `edge` payload. Remaining ordinary targets
use `link` records with `source`, `target`, and `exists`; they have no invented semantic kind.
Catalog queries retain `scope` and `bundle-edge` records, with semantic metadata nested under
`relationship`. Detailed catalog text displays kinds, inverse labels, paths and attributes.

Declared nested reference selectors extract individual metadata occurrences. Semantic rules pair
targets with kinds and attributes in the same list record; opaque path-like strings do not create
edges. Catalog graph inspection retains qualified identity and resource/snapshot evidence.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--details` | bool | no | Show individual semantic relationship occurrences and configured inverse labels (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `link,relationship,scope,bundle-edge`.
