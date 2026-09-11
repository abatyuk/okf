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
    blob_sha: 7a22138be071fc20ede774d812f9f98d018e7341
last_modified: 2026-09-07T18:22:06Z
---
# okf lint

Advisory checks where the opinions live: broken links, missing descriptions, orphans, and ontology violations, at error/warn/info severities with a --fail-on threshold.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |
| `--fix` | bool | no | Apply auto-fixable findings (v1: none are auto-fixable — reports what it would do) |
| `--fail-on <value>` | string | no | Severity threshold that makes the run fail (exit 1): never|info|warn|error|any |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `finding`.
