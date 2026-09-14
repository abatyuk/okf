---
type: Command
title: okf version
description: Prints the CLI version and the OKF spec version(s) it supports.
group: meta
mutates: false
implemented_by:
- /components/okf-cli
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 2e9e16e4bfa3271ab135d7f1ca406374da1a1407
last_modified: 2026-09-07T18:22:07Z
---
# okf version

Prints the CLI version and the OKF spec version(s) it supports.

## Arguments

_No arguments._

Global `--json` requests NDJSON.

Output stream: `version`.
