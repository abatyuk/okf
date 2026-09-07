---
type: Component
title: okf-cli crate
description: The thin binary named okf.
layer: cli
depends_on:
- /components/okf-core
sources:
- resource: ../crates/okf-cli/src/main.rs
  kind: git-path
  fingerprint:
    blob_sha: 70674e1af69e39b499c324cba922093d95d14936
last_modified: 2026-09-07T18:47:09Z
---
# okf-cli crate

The thin binary named okf. Responsible only for clap argument parsing, rendering core results as human text or NDJSON, and mapping results and errors to process exit codes. It holds no domain logic; every command is a thin function that parses args, calls okf-core, and hands the result to output.

Location: `crates/okf-cli`.
