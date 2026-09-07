---
type: DomainType
title: Frontmatter
description: The order-preserving, unknown-key-preserving YAML mapping at the top of a concept.
module: model
defined_in:
- /components/model
sources:
- resource: ../crates/okf-core/src/model/frontmatter.rs
  kind: git-path
  fingerprint:
    blob_sha: adf7ff4edacef5d7b0a10e8bb8eadadafb85debb
last_modified: 2026-09-07T18:22:07Z
---
# Frontmatter

The order-preserving, unknown-key-preserving YAML mapping at the top of a concept. The linchpin type: everything round-trips through it, and the NDJSON concept record is this map serialized verbatim plus computed id and trust_tier.

## Schema

```rust
pub struct Frontmatter {
    /// Insertion-ordered key/value pairs, preserving on-disk order and unknown keys verbatim.
    pub map: IndexMap<String, serde_yaml::Value>,
}
```

Defined in `okf-core`.
