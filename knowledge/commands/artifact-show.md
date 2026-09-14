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
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--from <value>` | string | no | Resolve a relative resource against this declaring concept id |
| `--lines <value>` | string | no | Retrieve only an inclusive, one-based START:END line range |
| `--max-bytes <value>` | int | no | Maximum bytes read into output (default: `65536`) |
| `--fetch` | bool | no | Explicitly request remote retrieval (requires a network-enabled build and policy) (default: `false`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `artifact-content`.
