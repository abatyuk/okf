---
type: Component
title: output module
description: NDJSON record types and their serializer.
layer: core
depends_on:
- /components/model
sources:
- resource: ../crates/okf-core/src/output/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: c9dadff8f9444ccfcbec587eba7767b6020e778e
last_modified: 2026-09-07T18:47:09Z
---
# output module

NDJSON record types and their serializer. Concept records preserve frontmatter values and add collision-safe identity, trust, lifecycle, generation, and verification views; findings and change records back other machine outputs.

Location: `crates/okf-core/src/output`.
