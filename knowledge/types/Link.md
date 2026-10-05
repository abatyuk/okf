---
type: DomainType
title: Link
description: A parsed reference to another concept, classified as bundle-relative, relative, bare, or external.
module: model
defined_in:
- /components/model
sources:
- resource: crates/okf-core/src/model/link.rs
  kind: git-path
  fingerprint:
    blob_sha: 26329c9080146c41e206bfacbb7e6dfb93bcedde
last_modified: 2026-10-04T20:39:29Z
---
# Link

A parsed reference from one [Concept](Concept.md) to another, resolved to a [ConceptId](ConceptId.md) and classified as bundle-relative, relative, bare, or external. Broken links are recorded, never rejected.

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

Declared ontology selectors can extract links from nested list/object metadata while retaining
concrete occurrence paths. Ordinary path-looking values remain opaque without declarations.
Catalog resolution qualifies registered cross-bundle targets, retaining original resources
and out-of-scope/missing statuses without matching names heuristically.

Repository-relative Git fingerprint sources retain their Git-root namespace; a Markdown artifact
resource is not reinterpreted as a document-relative concept ID. Standard kind-less Markdown
source lineage retains ordinary document-relative semantics. Catalog source resolution keeps
those namespaces explicit.
