---
type: DomainType
title: Change record
description: 'The NDJSON record for a diff result: a concept that was added, removed, or changed versus a git ref.'
module: output
defined_in:
- /components/output
sources:
- resource: ../crates/okf-cli/src/commands/mutate.rs
  kind: git-path
  fingerprint:
    blob_sha: 799c28f74bab65fc1376854ee1d486505f1a1ba5
last_modified: 2026-09-07T18:22:07Z
---
# Change record

The NDJSON record emitted by [init](../commands/init.md), [add](../commands/add.md),
[edit](../commands/edit.md), [mv](../commands/mv.md), [rm](../commands/rm.md),
[verify](../commands/verify.md), and [refresh](../commands/refresh.md). [okf diff](../commands/diff.md)
uses a related record to report concepts added, removed, or changed versus a Git ref.

## Schema

```jsonc
// `--json` record emitted by mutating commands (`init/add/edit/mv/rm/verify/refresh`).
// Shape varies by op; `op` names it. Example from `okf edit`:
{"kind": "change", "op": "edit", "id": "/types/Concept",
 "path": "knowledge/types/Concept.md",
 "changes": [{"op": "set", "key": "title", "existed": true}]}
// `okf diff` emits: {"kind": "diff", "change": "added|removed|changed", "concept": "<id>"}
```
