---
type: Command
title: okf affected
description: 'Impact query: given a set of changed links, computes the blast radius of concepts that may need review via a reverse walk, direct by default or --transitive.'
group: check
mutates: false
implemented_by:
- /components/graph
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
# okf affected

Impact query: given a set of changed links, computes the blast radius of concepts that may need review via a reverse walk, direct by default or --transitive.

Catalog-aware affected analysis walks qualified reverse edges within explicitly examined scope.
Requested and available bundle/version scope accompany results; consumers outside that scope
remain unknown. Snapshot mismatch, ordinary target resolution and fingerprint change are
separate results. A current candidate is not an accepted substitute.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |
| `--changed <value>` | list<string> | no | A changed link/concept-id/resource (repeatable; also read from stdin lines) |
| `--transitive` | bool | no | Follow the cascade past direct dependents (default: `false`) |
| `--depth <value>` | int | no | Cap hops with --transitive; otherwise ignored with a diagnostic |
| `--fail-on <value>` | string | no | Fail (exit 1) on any affected concept: never (default) | info | warn | error | any (default: `never`) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `affected,scope,bundle-affected`.
