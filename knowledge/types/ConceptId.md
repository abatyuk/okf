---
type: DomainType
title: ConceptId
description: A concept's identity, equal to its bundle-relative path without the .md suffix (for example /metrics/revenue).
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
# ConceptId

A concept's identity, equal to its bundle-relative path without the .md suffix (for example /metrics/revenue). Because the id is the path, moves must rewrite inbound links.

## Schema

```rust
pub struct ConceptId(pub String);  // == bundle-relative path without `.md`
```

Defined in `okf-core`.
