---
type: Component
title: mutate module
description: 'Atomic, self-validating concept writes that preserve unknown YAML meaning and body text while applying bundle mutations.'
layer: core
depends_on:
- /components/model
- /components/ontology
- /components/graph
- /components/parse
sources:
- resource: ../crates/okf-core/src/mutate/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: c425ec76e82eeef10f11db6e9e6f393d31a5c9f2
last_modified: 2026-09-07T18:47:09Z
---
# mutate module

Atomic, self-validating concept writes preserve unknown YAML meaning and body text while applying init, add, edit, mv, rm, verify, and refresh. YAML comments and scalar presentation may normalize.

Location: `crates/okf-core/src/mutate`.
