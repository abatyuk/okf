---
type: Command
title: okf version
description: Prints the CLI version and the OKF spec version(s) it supports.
group: meta
mutates: false
implemented_by:
- /components/okf-cli
sources:
- resource: ../crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: a1bdcb78fa383c159cc63689e31430beb78d30b4
last_modified: 2026-09-07T18:22:07Z
---
# okf version

Prints the CLI version and the OKF spec version(s) it supports.

## Arguments

_No arguments._

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `version`.
