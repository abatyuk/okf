---
type: Component
title: mutate module
description: Atomic, self-validating concept writes that preserve unknown YAML meaning and body text while applying bundle mutations.
layer: core
depends_on:
- /components/bundle
- /components/check
- /components/fingerprint
- /components/model
- /components/ontology
- /components/graph
- /components/parse
- /components/ports
sources:
- resource: crates/okf-core/src/mutate/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 5ab17148bcf6a0e285d113adffda8a02020e4e0e
- resource: crates/okf-core/src/mutate/structured.rs
  kind: git-path
  fingerprint:
    blob_sha: 6c504c52f0f9f23b83fde8b312aae82dafcc8c78
last_modified: 2026-10-05T10:16:06Z
---
# mutate module

Atomic, self-validating concept writes preserve unknown YAML meaning and body text while applying init, add, edit, mv, rm, verify, and refresh. They compose [bundle](bundle.md), [check](check.md), [fingerprint](fingerprint.md), [model](model.md), [ontology](ontology.md), [graph](graph.md), [parse](parse.md), and effect [port](ports.md) services. YAML comments and scalar presentation may normalize.

Location: `crates/okf-core/src/mutate`.

Structured frontmatter mutations operate on YAML values before persistence. Object paths create
missing maps and reject list traversal; RFC 6902 patches use concrete JSON Pointer addresses.
All patch operations and concept conformance checks complete before writing. Dry-run applies the
same preflight without filesystem persistence or lifecycle changes. Meaningful writes preserve
existing verification invalidation and generated timestamp semantics.
