---
type: Command
title: okf lint
description: 'Advisory checks where the opinions live: broken links, missing descriptions, orphans, and ontology violations, at error/warn/info severities with a --fail-on threshold.'
group: check
mutates: false
implemented_by:
- /components/check
sources:
- resource: crates/okf-cli/src/commands/check.rs
  kind: git-path
  fingerprint:
    blob_sha: a09a2fd8e2273f85b0f64bb012797968d9dc634f
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: 126dddfab3fb8caa78cfd61396db89dbcc324a33
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 28db6285f18a0400ab4b08c79c46322e58c2d833
last_modified: 2026-10-04T20:39:29Z
---
# okf lint

Advisory checks where the opinions live: broken links, missing descriptions, orphans, and ontology violations, at error/warn/info severities with a --fail-on threshold.

Recursive ontology diagnostics retain the `ontology-violation` rule with concrete field paths
and detailed metadata codes. Invalid schema configuration is separate from advisory findings;
unsupported constraints and depth/finding-budget exhaustion remain visible. Local index-coverage
checks diagnose omitted immediate children and concept-containing directories without rewriting
curated navigation. Catalog-aware checks retain explicit examination scope.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--fix` | bool | no | Apply auto-fixable findings (v1: none are auto-fixable — reports what it would do) (default: `false`) |
| `--fail-on <value>` | string | no | Severity threshold that makes the run fail (exit 1): never|info|warn|error|any (default: `error`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `finding,scope`.
