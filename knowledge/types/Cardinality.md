---
type: DomainType
title: Cardinality
description: 'How many targets a reference (or list field) may have: 0..1, 1..1, 0..n, or 1..n.'
module: ontology
defined_in:
- /components/ontology
sources:
- resource: ../crates/okf-core/src/ontology/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: 878335dafa46f950612887e5a0aad823f8a573a3
last_modified: 2026-09-07T18:22:07Z
---
# Cardinality

How many targets a [ReferenceRule](ReferenceRule.md) or list [Field](Field.md) may have: `0..1`, `1..1`, `0..n`, or `1..n`.

## Schema

```rust
pub enum Cardinality {
    ZeroOne,  // 0..1  optional single
    OneOne,   // 1..1  required single
    ZeroN,    // 0..n  optional many
    OneN,     // 1..n  at least one
}
```

Defined in `okf-core`.
