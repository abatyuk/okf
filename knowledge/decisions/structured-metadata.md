---
type: DesignDecision
title: Declared structured metadata interpretation
description: Nested constraints and semantic references remain optional local interpretation rather than OKF conformance.
decision_status: accepted
affects:
- /components/ontology
- /components/graph
- /components/query
- /components/check
sources:
- resource: docs/structured-metadata-capability.md
  kind: git-path
  fingerprint:
    blob_sha: 50a614920c734d78ecb40fe8b2600a33d02a021f
- resource: crates/okf-core/src/query/selector.rs
  kind: git-path
  fingerprint:
    blob_sha: 1d701ba77f220d734005c8f09f4f0cf255fbdfe0
- resource: crates/okf-core/src/graph/relationships.rs
  kind: git-path
  fingerprint:
    blob_sha: cbd606b8b81010423384005588fa0a4c48af3702
- resource: https://github.com/abatyuk/okf/blob/main/docs/structured-metadata-capability.md
- resource: crates/okf-core/src/ontology/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: 0b71d797088eff17079d10e830f2150a820f3bc5
last_modified: 2026-10-05T10:18:41Z
---
# Declared structured metadata interpretation

One selector grammar walks `serde_yaml::Value` directly: dotted property names, explicit `[]`
list traversal, and JSON bracket-quoted literal keys. It does not infer traversal, evaluate code,
or expose a general JSONPath language. Occurrence paths preserve same-record pairing.

Named field types compose nested lists and objects. Explicit fields, values and item definitions
replace inherited counterparts. Recursive lint checks exact YAML types and declared constraints;
root metadata remains open and only explicitly closed custom objects reject unknown keys.
Configuration errors, unsupported constraints and incomplete checks remain distinguishable.

Declared selectors alone make custom metadata references. Semantic relationships carry authored
conditions; inverse backlinks do not express consent or compliance. Typed query filters combine
with AND, facets concern examined matches before paging, and bounded expansion reports gaps.
Computed display and projection records preserve authored metadata. Index coverage is a read-only
local navigation check; optional indexes do not become a conformance requirement.

[Explicit structured mutation inputs](structured-authoring.md) now author these values and
declarations through CLI YAML inputs, object paths, concrete patches and atomic ontology changes.
Query selectors and inverse views retain their read interpretation; mutation does not assert
verification, target consent, or policy compliance.
