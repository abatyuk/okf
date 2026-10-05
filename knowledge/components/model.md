---
type: Component
title: model module
description: 'In-memory domain types with no I/O: Concept, ConceptId, the order-preserving Frontmatter, trust types, source types, and Link parsing.'
layer: core
depends_on:
- /components/ontology
sources:
- resource: crates/okf-core/src/model/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: db5f3d22c9c74b2c999806321af0cc3d88929511
last_modified: 2026-10-04T20:39:29Z
---
# model module

In-memory domain types with no I/O: [Concept](../types/Concept.md), [ConceptId](../types/ConceptId.md), the order-preserving [Frontmatter](../types/Frontmatter.md), trust types ([Actor](../types/Actor.md), [Verified](../types/Verified.md), and [TrustTier](../types/TrustTier.md)), source types ([Source](../types/Source.md), [SourceKind](../types/SourceKind.md), and [Fingerprint](../types/Fingerprint.md)), and [Link](../types/Link.md) parsing. Link extraction consults the tool-local [ontology](ontology.md) for declared reference fields. The linchpin of the crate, because everything round-trips through Frontmatter.

Location: `crates/okf-core/src/model`.
