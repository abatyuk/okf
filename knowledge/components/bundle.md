---
type: Component
title: bundle module
description: Bundle loading and location.
layer: core
depends_on:
- /components/parse
- /components/model
sources:
- resource: ../crates/okf-core/src/bundle/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: c14580d8574e78b2a9d3b6634abadace8de5a9d0
last_modified: 2026-09-07T18:47:09Z
---
# bundle module

Bundle loading and location. Walks a directory into a set of Concepts (honoring reserved filenames like index.md and log.md) and resolves which bundle to operate on via the arg then config then env then cwd chain.

Location: `crates/okf-core/src/bundle`.
