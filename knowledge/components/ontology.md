---
type: Component
title: ontology module
description: 'The tool-local ontology: concept types, typed fields with composition via extends, and typed reference rules with cardinality.'
layer: core
depends_on:
- /components/model
sources:
- resource: crates/okf-core/src/ontology/field_types.rs
  kind: git-path
  fingerprint:
    blob_sha: defb25349e8b5d1eac2730f733bba3ea81245e09
- resource: crates/okf-core/src/ontology/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: 0b71d797088eff17079d10e830f2150a820f3bc5
- resource: crates/okf-core/src/ontology/load.rs
  kind: git-path
  fingerprint:
    blob_sha: 92e9f44078458a1f28e05f6cc629f32071bdceef
- resource: crates/okf-core/src/ontology/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 8d5e8a205a20544e87578037b2da1514ced03dca
- resource: crates/okf-core/src/ontology/edit.rs
  kind: git-path
  fingerprint:
    blob_sha: 885cd3dedad75883309028b70b863cc31931c19b
- resource: crates/okf-cli/src/structured.rs
  kind: git-path
  fingerprint:
    blob_sha: 889819283f7bd1c986e0ce78deb39f77fd481633
last_modified: 2026-10-05T10:16:06Z
---
# ontology module

Optional tool-local definitions: exact concept types, reusable nested field types, recursive
constraints, declared reference selectors and semantic relationship rules. Composition resolves
named references at every nested level; inheritance retains most-derived constraints and explicit
fields, values or item settings replace inherited counterparts. Invalid schemas and cycles are
configuration errors. Unknown extensions remain preserved and unenforced constraints remain visible.

Loads ontology YAML and supplies effective definition inspection. Scalar and structured add/update/remove writes are available. YAML declaration inputs replace
named entries, reusable field-type commands replace full definitions, and bulk apply merges
changes and explicit removals before validating and writing the combined result once. [Lint](../commands/lint.md) checks
declarations advisorily, without changing OKF conformance or proving relationship truth.

Location: `crates/okf-core/src/ontology`.

Concept-type ancestry supports relationship target restrictions. A dynamic kind may narrow its
reference rule's allowed target union; this local ancestry does not grant standard computation
semantics to arbitrary types. Each bundle owns those definitions.

Ontology parse and schema errors identify the effective ontology file, including configured sidecars and historical interpretation files.

Structured declaration edits preserve unrelated ontology extensions. Concept updates merge by
named declaration; reusable field-type updates replace complete definitions. Bulk change documents
contain `field_types`, `concepts`, and explicit `remove` maps rather than a complete sidecar.
Conflicting operations fail, and dry-run previews the same resulting ontology without writing.
