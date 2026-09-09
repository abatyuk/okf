---
type: Component
title: query module
description: Read-only lookups, including specification-native index.md browsing and targeted concept queries.
layer: core
depends_on:
- /components/model
- /components/graph
- /components/bundle
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

Read-only lookups: browse reads or synthesizes structural directory indexes; search and list locate concepts; show progressively reveals outlines and line ranges; resolve maps links and structural resources to paths; stats and diff summarize bundle state.

Location: `crates/okf-core/src/query`.
