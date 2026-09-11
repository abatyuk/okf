---
type: DomainType
title: Affected record
description: 'The NDJSON record for an impact-query result: a concept reached by the affected reverse walk and why it was reached.'
module: output
defined_in:
- /components/output
sources:
- resource: ../crates/okf-cli/src/commands/check.rs
  kind: git-path
  fingerprint:
    blob_sha: ab70a3ae16e74bf72335cb4204a919490b63dfc0
last_modified: 2026-09-07T18:22:07Z
---
# Affected record

The NDJSON record emitted by [okf affected](../commands/affected.md): a concept reached by the affected reverse walk and why it was reached.

## Schema

```jsonc
// `--json` record emitted by `okf affected` (one per impacted concept).
{"kind": "affected", "concept": "/policies/travel"}
```
