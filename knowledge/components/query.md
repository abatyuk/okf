---
type: Component
title: query module
description: Read-only concept queries, corrected path resolution, and bounded artifact/computation inspection.
layer: core
depends_on:
- /components/bundle
- /components/check
- /components/graph
- /components/model
- /components/ontology
- /components/parse
- /components/ports
- /components/render
last_modified: 2026-10-04T20:39:29Z
sources:
- resource: crates/okf-core/src/query/metadata.rs
  kind: git-path
  fingerprint:
    blob_sha: 3f4c8c00963a000231817eed107de3e440965fce
- resource: crates/okf-core/src/bundle/settings.rs
  kind: git-path
  fingerprint:
    blob_sha: b05d13b997af6da4c50cacc8c6018a1d92d9e32c
- resource: crates/okf-core/src/query/selector.rs
  kind: git-path
  fingerprint:
    blob_sha: 1d701ba77f220d734005c8f09f4f0cf255fbdfe0
- resource: crates/okf-core/src/query/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: f24b1a81d701737379ccc88a336d3a11176490f1
- resource: crates/okf-core/src/query/browse.rs
  kind: git-path
  fingerprint:
    blob_sha: af32a174d57deb7f6ae9d3423b59d3986845dc64
---
# query module

Read-only lookups: browse reads or synthesizes structural directory indexes through [render](render.md); search/list/show retrieve concepts from a [bundle](bundle.md); artifact commands safely resolve path-valued resources; computation check inspects contracts without execution; and stats/diff compose [check](check.md), [graph](graph.md), [model](model.md), [ontology](ontology.md), [parse](parse.md), and effect [port](ports.md) services.

Location: `crates/okf-core/src/query`.

Structured queries share explicit YAML selectors with graph rules: nested properties, `[]` list
traversal, and bracket-quoted literal keys. The walker retains occurrence paths. Typed conditions,
views, projections, JSON facets, pagination and bounded related expansion compute reader output
without changing authored frontmatter. Results retain scan completeness and examination scope;
facets concern examined matches before the displayed page.

Parse failures identify the actual files examined by the query. Bounded scans collect failures within the scan budget; historical reads include revision context and distinguish malformed content from unavailable Git material.
