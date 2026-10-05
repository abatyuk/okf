---
type: Component
title: output module
description: NDJSON record types and their serializer.
layer: core
depends_on:
- /components/model
sources:
- resource: crates/okf-core/src/output/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 85506dc0f17397f6d7e3841c47eb10ec16f92561
last_modified: 2026-10-04T20:39:29Z
---
# output module

NDJSON record types and their serializer. [Concept records](../types/ConceptRecord.md) preserve frontmatter values and add collision-safe identity, trust, lifecycle, generation, and verification views; [Finding](../types/Finding.md), [Change](../types/Change.md), and [Affected](../types/Affected.md) records back other machine outputs.

Location: `crates/okf-core/src/output`.
