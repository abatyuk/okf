---
type: Command
title: okf validate
description: 'Conformance validation: the spec''s three hard rules only (parseable frontmatter, non-empty type, reserved-filename structure).'
group: check
mutates: false
implemented_by:
- /components/check
sources:
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: bab34f363d28125adac33f33fefc2c9e04329d73
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 6938cba0788280234436282cdce1e4a880b25d22
last_modified: 2026-10-04T20:39:29Z
---
# okf validate

Conformance validation: the spec's three hard rules only (parseable frontmatter, non-empty type, reserved-filename structure). Stays permissive about everything else. Exit 1 if nonconformant.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `violation`.
