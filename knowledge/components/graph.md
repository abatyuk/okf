---
type: Component
title: graph module
description: Forward and reverse adjacency built from concept links.
layer: core
depends_on:
- /components/model
sources:
- resource: ../crates/okf-core/src/graph/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: fca013193fc85476bff06a1b39ba28176833690d
last_modified: 2026-09-07T18:47:09Z
---
# graph module

Forward and reverse adjacency built from concept links. Powers backlinks, the affected impact reverse walk, and the mermaid/dot/graphml renderers.

Location: `crates/okf-core/src/graph`.
