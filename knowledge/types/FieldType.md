---
type: DomainType
title: FieldType
description: 'The type of a field: string, text, int, bool, date, datetime, uri, enum, list, object, or a name defined under field_types.'
module: ontology
defined_in:
- /components/ontology
sources:
- resource: ../crates/okf-core/src/ontology/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: 878335dafa46f950612887e5a0aad823f8a573a3
last_modified: 2026-09-07T18:22:07Z
---
# FieldType

The type of a field: string, text, int, bool, date, datetime, uri, enum, list, object, or a name defined under field_types. Composes via extends and nested object fields.

## Schema

```rust
pub enum FieldType {
    String, Text, Int, Bool, Date, Datetime, Uri, Enum, List, Object,
}
```

Defined in `okf-core`.
