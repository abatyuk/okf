---
type: DomainType
title: OkfError
description: The crate's error type.
module: okf-core
defined_in:
- /components/okf-core
sources:
- resource: crates/okf-core/src/error.rs
  kind: git-path
  fingerprint:
    blob_sha: 51f2175ca838cad34df206f4d7dcaed07e396938
last_modified: 2026-10-04T20:39:29Z
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

`at_path` prefixes diagnostics with the failing file while preserving the variant and exit code. `yaml_files` combines the malformed concept files examined in a bundle load into one YAML error; conformance validation continues to report separate per-file findings.
