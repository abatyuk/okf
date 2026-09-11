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

The order-preserving, unknown-key/value-preserving YAML mapping at the top of a [Concept](Concept.md). It round-trips semantic values and top-level order; YAML comments and presentation may normalize. A [Concept record](ConceptRecord.md) adds collision-safe computed lifecycle, generation, verification, identity, and trust views.

## Schema

```rust
pub struct Frontmatter {
    /// Insertion-ordered key/value pairs, preserving semantic values and top-level key order.
    pub map: IndexMap<String, serde_yaml::Value>,
}
```

Defined in `okf-core`.
