---
type: Command
title: okf search
description: Searches concepts by type, tag, free text, and/or frontmatter field, returning matching concept records.
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
# okf search

Searches concepts by type, tag, free text, and/or frontmatter field, returning matching concept
records in deterministic relevance order by default.

Text search is case-insensitive. The default `phrase` mode matches reader-visible Markdown as
well as literal source: whitespace and soft line wraps within a paragraph are equivalent, and
inline formatting or link destinations do not interrupt visible phrases. Repeated `--text`
phrases combine with AND. Use `--match all` or `--match any` for token matching and `--match
literal` when Markdown source characters must match exactly.

Use `--in` to restrict text matching to `id`, `title`, `description`, `body`, and/or
arbitrary `frontmatter` values. Results rank exact ID/title matches above description,
frontmatter, and body matches; `--sort id` restores concept-ID order, while `--limit` is
applied after every filter and sorting step.

JSON results retain the concept record at the top level and add `search.score` plus bounded
`search.matches` evidence. Each match reports its query unit, field, snippet, and a one-based
serialized document line for body matches (`null` for metadata), directly usable with `show
--lines`.

```sh
okf search ./knowledge --text "travel policy" --text reimbursement
okf search ./knowledge --text "travel reimbursement" --match all --in title,body
okf search ./knowledge --text owner --in frontmatter --sort id --limit 20 --json
```

Structured queries retain literal-key `--field` compatibility while `--facet-filter` parses
selectors and nonnull typed JSON operands. Repeated conditions combine with AND; `in` supplies
alternatives within one condition. Missing/null values do not satisfy negative membership, while
a present empty list can. Objects and incompatible operand types are diagnosed without coercion.

`--project` emits separate projection records; views/columns do not overwrite frontmatter.
`--facets --json` aggregates matching documents before the page and preserves scan completeness.
`--offset` requires a positive `--limit`; `--scan-limit` and `--full-scan` conflict. Related expansion
is opt-in, one hop and outbound; selected relationship occurrences keep their attributes and
qualified target identities. Explicit scope bounds available targets and result claims.

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
