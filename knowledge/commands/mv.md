---
type: Command
title: okf mv
description: Moves or renames a concept and rewrites every inbound link.
group: mutate
mutates: true
implemented_by:
- /components/mutate
sources:
- resource: crates/okf-cli/src/commands/mutate.rs
  kind: git-path
  fingerprint:
    blob_sha: 6805fef5102d8746ad060d24254e2e4423e69d38
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
# okf mv

Moves or renames a concept and rewrites body links, ontology references, and standard internal `sources[].resource` paths. It also rebases the moved concept's relative standard paths. Repository-relative `git-path` and `git-commit` source resources remain unchanged.

Moves rewrite declared nested-reference occurrences and curated navigation within the selected bundle.
Publication is journaled per file. Cross-bundle move orchestration remains unsupported.
Inspect explicit-scope consumers and coordinate reviewed changes across bundles manually; a
single-bundle move cannot establish that every external consumer was repaired.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<old>` | positional | yes | Existing concept id |
| `<new>` | positional | yes | New concept id |
| `<bundle>` | positional | no | Bundle directory (explicit, then $OKF_BUNDLE, nearest okf.toml, or current directory) |

Global `--json` requests NDJSON. The optional trailing `bundle` positional is a path; `--bundle-id` selects a catalog identity. Selection uses explicit path or ID, `$OKF_BUNDLE` (path), registered containing bundle, configured default, sole catalog entry, then uncataloged cwd. Selection and examination scope are separate.

Output stream: `change`.
