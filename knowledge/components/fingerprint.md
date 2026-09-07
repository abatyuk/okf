---
type: Component
title: fingerprint module
description: 'Source fingerprinting, dispatched on SourceKind: git-commit and git-path via the git CLI, line-range and markdown-heading over canonicalized text, and optional url fingerprints behind a feature flag.'
layer: core
depends_on:
- /components/model
- /components/ports
sources:
- resource: ../crates/okf-core/src/fingerprint/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: b68e0a3cde62ee7cf8d68292512ecf7ffea5f954
last_modified: 2026-09-07T18:47:09Z
---
# fingerprint module

Source fingerprinting, dispatched on SourceKind: git-commit and git-path via the git CLI, line-range and markdown-heading over canonicalized text, and optional url fingerprints behind a feature flag. Canonicalization normalizes line endings and whitespace so fingerprints stay stable.

Location: `crates/okf-core/src/fingerprint`.
