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

NDJSON record types and their serializer. The Concept record wraps Frontmatter verbatim plus computed id and trust_tier; Finding, Affected, and Change records back the other machine outputs.

Location: `crates/okf-core/src/output`.
