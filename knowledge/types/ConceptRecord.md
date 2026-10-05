---
type: DomainType
title: Concept record (NDJSON)
description: 'The machine representation of a concept: frontmatter JSON plus collision-safe computed identity, trust, lifecycle, generation, and verification views.'
module: output
defined_in:
- /components/output
sources:
- resource: crates/okf-core/src/output/record.rs
  kind: git-path
  fingerprint:
    blob_sha: 3cb0820f0e1eaa08070e42adc897c4cf527988bc
last_modified: 2026-10-04T20:39:29Z
---
# Concept record (NDJSON)

The machine representation of a [Concept](Concept.md): [Frontmatter](Frontmatter.md) JSON plus collision-safe computed identity, [trust](TrustTier.md), lifecycle, generation, and [verification](Verified.md) views. A colliding frontmatter extension is retained under `frontmatter_conflicts`.

## Schema

```jsonc
// `--json` concept record: frontmatter values and order plus computed semantic views.
{
  "id": "/metrics/revenue",        // computed: bundle-relative path w/o `.md`
  "type": "Metric",                 // …all frontmatter keys, verbatim…
  "verified": [{"by": "process:dbt", "at": "2026-02-01T00:00:00Z"}],
  "trust_tier": "machine-confirmed", // computed from valid `verified`
  "effective_status": "stable",
  "latest_verified_at": "2026-02-01T00:00:00Z",
  "verification_current": null
}
```

Multi-bundle query output emits a separate `concept-identity` record before legacy concept/search
records. Qualified identity never overwrites authored `bundle` or `version` fields. Explicit
projections are separate versioned records with authored null distinguished from missing values.
