---
type: Component
title: render module
description: Derived output artifacts, including deterministic specification-conformant index.md generation.
layer: core
depends_on:
- /components/bundle
- /components/check
- /components/model
- /components/graph
- /components/ontology
- /components/parse
last_modified: 2026-10-04T20:39:29Z
sources:
- resource: crates/okf-core/src/render/index.rs
  kind: git-path
  fingerprint:
    blob_sha: 053f4712cbc9abd787902f9ba291bf1d07507889
---
# render module

Derived output artifacts: progressive-disclosure index.md generation and the optional html/md/pdf/graphml/obsidian document renderers behind feature flags. Rendering composes [bundle](bundle.md), [check](check.md), [model](model.md), [graph](graph.md), [ontology](ontology.md), and [parse](parse.md) services.

Location: `crates/okf-core/src/render`.
