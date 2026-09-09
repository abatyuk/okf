---
type: Component
title: check module
description: Read-only diagnostics, including OKF reserved-file conformance.
layer: core
depends_on:
- /components/model
- /components/fingerprint
- /components/ontology
- /components/graph
last_modified: 2026-09-09T13:17:50Z
sources:
- resource: crates/okf-core/src/check/validate.rs
  kind: git-path
  fingerprint:
    blob_sha: ea7b75b29440a8ed6d6d6b69ed2c889b3eb2b555
---
# check module

Read-only diagnostics. Validate enforces the three conformance rules, including the root-only `okf_version` frontmatter exception for `index.md`; the lint engine emits Findings; stale detects drift between recorded and recomputed source fingerprints.

Location: `crates/okf-core/src/check`.
