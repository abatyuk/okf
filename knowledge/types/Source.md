---
type: DomainType
title: Source
description: 'A provenance entry in sources[]: the OKF fields plus a typed kind and a recorded fingerprint, so stale has something to compare against.'
module: model
defined_in:
- /components/model
sources:
- resource: crates/okf-core/src/model/source.rs
  kind: git-path
  fingerprint:
    blob_sha: ae3eabcbf52969391c8eded9a73b9e868720d1cd
last_modified: 2026-10-04T20:39:29Z
---
# Source

A provenance entry in `sources[]`: the OKF fields plus a [SourceKind](SourceKind.md) and a recorded [Fingerprint](Fingerprint.md), so stale has something to compare against.

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

Standard provenance requires `resource`; fingerprint kind/baseline extensions remain optional.
`bundle_ref` is a producer-defined extension retained in `extra` and interpreted by catalog
resolution without changing ordinary resource semantics. Snapshot evidence and source fingerprint
drift are separate; refreshing a reviewed baseline does not accept a current snapshot candidate
or append a verification event.
