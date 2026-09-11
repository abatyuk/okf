---
type: Component
title: model module
description: 'In-memory domain types with no I/O: Concept, ConceptId, the order-preserving Frontmatter, trust types, source types, and Link parsing.'
layer: core
depends_on:
- /components/ontology
sources:
- resource: ../crates/okf-core/src/model/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: af4e0dd88aa5f035d93f3a7c926622517f9b7e03
last_modified: 2026-09-07T18:47:09Z
---
# model module

In-memory domain types with no I/O: [Concept](../types/Concept.md), [ConceptId](../types/ConceptId.md), the order-preserving [Frontmatter](../types/Frontmatter.md), trust types ([Actor](../types/Actor.md), [Verified](../types/Verified.md), and [TrustTier](../types/TrustTier.md)), source types ([Source](../types/Source.md), [SourceKind](../types/SourceKind.md), and [Fingerprint](../types/Fingerprint.md)), and [Link](../types/Link.md) parsing. Link extraction consults the tool-local [ontology](ontology.md) for declared reference fields. The linchpin of the crate, because everything round-trips through Frontmatter.

Location: `crates/okf-core/src/model`.
