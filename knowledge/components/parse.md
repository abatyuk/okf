---
type: Component
title: parse module
description: 'Lossless markdown and YAML handling: split frontmatter from body, build the heading tree, extract sections, load YAML order-preservingly, and serialize back with a round-trip guarantee.'
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

Lossless markdown and YAML handling: split frontmatter from body, build the heading tree, extract sections, load YAML order-preservingly, and serialize back with a round-trip guarantee.

Location: `crates/okf-core/src/parse`.
