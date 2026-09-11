---
type: Component
title: graph module
description: Forward and reverse adjacency built from concept links.
layer: core
depends_on:
- /components/bundle
- /components/model
- /components/ontology
sources:
- resource: crates/okf-core/src/graph/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: b42317411ca9f3445deb405efc6707f208623d4b
- resource: crates/okf-core/src/graph/render.rs
  kind: git-path
  fingerprint:
    blob_sha: fc4228d7b0dfeeaa202c0a7f61c20634e353f5ce
last_modified: 2026-09-07T18:47:09Z
---
# graph module

Forward and reverse adjacency built from the loaded [bundle](bundle.md), [model](model.md) links,
[ontology](ontology.md) references, and standard internal Markdown `sources[].resource` lineage.
Broken targets remain edges. Powers direct [links](../commands/links.md),
[backlinks](../commands/backlinks.md), affected analysis, move/remove safety, and whole or
direction- and depth-bounded [graph](../commands/graph.md) rendering.

Location: `crates/okf-core/src/graph`.
