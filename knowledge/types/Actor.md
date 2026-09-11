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

Who performed a generation or verification event. Valid forms are `human:<id>`, `process:<id>`, or `<producer>/<version>`. A valid human verification lifts trust to human-reviewed; other valid verification actors give machine-confirmed.

## Schema

**Convention, not a struct.** An actor is the `by` value on a `verified` entry, a
prefixed string:

- `human:<id>`  — e.g. `human:andrey` → derives trust tier **human-reviewed**
- `process:<id>` — e.g. `process:ci` / `process:dbt` → **machine-confirmed**
- `<producer>/<version>` — an agent/tool producer → **machine-confirmed** when used in a valid verification event
