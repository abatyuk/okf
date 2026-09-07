---
type: DomainType
title: Concept record (NDJSON)
description: 'The machine representation of a concept: its Frontmatter serialized to JSON verbatim plus computed id and trust_tier.'
module: output
defined_in:
- /components/output
sources:
- resource: ../crates/okf-core/src/output/record.rs
  kind: git-path
  fingerprint:
    blob_sha: 73e1e2469e27696da6e4919619917a6709cafbcf
last_modified: 2026-09-07T18:22:07Z
---
# Concept record (NDJSON)

The machine representation of a concept: its Frontmatter serialized to JSON verbatim plus computed id and trust_tier. Wraps Frontmatter rather than re-declaring fields.

## Schema

```jsonc
// `--json` `concept` record: the frontmatter serialized VERBATIM (key order preserved),
// plus two computed keys. Built by okf-core::output::record::concept_record.
{
  "id": "/metrics/revenue",        // computed: bundle-relative path w/o `.md`
  "type": "Metric",                 // …all frontmatter keys, verbatim…
  "verified": [{"by": "process:dbt", "at": "2026-02-01"}],
  "trust_tier": "machine-confirmed" // computed from `verified`
}
```
