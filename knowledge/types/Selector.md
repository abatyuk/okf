---
type: DomainType
title: Selector
description: A non-executable YAML metadata path with explicit list traversal and retained occurrence paths.
module: query
defined_in:
- /components/query
sources:
- resource: crates/okf-core/src/query/selector.rs
  kind: git-path
  fingerprint:
    blob_sha: 1d701ba77f220d734005c8f09f4f0cf255fbdfe0
last_modified: 2026-10-04T20:39:29Z
---
# Selector

Parses property steps, dotted nested properties, explicit `[]` list traversal, and bracket-quoted
JSON string keys. No implicit list traversal, recursive descent, indices, predicates or functions
are supported. Computed projection names such as `$id` are separate from authored selectors.

Selections retain leaf values, concrete paths, list indices, and empty-list information. Shared
reference, relationship, filter, projection and facet consumers use this representation rather
than executing an external expression language or flattening away occurrence identity.
