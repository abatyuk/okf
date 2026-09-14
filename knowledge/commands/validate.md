---
type: Command
title: okf validate
description: 'Conformance validation: the spec''s three hard rules only (parseable frontmatter, non-empty type, reserved-filename structure).'
group: check
mutates: false
implemented_by:
- /components/check
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:07Z
---
# okf validate

Conformance validation: the spec's three hard rules only (parseable frontmatter, non-empty type, reserved-filename structure). Stays permissive about everything else. Exit 1 if nonconformant.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `violation`.
