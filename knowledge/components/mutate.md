---
type: Component
title: mutate module
description: 'Lossless, self-validating writes to the bundle: init, add (scaffold from ontology), edit, mv (rename plus rewrite of every inbound link), rm, and the verify/refresh trust and fingerprint writers.'
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

Lossless, self-validating writes to the bundle: init, add (scaffold from ontology), edit, mv (rename plus rewrite of every inbound link), rm, and the verify/refresh trust and fingerprint writers.

Location: `crates/okf-core/src/mutate`.
