---
type: Command
title: okf resolve
description: Resolves a link or concept id to a concrete bundle-relative file path.
group: query
mutates: false
implemented_by:
- /components/query
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:06Z
---
# okf resolve

Resolves a concept [Link](../types/Link.md) to a concrete bundle-relative Markdown path using standard document-relative semantics. Use [okf artifact resolve](artifact-resolve.md) for arbitrary path-valued resources.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<link>` | positional | yes | Link or concept id to resolve |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--from <value>` | string | no | Resolve a relative link against this containing concept id |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `resolved`.
