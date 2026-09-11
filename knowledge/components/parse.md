---
type: Component
title: parse module
description: 'Semantic markdown and YAML handling that preserves unknown keys, values, order, and body text while YAML presentation may normalize.'
layer: core
depends_on:
- /components/model
sources:
- resource: ../crates/okf-core/src/parse/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: da4e512dba289d2a1d1ff50a314d5d068a8e5905
last_modified: 2026-09-07T18:47:09Z
---
# parse module

Semantic markdown and YAML handling: split frontmatter from body, build the heading tree, extract sections, retain unknown keys/values and top-level order, and serialize back. Body text is preserved; YAML comments and presentation may normalize.

Location: `crates/okf-core/src/parse`.
