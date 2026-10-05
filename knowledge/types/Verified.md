---
type: DomainType
title: Verified
description: 'A single verification entry in a concept''s verified list: an actor plus a timestamp.'
module: model
defined_in:
- /components/model
sources:
- resource: crates/okf-core/src/model/trust.rs
  kind: git-path
  fingerprint:
    blob_sha: 8468ffc5adf9f8080dc6045a7b947542c0307bcd
last_modified: 2026-10-04T20:39:29Z
---
# Verified

A single verification entry in a concept's verified list: an [Actor](Actor.md) plus a timestamp. The set of these entries determines the derived [TrustTier](TrustTier.md), and [okf verify](../commands/verify.md) appends them.

## Schema

```yaml
# a `verified:` frontmatter entry (the write-side of trust, appended by `okf verify`)
verified:
  - by: process:ci      # an Actor string (see the Actor type)
    at: 2026-01-01      # ISO date the verification happened
```
