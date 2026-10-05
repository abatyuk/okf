---
type: DomainType
title: Severity
description: 'A lint finding''s level: error, warn, or info.'
module: check
defined_in:
- /components/check
sources:
- resource: crates/okf-core/src/check/lint/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 5f3c08e23d20676ece7e1c824807ad0ae03f4cda
last_modified: 2026-10-04T20:39:29Z
---
# Severity

A lint [Finding](Finding.md)'s level: error, warn, or info.

## Schema

```rust
pub enum Severity {
    Info, Warn, Error,
}
```

Defined in `okf-core`.
