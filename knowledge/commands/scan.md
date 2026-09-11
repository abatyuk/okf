---
type: Command
title: okf scan
description: Walks the physical bundle tree and reports every Markdown file without parsing concepts.
group: check
mutates: false
implemented_by:
- /components/bundle
sources:
- resource: ../crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: a1bdcb78fa383c159cc63689e31430beb78d30b4
last_modified: 2026-09-07T18:22:06Z
---
# okf scan

Walks the physical bundle tree and reports every Markdown file, including reserved and Git-ignored files, without parsing concepts. Use [okf source-scan](source-scan.md) for a general repository inventory.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `scan`.
