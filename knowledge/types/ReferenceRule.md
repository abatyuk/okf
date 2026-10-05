---
type: DomainType
title: ReferenceRule
description: A declared concept reference with target types, cardinality and optional nested selector.
module: ontology
defined_in:
- /components/ontology
sources:
- resource: crates/okf-core/src/ontology/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: 0b71d797088eff17079d10e830f2150a820f3bc5
last_modified: 2026-10-04T20:39:29Z
---
# ReferenceRule

A declared edge rule with allowed target type(s), [cardinality](Cardinality.md), and an optional
nested selector. The legacy map key continues to name a literal top-level field. With a selector,
`relations[].target` walks explicit list occurrences, while `["policy.status"]` names a literal
dotted key. Each occurrence retains its concrete field path for diagnostics and semantic pairing.

[Ontology](../components/ontology.md) relationships name the reference rule and optionally supply
kind selectors, inverse names, and attribute selectors. Those values must pair in the same
record; a named inverse is a view of an incoming assertion, not a reverse assertion written by
the target. Paths outside the selected examination scope remain unknown.

[Lint](../commands/lint.md) checks rules advisorily. Ordinary custom strings, artifact paths and
standard source resources are not reinterpreted as typed relationships without a declaration.

Defined in `crates/okf-core/src/ontology/schema.rs`.
