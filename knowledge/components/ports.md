---
type: Component
title: ports module
description: 'Effect boundaries expressed as traits: fs, git, clock, and net.'
layer: core
sources:
- resource: ../crates/okf-core/src/ports/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: f13910f045fc463757f73d3255df250876e0807a
last_modified: 2026-09-07T18:47:09Z
---
# ports module

Effect boundaries expressed as traits: fs, git, clock, and net. Real implementations in production, fakes in tests, so fingerprint/stale/diff stay hermetically testable. The git port is a thin wrapper over the git CLI, not a git library.

Location: `crates/okf-core/src/ports`.
