---
type: Command
title: okf catalog
description: Inspects configured and effective catalog locations, availability and bundle settings.
group: meta
mutates: false
implemented_by:
- /components/bundle
sources:
- resource: crates/okf-cli/src/commands/catalog.rs
  kind: git-path
  fingerprint:
    blob_sha: b1189d92774c88cf1c3c4d5108d3da823b141b5c
- resource: crates/okf-cli/src/commands/schema.rs
  kind: git-path
  fingerprint:
    blob_sha: bab34f363d28125adac33f33fefc2c9e04329d73
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 6938cba0788280234436282cdce1e4a880b25d22
- resource: crates/okf-core/src/bundle/catalog.rs
  kind: git-path
  fingerprint:
    blob_sha: 1c08a7309b9fb6ed49c9f14713095aee01b641b3
- resource: crates/okf-core/src/bundle/context.rs
  kind: git-path
  fingerprint:
    blob_sha: c5a39cc31c202ec2ffc6da62728fb2423d679caf
last_modified: 2026-10-04T20:39:29Z
---
# okf catalog

Lists effective local registrations without selecting a primary bundle. IDs, configured and
overridden locations, availability and applicable tool settings are inspection context. Unknown
override IDs, conflicting registrations and malformed configuration are errors. Catalog
registration does not authorize traversal or prove publisher identity. The CLI does not acquire
remote bundles or accept different snapshots through a location override.

## Arguments

_No arguments._

Global `--json` requests NDJSON.

Output stream: `bundle-registration,effective-settings`.
