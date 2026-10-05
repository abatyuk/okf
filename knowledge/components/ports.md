---
type: Component
title: ports module
description: 'Effect boundaries expressed as traits: fs, git, clock, and net.'
layer: core
sources:
- resource: crates/okf-core/src/ports/git.rs
  kind: git-path
  fingerprint:
    blob_sha: 74481c33381ee3dc92eeed067d80b05d2993e503
- resource: crates/okf-core/src/ports/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 385ec85488e55ec0fe3b9e2f3d204c2bf6b1b1fd
last_modified: 2026-10-04T20:39:29Z
---
# ports module

Effect boundaries expressed as traits: fs, git, clock, and net. Real implementations in production, fakes in tests, so fingerprint/stale/diff stay hermetically testable. The git port is a thin wrapper over the git CLI, not a git library.

Location: `crates/okf-core/src/ports`.

Local Git reads disable implicit partial-clone object fetching and terminal prompts. Catalog
snapshot inspection does not acquire missing material or authorize execution.
