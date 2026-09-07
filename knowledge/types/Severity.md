---
type: DomainType
title: Severity
description: 'A lint finding''s level: error, warn, or info.'
module: check
defined_in:
- /components/check
sources:
- resource: ../crates/okf-core/src/check/lint/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 9a19460fb9ee933ba546e14b00cb6dca2054bab3
last_modified: 2026-09-07T18:22:07Z
---
# Severity

A lint finding's level: error, warn, or info.

## Schema

```rust
pub enum Severity {
    Info, Warn, Error,
}
```

Defined in `okf-core`.
