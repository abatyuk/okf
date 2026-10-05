---
type: DesignDecision
title: Explicit structured mutation inputs
description: YAML assignments, object paths and concrete patches extend authoring while preserving literal scalar flags.
decision_status: accepted
affects:
- /components/mutate
- /components/ontology
- /components/okf-cli
sources:
- resource: crates/okf-cli/src/structured.rs
  kind: git-path
  fingerprint:
    blob_sha: 889819283f7bd1c986e0ce78deb39f77fd481633
- resource: crates/okf-core/src/mutate/structured.rs
  kind: git-path
  fingerprint:
    blob_sha: 6c504c52f0f9f23b83fde8b312aae82dafcc8c78
- resource: crates/okf-core/src/ontology/edit.rs
  kind: git-path
  fingerprint:
    blob_sha: 885cd3dedad75883309028b70b863cc31931c19b
- resource: plugins/okf/skills/ontology/references/structured-authoring.md
  kind: git-path
  fingerprint:
    blob_sha: a05ec02fcc363f85ee522cef43b0662a0d05b256
last_modified: 2026-10-05T10:16:06Z
---
# Explicit structured mutation inputs

Existing scalar flags retain literal keys. YAML values (including JSON syntax) make objects,
lists and explicit scalar types authorable without another shorthand grammar. Object-path writes
handle common nested map changes and create missing maps; they reject lists and scalar traversal.
Concrete JSON Pointer patches supply list addressing and guarded RFC 6902 changes. Query selectors
remain separate because wildcard selection does not identify a single mutation address.

Named ontology declarations replace completely; omitted declarations remain on concept update.
Reusable field types replace whole definitions. Dependent bulk changes and explicit removals
validate the final ontology before writing once, avoiding order-dependent intermediate schemas.

Strict input parsing rejects ambiguous duplicate/non-string keys, multiple documents, tags,
aliases and merge keys. Only one input may consume stdin. Dry-run uses the same preflight without
writes. Presentation can normalize; preserved meaning is distinct from exact byte preservation.
Concept lifecycle consequences remain those of meaningful edits; schema validity is advisory and
neither establishes verification nor proves relationship acceptance.
