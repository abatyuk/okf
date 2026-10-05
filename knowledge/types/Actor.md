---
type: DomainType
title: Actor
description: 'Who performed a verification, carried as a prefixed string such as human: or process:.'
module: model
defined_in:
- /components/model
sources:
- resource: crates/okf-core/src/model/standard.rs
  kind: git-path
  fingerprint:
    blob_sha: 7a448d2997e70754c1726193dc465301bf1417b2
- resource: crates/okf-core/src/model/trust.rs
  kind: git-path
  fingerprint:
    blob_sha: 8468ffc5adf9f8080dc6045a7b947542c0307bcd
last_modified: 2026-10-04T20:39:29Z
---
# Actor

Who performed a generation or [verification event](Verified.md). Valid forms are `human:<id>`, `process:<id>`, or `<producer>/<version>`. A valid human verification lifts [TrustTier](TrustTier.md) to human-reviewed; other valid verification actors give machine-confirmed.

## Schema

**Convention, not a struct.** An actor is the `by` value on a `verified` entry, a
prefixed string:

- `human:<id>`  — e.g. `human:andrey` → derives trust tier **human-reviewed**
- `process:<id>` — e.g. `process:ci` / `process:dbt` → **machine-confirmed**
- `<producer>/<version>` — an agent/tool producer → **machine-confirmed** when used in a valid verification event
