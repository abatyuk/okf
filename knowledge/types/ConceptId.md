---
type: DomainType
title: ConceptId
description: A concept's identity, equal to its bundle-relative path without the .md suffix (for example /metrics/revenue).
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
# ConceptId

A [Concept](Concept.md)'s identity, equal to its bundle-relative path without the `.md` suffix (for example `/metrics/revenue`). Because the id is the path, [okf mv](../commands/mv.md) must rewrite inbound [Link](Link.md) values.

## Schema

```rust
pub struct ConceptId(pub String);  // == bundle-relative path without `.md`
```

Defined in `okf-core`.
