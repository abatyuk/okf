---
type: DomainType
title: ConceptType
description: 'An ontology entry: a named type with its typed fields and reference rules.'
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
# ConceptType

An ontology entry: a named type with its typed fields and reference rules. The key is the OKF type string verbatim, so types with spaces are quoted.

## Schema

```rust
pub struct ConceptType {
    pub description: Option<String>,
    pub requires: Vec<String>,                 // OKF built-in fields required (advisory)
    pub fields: IndexMap<String, Field>,       // typed custom fields
    pub attested: bool,                        // scaffold as Attested Computation
    pub trust: Option<TrustExpectation>,       // advisory min tier
    pub references: IndexMap<String, ReferenceRule>,
    pub extra: IndexMap<String, serde_yaml::Value>,  // unknown keys preserved
}
```

Defined in `okf-core`.
