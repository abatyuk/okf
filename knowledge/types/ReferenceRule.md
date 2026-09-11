---
type: DomainType
title: ReferenceRule
description: 'A typed edge rule: a frontmatter key on the source concept, the allowed target type(s), and a cardinality.'
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
# ReferenceRule

A typed edge rule: a frontmatter key on the source concept, the allowed target type(s), and a [Cardinality](Cardinality.md). [okf lint](../commands/lint.md) enforces it advisorily.

## Schema

```rust
pub struct ReferenceRule {
    pub target: Target,            // one type or a union of types
    pub cardinality: Cardinality,
    pub extra: IndexMap<String, serde_yaml::Value>,
}
```

Defined in `okf-core`.
