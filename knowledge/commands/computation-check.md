---
type: Command
title: okf computation check
description: Inspects an Attested Computation contract and its artifacts without executing the computation.
group: check
mutates: false
implemented_by:
- /components/check
- /components/query
---
# okf computation check

Checks the exact type, required runtime, parameters, inline-or-file computation choice, executor receipt, and attester resources. Its execution state is always `not-run`.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<concept>` | positional | yes | Concept id (leading slash optional), e.g. `tables/customers` |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `computation-contract`.
