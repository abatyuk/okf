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
    blob_sha: 7a22138be071fc20ede774d812f9f98d018e7341
last_modified: 2026-09-07T18:22:06Z
---
# okf diff

Concept-level diff of the working tree against a git ref: which concepts were added, removed, or changed.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<git_ref>` | positional | yes | Git ref to diff against (e.g. `HEAD`, a branch, or a commit) |
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--fail-on <value>` | string | no | Fail (exit 1) on any change: never (default) | info | warn | error | any |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `diff`.
