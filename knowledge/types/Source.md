---
type: DomainType
title: Source
description: 'A provenance entry in sources[]: the OKF fields plus a typed kind and a recorded fingerprint, so stale has something to compare against.'
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
# Source

A provenance entry in sources[]: the OKF fields plus a typed kind and a recorded fingerprint, so stale has something to compare against.

## Schema

```rust
pub struct Source {
    pub resource: String,
    pub kind: SourceKind,
    pub fingerprint: Fingerprint,
    /// Other spec/unknown keys on the entry, in original order.
    pub extra: IndexMap<String, serde_yaml::Value>,
}
```

Defined in `okf-core`.
