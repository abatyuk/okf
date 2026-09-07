---
type: Command
title: okf validate
description: 'Conformance validation: the spec''s three hard rules only (parseable frontmatter, non-empty type, reserved-filename structure).'
group: check
mutates: false
implemented_by:
- /components/check
sources:
- resource: ../crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: a1bdcb78fa383c159cc63689e31430beb78d30b4
last_modified: 2026-09-07T18:22:07Z
---
# okf validate

Conformance validation: the spec's three hard rules only (parseable frontmatter, non-empty type, reserved-filename structure). Stays permissive about everything else. Exit 1 if nonconformant.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `violation`.
