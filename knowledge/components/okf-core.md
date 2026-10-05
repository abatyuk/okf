---
type: Component
title: okf-core crate
description: The deterministic core library.
layer: core
sources:
- resource: crates/okf-core/src/lib.rs
  kind: git-path
  fingerprint:
    blob_sha: 90fbc9075e106be68d4219d62f48d86bbe555e0a
last_modified: 2026-10-04T20:39:29Z
---
# okf-core crate

The deterministic core library. Its public module map comprises [bundle](bundle.md), [check](check.md), [fingerprint](fingerprint.md), [graph](graph.md), [model](model.md), [mutate](mutate.md), [ontology](ontology.md), [output](output.md), [parse](parse.md), [ports](ports.md), [query](query.md), and [render](render.md). It takes no argv and writes no stdout; it exposes typed results that the CLI renders. This is the brain half of the hands-vs-brain split.

Location: `crates/okf-core`.
