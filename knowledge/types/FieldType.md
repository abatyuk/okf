---
type: DomainType
title: FieldType
description: 'The type of a field: string, text, int, bool, date, datetime, uri, enum, list, object, or a name defined under field_types.'
module: ontology
defined_in:
- /components/ontology
sources:
- resource: crates/okf-core/src/ontology/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: 0b71d797088eff17079d10e830f2150a820f3bc5
last_modified: 2026-10-04T20:39:29Z
---
# FieldType

The type of a [Field](Field.md): string, text, int, bool, date, datetime, URI, enum, list, object, or a name defined under `field_types`. Composes via `extends` and nested object fields.

## Schema

```rust
pub enum FieldType {
    String, Text, Int, Bool, Date, Datetime, Uri, Enum, List, Object,
}
```

Defined in `okf-core`.
