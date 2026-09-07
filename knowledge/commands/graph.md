---
type: Command
title: okf graph
description: Renders the link graph, or a subtree rooted at a concept, as mermaid, dot, or graphml.
group: query
mutates: false
implemented_by:
- /components/graph
sources:
- resource: ../crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: a1bdcb78fa383c159cc63689e31430beb78d30b4
last_modified: 2026-09-07T18:22:06Z
---
# okf graph

Renders the link graph, or a subtree rooted at a concept, as mermaid, dot, or graphml.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `<subtree>` | positional | no | Optional subtree root: only the subgraph forward-reachable from this concept |
| `--format <value>` | string | no | Output format: mermaid (default), dot, or graphml (default: `mermaid`) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `graph`.
