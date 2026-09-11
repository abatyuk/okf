---
type: DomainType
title: Link
description: A parsed reference to another concept, classified as bundle-relative, relative, bare, or external.
module: model
defined_in:
- /components/model
sources:
- resource: ../crates/okf-core/src/model/link.rs
  kind: git-path
  fingerprint:
    blob_sha: ca34554f8fcd422c30315e5650116de9776996a7
last_modified: 2026-09-07T18:22:07Z
---
# Link

A parsed reference to another concept, classified as bundle-relative, relative, bare, or external. Broken links are recorded, never rejected.

## Schema

```rust
// Links are parsed + classified (model/link.rs); there is no owning struct.
pub enum LinkKind {
    BundleRelative,  // leading `/`  — from the bundle root
    Relative,        // `./` or `../` — from the concept's directory
    Bare,            // no prefix     — relative to the concept directory
    External,        // URI scheme or `#fragment` — never a concept edge
}
// fn classify(&str) -> LinkKind; fn resolve_link(from, raw) -> ConceptId
```

Defined in `okf-core`.
