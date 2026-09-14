---
type: Command
title: okf diff
description: 'Concept-level diff of the working tree against a git ref: which concepts were added, removed, or changed.'
group: check
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
# okf diff

Concept-level diff of the working tree against a git ref: which concepts were added, removed, or changed.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<git_ref>` | positional | yes | Git ref to diff against (e.g. `HEAD`, a branch, or a commit) |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--fail-on <value>` | string | no | Fail (exit 1) on any change: never (default) | info | warn | error | any (default: `never`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `diff`.
