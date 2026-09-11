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
last_modified: 2026-09-09T13:17:50Z
sources:
- resource: crates/okf-core/src/query/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 24f3127b319aaed41f39a3d342ffdb7dd0e62665
- resource: crates/okf-core/src/query/browse.rs
  kind: git-path
  fingerprint:
    blob_sha: f908123790031cf642ee8aaf0219d51e7e763dc3
---
# query module

Read-only lookups: browse reads or synthesizes structural directory indexes through [render](render.md); search/list/show retrieve concepts from a [bundle](bundle.md); artifact commands safely resolve path-valued resources; computation check inspects contracts without execution; and stats/diff compose [check](check.md), [graph](graph.md), [model](model.md), [ontology](ontology.md), [parse](parse.md), and effect [port](ports.md) services.

Location: `crates/okf-core/src/query`.
