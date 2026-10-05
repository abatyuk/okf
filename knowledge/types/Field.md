---
type: DomainType
title: Field
description: 'A typed field on a concept type: a name, a FieldType, and a required flag.'
module: ontology
defined_in:
- /components/ontology
sources:
- resource: crates/okf-core/src/ontology/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: 0b71d797088eff17079d10e830f2150a820f3bc5
last_modified: 2026-10-04T20:39:29Z
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

Recursive checking resolves named types for object children and list items. Type checks do not
coerce YAML values. Inclusive `min`/`max`, regex `pattern`, `unique_items`, and explicit
`additional_properties: false` constrain custom values. The root metadata stays open. Missing
optional parents are skipped; required fields reject missing/null, blank strings, and empty lists.
Unknown constraints are preserved and reported as unenforced rather than silently checked.
