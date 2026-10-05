---
type: Component
title: parse module
description: Semantic markdown and YAML handling that preserves unknown keys, values, order, and body text while YAML presentation may normalize.
layer: core
depends_on:
- /components/model
sources:
- resource: crates/okf-core/src/parse/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 9b13dea795e8f888aa416e99e9a1fa3e6d899916
last_modified: 2026-10-04T20:39:29Z
---
# parse module

Semantic markdown and YAML handling: split frontmatter from body, build the heading tree, extract sections, retain unknown keys/values and top-level order, and serialize back. Body text is preserved; YAML comments and presentation may normalize.

Location: `crates/okf-core/src/parse`.

`parse_concept_file` adds the document path to parser errors without changing their exit class; text-only `parse_concept` remains available for in-memory parsing.
