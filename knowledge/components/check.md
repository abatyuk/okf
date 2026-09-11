---
type: Component
title: check module
description: Conformance, optional-family, lifecycle, drift, and compatibility diagnostics.
layer: core
depends_on:
- /components/bundle
- /components/model
- /components/fingerprint
- /components/ontology
- /components/graph
- /components/parse
- /components/ports
- /components/query
last_modified: 2026-09-09T13:17:50Z
sources:
- resource: crates/okf-core/src/check/validate.rs
  kind: git-path
  fingerprint:
    blob_sha: ea7b75b29440a8ed6d6d6b69ed2c889b3eb2b555
---
# check module

Diagnostics. Validate enforces all three conformance rules including index/log structure and root version syntax; lint checks optional families advisorily; stale compares corrected instants and fingerprint extensions; doctor performs tolerant upgrade preflight and safe repair. The implementation composes [bundle loading](bundle.md), the [model](model.md), [fingerprinting](fingerprint.md), the [ontology](ontology.md), the [graph](graph.md), [parsing](parse.md), effect [ports](ports.md), and artifact [queries](query.md).

Location: `crates/okf-core/src/check`.
