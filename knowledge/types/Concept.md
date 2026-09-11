---
type: DomainType
title: Concept
description: 'A single knowledge unit: its ConceptId, parsed Frontmatter, and markdown body.'
module: model
defined_in:
- /components/model
sources:
- resource: ../crates/okf-core/src/model/concept.rs
  kind: git-path
  fingerprint:
    blob_sha: 5e0498b30ce9721871a1e668c0bbf2f5de0080af
last_modified: 2026-09-07T18:22:07Z
---
# Concept

A single knowledge unit: its [ConceptId](ConceptId.md), parsed [Frontmatter](Frontmatter.md), and markdown body. The unit the whole tool loads, queries, and mutates.

## Schema

```rust
pub struct Concept {
    pub id: ConceptId,
    pub frontmatter: Frontmatter,
    pub body: String,
}
```

Defined in `okf-core`.
