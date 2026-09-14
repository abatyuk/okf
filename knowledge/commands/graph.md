---
type: Command
title: okf graph
description: Renders the entire link graph or a direction- and depth-bounded neighborhood as mermaid, dot, or graphml.
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
- resource: crates/okf-core/src/graph/render.rs
  kind: git-path
  fingerprint:
    blob_sha: d983856c864548bae7ca134b4300434c7d584618
last_modified: 2026-09-07T18:22:06Z
---
# okf graph

Renders the entire link graph or a rooted neighborhood as mermaid, dot, or graphml. Rooted views
can follow incoming, outgoing, or both edge directions and stop at a maximum neighbor distance.
Use [links](links.md) when only one concept's direct outbound targets are needed.
With `--json`, the selected graph artifact is returned as one `graph` record containing its
format, root, direction, depth, and content.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--root <value>` | string | no | Optional concept at the neighborhood root; without one, render the entire graph |
| `--format <value>` | string | no | Output format: mermaid (default), dot, or graphml (default: `mermaid`) |
| `--direction <value>` | string | no | Edges to follow from the root: outgoing (default), incoming, or both (default: `outgoing`) |
| `--depth <value>` | int | no | Maximum neighbor distance from the root (0 = root only; default: unbounded) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `graph`.
