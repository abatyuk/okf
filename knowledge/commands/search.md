---
type: Command
title: okf search
description: Searches concepts by type, tag, free text, and/or frontmatter field, returning matching concept records.
group: query
mutates: false
implemented_by:
- /components/query
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:07Z
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

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `concept`.
