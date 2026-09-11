---
type: Command
title: okf artifact show
description: Retrieves a bounded local or explicitly authorized remote artifact without executing it.
group: query
mutates: false
implemented_by:
- /components/query
---
# okf artifact show

Returns bounded UTF-8 text or binary metadata and a digest. Remote HTTP(S) retrieval requires `--fetch` and the network-enabled build feature. Retrieval never grants execution authority.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<resource>` | positional | yes | Local artifact path to retrieve |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--from <value>` | string | no | Resolve a relative resource against this declaring concept id |
| `--lines <value>` | string | no | Retrieve only an inclusive, one-based START:END line range |
| `--max-bytes <value>` | string | no | Maximum bytes read into output (default: `65536`) |
| `--fetch` | bool | no | Explicitly request remote retrieval (requires a network-enabled build and policy) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `artifact-content`.
