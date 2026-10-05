---
type: Command
title: okf version
description: Prints the CLI version and the OKF spec version(s) it supports.
group: meta
mutates: false
implemented_by:
- /components/okf-cli
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
# okf version

Prints the CLI version and the OKF spec version(s) it supports.

## Arguments

_No arguments._

Global `--json` requests NDJSON.

Output stream: `version`.
