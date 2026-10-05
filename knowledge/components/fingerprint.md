---
type: Component
title: fingerprint module
description: 'Source fingerprinting, dispatched on SourceKind: git-commit and git-path via the git CLI, line-range and markdown-heading over canonicalized text, and optional url fingerprints behind a feature flag.'
layer: core
depends_on:
- /components/model
- /components/parse
- /components/ports
sources:
- resource: crates/okf-core/src/fingerprint/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 73122fdeb462398fc9d3daaab7ef43d3acfc57a1
last_modified: 2026-10-04T20:39:29Z
---
# fingerprint module

Source fingerprinting, dispatched on [SourceKind](../types/SourceKind.md): git-commit and git-path through effect [ports](ports.md), line-range and markdown-heading through Markdown [parsing](parse.md), and optional URL fingerprints behind a feature flag. Canonicalization normalizes line endings and whitespace so [Fingerprint](../types/Fingerprint.md) values stay stable.

Location: `crates/okf-core/src/fingerprint`.
