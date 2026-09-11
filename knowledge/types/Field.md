---
type: DomainType
title: Field
description: 'A typed field on a concept type: a name, a FieldType, and a required flag.'
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
# Field

A typed field on a concept type: a name, a [FieldType](FieldType.md), and a required flag.

## Schema

```rust
pub struct Field {
    pub type_name: String,              // serialized as `type`
    pub required: bool,                 // default false
    pub values: Option<Vec<String>>,    // for `enum`
    pub item: Option<String>,           // for `list`
    pub fields: Option<IndexMap<String, Field>>,  // for `object`
    pub constraints: IndexMap<String, serde_yaml::Value>,  // pattern/min/max/…
}
```

Defined in `okf-core`.
