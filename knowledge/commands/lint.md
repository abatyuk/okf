---
type: Command
title: okf lint
description: 'Advisory checks where the opinions live: broken links, missing descriptions, orphans, and ontology violations, at error/warn/info severities with a --fail-on threshold.'
group: check
mutates: false
implemented_by:
- /components/check
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:06Z
---
# okf lint

Advisory checks where the opinions live: broken links, missing descriptions, orphans, and ontology violations, at error/warn/info severities with a --fail-on threshold.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--fix` | bool | no | Apply auto-fixable findings (v1: none are auto-fixable — reports what it would do) (default: `false`) |
| `--fail-on <value>` | string | no | Severity threshold that makes the run fail (exit 1): never|info|warn|error|any (default: `error`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional resolves as explicit argument, `$OKF_BUNDLE`, nearest `okf.toml`, then cwd.

Output stream: `finding`.
