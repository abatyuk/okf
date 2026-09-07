---
type: Component
title: okf-core crate
description: The deterministic core library.
layer: core
sources:
- resource: ../crates/okf-core/src/lib.rs
  kind: git-path
  fingerprint:
    blob_sha: 0bd605e6c2ec6096e61f1c37e2435e21e8bd93ef
last_modified: 2026-09-07T18:47:09Z
---
# okf-core crate

The deterministic core library. All parsing, validation, linting, querying, graphing, fingerprinting, trust derivation, and rendering logic lives here. It takes no argv and writes no stdout; it exposes typed results that the CLI renders. This is the brain half of the hands-vs-brain split.

Location: `crates/okf-core`.
