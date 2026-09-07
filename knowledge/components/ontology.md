---
type: Component
title: ontology module
description: 'The tool-local ontology: concept types, typed fields with composition via extends, and typed reference rules with cardinality.'
layer: core
depends_on:
- /components/model
sources:
- resource: ../crates/okf-core/src/ontology/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 596999c77cec95028c36fa4bb93afec97ca46ce4
last_modified: 2026-09-07T18:47:09Z
---
# ontology module

The tool-local ontology: concept types, typed fields with composition via extends, and typed reference rules with cardinality. Loads ontology.yaml and applies add/update/remove edits that back the okf ontology commands.

Location: `crates/okf-core/src/ontology`.
