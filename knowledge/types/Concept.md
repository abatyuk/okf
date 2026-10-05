---
type: DomainType
title: Concept
description: 'A single knowledge unit: its ConceptId, parsed Frontmatter, and markdown body.'
module: model
defined_in:
- /components/model
sources:
- resource: crates/okf-core/src/model/concept.rs
  kind: git-path
  fingerprint:
    blob_sha: 411ef200d561435377d75be8845962f577cffdf4
last_modified: 2026-10-04T20:39:29Z
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
