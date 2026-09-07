---
type: DomainType
title: Actor
description: 'Who performed a verification, carried as a prefixed string such as human: or process:.'
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
# Actor

Who performed a verification, carried as a prefixed string such as human: or process:. A human: actor lifts a concept to human-reviewed; other actors give machine-confirmed.

## Schema

**Convention, not a struct.** An actor is the `by` value on a `verified` entry, a
prefixed string:

- `human:<id>`  — e.g. `human:andrey` → derives trust tier **human-reviewed**
- `process:<id>` — e.g. `process:ci` / `process:dbt` → **machine-confirmed**
- any other non-`human:` prefix → **machine-confirmed**
