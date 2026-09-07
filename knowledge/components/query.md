---
type: Component
title: query module
description: 'Read-only lookups: search (list is search with no filter), show, resolve a link or id to a path, stats summary, and a concept-level diff against a git ref.'
layer: core
depends_on:
- /components/model
- /components/graph
- /components/bundle
sources:
- resource: ../crates/okf-core/src/query/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 2dd8114d6a52e8c0deea408a556477f87978c5c0
last_modified: 2026-09-07T18:47:09Z
---
# query module

Read-only lookups: search (list is search with no filter), show, resolve a link or id to a path, stats summary, and a concept-level diff against a git ref.

Location: `crates/okf-core/src/query`.
