---
type: DomainType
title: TrustTier
description: 'The derived trust level of a concept: unverified, machine-confirmed, or human-reviewed.'
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
# TrustTier

The derived trust level of a concept: unverified, machine-confirmed, or human-reviewed. Never asserted directly; computed only from structurally valid [Verified](Verified.md) events using valid [Actor](Actor.md) and timestamp syntax.

## Schema

```rust
pub enum TrustTier {
    Unverified,        // no verified entries
    MachineConfirmed,  // verified by non-human actors only
    HumanReviewed,     // at least one `human:` actor
}
```

Defined in `okf-core`.
