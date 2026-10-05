---
type: DomainType
title: Affected record
description: 'The NDJSON record for an impact-query result: a concept reached by the affected reverse walk and why it was reached.'
module: output
defined_in:
- /components/output
sources:
- resource: crates/okf-cli/src/commands/check.rs
  kind: git-path
  fingerprint:
    blob_sha: a09a2fd8e2273f85b0f64bb012797968d9dc634f
last_modified: 2026-10-04T20:39:29Z
---
# Affected record

The NDJSON record emitted by [okf affected](../commands/affected.md): a concept reached by the affected reverse walk and why it was reached.

## Schema

```jsonc
// `--json` record emitted by `okf affected` (one per impacted concept).
{"kind": "affected", "concept": "/policies/travel"}
```

Catalog-aware impact output also emits scoped qualified identities through `bundle-affected`
records. Requested/examined/unavailable scope limits coverage; unexamined consumers remain
unknown. Consult [graph](../components/graph.md) and the generated command reference.
