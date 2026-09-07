---
type: Component
title: check module
description: Read-only diagnostics.
layer: core
depends_on:
- /components/model
- /components/fingerprint
- /components/ontology
- /components/graph
sources:
- resource: ../crates/okf-core/src/check/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: a805160ea422a63816d5a0b9c50118584e0f9a5a
last_modified: 2026-09-07T18:47:09Z
---
# check module

Read-only diagnostics. validate enforces only the three conformance rules; the lint rule engine emits Findings at error/warn/info severities; stale detects drift between recorded and recomputed source fingerprints.

Location: `crates/okf-core/src/check`.
