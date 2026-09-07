---
type: DomainType
title: Verified
description: 'A single verification entry in a concept''s verified list: an actor plus a timestamp.'
module: model
defined_in:
- /components/model
sources:
- resource: ../crates/okf-core/src/model/trust.rs
  kind: git-path
  fingerprint:
    blob_sha: 9988a3d3b50480eae3c1222e9a70c76eb1db5829
last_modified: 2026-09-07T18:22:07Z
---
# Verified

A single verification entry in a concept's verified list: an actor plus a timestamp. The set of these entries determines the derived TrustTier.

## Schema

```yaml
# a `verified:` frontmatter entry (the write-side of trust, appended by `okf verify`)
verified:
  - by: process:ci      # an Actor string (see the Actor type)
    at: 2026-01-01      # ISO date the verification happened
```
