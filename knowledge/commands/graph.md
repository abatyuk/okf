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
    blob_sha: 7a22138be071fc20ede774d812f9f98d018e7341
- resource: crates/okf-cli/src/commands/query.rs
  kind: git-path
  fingerprint:
    blob_sha: 98fcf5b787e69c801489823ffea093f747e64d22
- resource: crates/okf-core/src/graph/render.rs
  kind: git-path
  fingerprint:
    blob_sha: fc4228d7b0dfeeaa202c0a7f61c20634e353f5ce
last_modified: 2026-09-07T18:22:06Z
---
# okf graph

Renders the entire link graph or a rooted neighborhood as mermaid, dot, or graphml. Rooted views
can follow incoming, outgoing, or both edge directions and stop at a maximum neighbor distance.
Use [links](links.md) when only one concept's direct outbound targets are needed.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `<concept>` | positional | no | Optional concept at the neighborhood root; without one, render the entire graph |
| `--format <value>` | string | no | Output format: mermaid (default), dot, or graphml (default: `mermaid`) |
| `--direction <value>` | string | no | Edges to follow from the root: outgoing (default), incoming, or both |
| `--depth <value>` | string | no | Maximum neighbor distance from the root (0 = root only; default: unbounded) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `graph`.
