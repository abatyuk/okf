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
last_modified: 2026-10-04T20:39:29Z
sources:
- resource: crates/okf-core/src/check/lint/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 5f3c08e23d20676ece7e1c824807ad0ae03f4cda
- resource: crates/okf-core/src/check/lint/rules/ontology_violation.rs
  kind: git-path
  fingerprint:
    blob_sha: 8548d2ba01a234dc9fe404d6674069ba9d4fc3c3
- resource: crates/okf-core/src/check/lint/rules/index_coverage.rs
  kind: git-path
  fingerprint:
    blob_sha: f079c8ba19b4a42376bfd0382a7090dec277ee87
- resource: crates/okf-core/src/check/validate.rs
  kind: git-path
  fingerprint:
    blob_sha: 18eedacd345b0d45bd56718f8f8011799fcd62d4
---
# check module

Diagnostics. Validate enforces all three conformance rules including index/log structure and root version syntax; lint checks optional families advisorily; stale compares corrected instants and fingerprint extensions; doctor performs tolerant upgrade preflight and safe repair. The implementation composes [bundle loading](bundle.md), the [model](model.md), [fingerprinting](fingerprint.md), the [ontology](ontology.md), the [graph](graph.md), [parsing](parse.md), effect [ports](ports.md), and artifact [queries](query.md).

Location: `crates/okf-core/src/check`.

Recursive metadata lint adds concrete paths and codes while retaining advisory ontology rules.
A depth or finding budget stop reports an incomplete check. Index coverage checks immediate
concept and concept-containing-directory links, honors configured exclusions, and remains
read-only. Neither local navigation policy nor optional ontology constraints tighten portable
OKF conformance.
