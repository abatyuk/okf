---
type: DomainType
title: ConceptType
description: 'An ontology entry: a named type with its typed fields and reference rules.'
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
# ConceptType

An ontology entry: a named type with typed [Field](Field.md) values and [ReferenceRule](ReferenceRule.md) edges. The key is the OKF type string verbatim, so types with spaces are quoted.

## Schema

```rust
pub struct ConceptType {
    pub description: Option<String>,
    pub requires: Vec<String>,                 // OKF built-in fields required (advisory)
    pub fields: IndexMap<String, Field>,       // typed custom fields
    pub attested: bool,                        // scaffold as Attested Computation
    pub trust: Option<TrustExpectation>,       // advisory min tier
    pub references: IndexMap<String, ReferenceRule>,
    pub relationships: IndexMap<String, RelationshipRule>,
    pub extra: IndexMap<String, serde_yaml::Value>,  // unknown keys preserved
}
```

Defined in `okf-core`.

Relationship rules refer to declared reference selectors, pair kinds/attributes with concrete
occurrences, and expose named inverse views. Concept-type ancestry is declared through the
preserved `extends` extension (string or list); it supports narrowing relationship targets
without changing exact standard computation semantics.
