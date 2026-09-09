---
type: Component
title: render module
description: Derived output artifacts, including deterministic specification-conformant index.md generation.
layer: core
depends_on:
- /components/model
- /components/graph
last_modified: 2026-09-09T13:13:43Z
sources:
- resource: crates/okf-core/src/render/index.rs
  kind: git-path
  fingerprint:
    blob_sha: 35b21f6070042bf47f4651aed138ab2bc274f53a
---
# render module

Derived output artifacts: progressive-disclosure index.md generation and the optional html/md/pdf/graphml/obsidian document renderers behind feature flags.

Location: `crates/okf-core/src/render`.
