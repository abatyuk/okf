---
type: DomainType
title: Finding
description: 'A single lint result: a severity, a rule name, the concept it applies to, and a message.'
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
# Finding

A single lint result: a severity, a rule name, the concept it applies to, and a message. The --fail-on threshold decides which findings affect the exit code.

## Schema

```rust
pub struct Finding {
    pub rule: String,
    pub severity: Severity,
    pub concept: Option<String>,
    pub message: String,
}
```

Defined in `okf-core`.
