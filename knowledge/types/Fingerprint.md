---
type: DomainType
title: Fingerprint
description: A kind-specific record of a source's state at last sync (a blob id, a content hash, an etag).
module: model
defined_in:
- /components/model
sources:
- resource: ../crates/okf-core/src/model/source.rs
  kind: git-path
  fingerprint:
    blob_sha: ae3eabcbf52969391c8eded9a73b9e868720d1cd
last_modified: 2026-09-07T18:22:07Z
---
# Fingerprint

A kind-specific record of a [Source](Source.md)'s state at last sync (a blob id, a content hash, an etag). Drift is a mismatch between the recorded and recomputed fingerprint.

## Schema

```rust
pub struct Fingerprint {
    pub fields: Vec<(String, String)>,  // kind-specific, e.g. [("content_sha256", "…")]
}
```

Defined in `okf-core`.
