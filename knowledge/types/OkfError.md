---
type: DomainType
title: OkfError
description: The crate's error type.
module: okf-core
defined_in:
- /components/okf-core
sources:
- resource: ../crates/okf-core/src/error.rs
  kind: git-path
  fingerprint:
    blob_sha: c6d529332af5c9204af5fc744ee0b693297fb9fd
last_modified: 2026-09-07T18:22:07Z
---
# OkfError

The [okf-core](../components/okf-core.md) error type. Each variant maps to an exit class (2 usage,
3 environment/I/O, 4 internal); core never calls `process::exit`, but returns these for
[okf-cli](../components/okf-cli.md) to map.

## Schema

```rust
pub enum OkfError {
    Usage(String),        // bad args            → exit 2
    Environment(String),  // missing/unreadable  → exit 3
    Io(String),           // I/O failure         → exit 3
    Yaml(String),         // YAML parse failure   → exit 3
    Internal(String),     // broken invariant     → exit 4
}
```

Defined in `okf-core`.
