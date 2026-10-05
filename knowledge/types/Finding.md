---
type: DomainType
title: Finding
description: 'A single lint result: a severity, a rule name, the concept it applies to, and a message.'
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
# Finding

A single lint result: a [Severity](Severity.md), a rule name, the concept it applies to, and a message. The `--fail-on` threshold decides which findings affect the exit code.

## Schema

```rust
pub struct Finding {
    pub rule: String,
    pub code: Option<String>,
    pub field_path: Option<String>,
    pub severity: Severity,
    pub concept: Option<String>,
    pub message: String,
}
```

Defined in `okf-core`.

Recursive metadata findings retain `ontology-violation` with detailed codes and concrete paths.
Index coverage and incomplete checks remain advisory findings. Catalog-aware wrappers identify
the examined bundle; neither configuration errors nor local policy tighten OKF conformance.
