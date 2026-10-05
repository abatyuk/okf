---
type: Command
title: okf list
description: Lists all concepts (search with no filter).
group: query
mutates: false
implemented_by:
- /components/query
sources:
- resource: crates/okf-cli/src/commands/query.rs
  kind: git-path
  fingerprint:
    blob_sha: c9b643257e90f1e3e225cb4613be98f08f6abd02
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: 126dddfab3fb8caa78cfd61396db89dbcc324a33
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 28db6285f18a0400ab4b08c79c46322e58c2d833
last_modified: 2026-10-04T20:39:29Z
---
# okf list

Lists all concepts (search with no filter). Human output is a table of id, type, trust tier, and title; --json emits one concept record per line.

Named views, explicit columns/projections, JSON facets, pagination, and related expansion use the
same structured query interpretation as [search](search.md). Ordinary JSON concept records retain
authored metadata. Unrequested catalog bundles remain outside examination scope.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--type <value>` | string | no | Filter by exact concept `type` |
| `--tag <value>` | string | no | Filter by membership in the concept's `tags` |
| `--text <value>` | list<string> | no | Text query (repeatable). Repeated phrases use AND semantics by default |
| `--match <value>` | string | no | Text matching: phrase (default), all tokens, any token, or literal source text (default: `phrase`) |
| `--in <value>` | list<string> | no | Text fields to search (comma-separated or repeatable) (default: `id,title,description,body`) |
| `--sort <value>` | string | no | Result order: deterministic relevance (default) or concept id (default: `relevance`) |
| `--limit <value>` | int | no | Return at most this many results after all filters and sorting |
| `--field <value>` | list<string> | no | Filter by a frontmatter field, `key=value` (repeatable; AND) |
| `--facet-filter <value>` | list<string> | no | Typed nested selector condition (repeatable; AND) |
| `--facets` | bool | no | Emit JSON facets over all matches, before pagination (default: `false`) |
| `--facet <value>` | list<string> | no | Facet selector (repeatable); does not bypass high cardinality guard |
| `--offset <value>` | int | no | Start page at this nonnegative offset; requires a positive limit |
| `--scan-limit <value>` | int | no | Maximum examined documents across scope (positive) |
| `--full-scan` | bool | no | Examine the entire selected scope without a document scan budget (default: `false`) |
| `--project <value>` | list<string> | no | Explicit metadata projection selector (repeatable) |
| `--columns <value>` | list<string> | no | Human output column selector (repeatable) |
| `--view <value>` | string | no | Named bundle display view |
| `--expand <value>` | list<string> | no | Expand selected outbound relationship rule (repeatable) |
| `--no-expand` | bool | no | Disable configured human view expansion (default: `false`) |
| `--target-field <value>` | list<string> | no | Metadata selector for related targets (repeatable) |
| `--expansion-edges <value>` | int | no | Maximum edge occurrences per primary hit (1..100) (default: `10`) |
| `--expansion-targets <value>` | int | no | Maximum distinct target payloads per query (1..1000) (default: `100`) |
| `--expansion-bytes <value>` | int | no | Maximum serialized expansion bytes (1..1048576) (default: `262144`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `concept,concept-identity,scope,projection,query-summary,facet,facet-excluded,relationship,related-concept,expansion-summary,warning`.
