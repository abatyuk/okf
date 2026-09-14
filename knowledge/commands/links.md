---
type: Command
title: okf links
description: Lists the normalized direct concept links defined by one concept and reports whether each target exists.
group: query
mutates: false
implemented_by:
- /components/graph
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
- resource: crates/okf-cli/src/commands/query.rs
  kind: git-path
  fingerprint:
    blob_sha: 079709086641c8a57a21c3da0c2f2e027af390f3
- resource: crates/okf-core/src/model/link.rs
  kind: git-path
  fingerprint:
    blob_sha: d598a6692867ec1873c94c35e1b7e59a79ee5a2a
last_modified: 2026-09-11T21:59:16Z
---
# okf links

Lists one concept's direct normalized outbound edges from ontology-declared frontmatter
references, internal Markdown `sources[].resource` values, and Markdown body links. External and
self links are excluded, duplicate targets are collapsed, and missing concept targets remain
visible. Use [backlinks](backlinks.md) for the inverse one-hop query or a bounded
[graph](graph.md) for multi-hop context.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `link`.
