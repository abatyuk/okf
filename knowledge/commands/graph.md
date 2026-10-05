---
type: Command
title: okf graph
description: Renders the entire link graph or a direction- and depth-bounded neighborhood as mermaid, dot, or graphml.
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
- resource: crates/okf-core/src/graph/render.rs
  kind: git-path
  fingerprint:
    blob_sha: d983856c864548bae7ca134b4300434c7d584618
last_modified: 2026-10-04T20:39:29Z
---
# okf graph

Renders the entire link graph or a rooted neighborhood as mermaid, dot, or graphml. Rooted views
can follow incoming, outgoing, or both edge directions and stop at a maximum neighbor distance.
Use [links](links.md) when only one concept's direct outbound targets are needed.
With `--json`, the selected graph artifact is returned as one `graph` record containing its
format, root, direction, depth, and content.

Cross-bundle nodes retain bundle identity, concept ID and version evidence. Registration alone
does not widen examination scope. Edges preserve original references, unresolved statuses and
optional provenance snapshot evidence; current candidates are separate from matched requests.
Working-tree versions remain mutable rather than being labeled immutable commits.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--root <value>` | string | no | Optional concept at the neighborhood root; without one, render the entire graph |
| `--format <value>` | string | no | Output format: mermaid (default), dot, or graphml (default: `mermaid`) |
| `--direction <value>` | string | no | Edges to follow from the root: outgoing (default), incoming, or both (default: `outgoing`) |
| `--depth <value>` | int | no | Maximum neighbor distance from the root (0 = root only; default: unbounded) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `graph,scope,bundle-node,bundle-edge`.
