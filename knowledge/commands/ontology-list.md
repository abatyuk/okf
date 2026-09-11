---
type: Command
title: okf ontology list
description: Lists the concept types defined in the tool-local ontology.yaml with their descriptions.
group: query
mutates: false
implemented_by:
- /components/ontology
sources:
- resource: crates/okf-cli/src/cli.rs
  kind: git-path
  fingerprint:
    blob_sha: 7a22138be071fc20ede774d812f9f98d018e7341
last_modified: 2026-09-07T18:22:06Z
---
# okf ontology list

Lists the concept types defined in the tool-local ontology.yaml with their descriptions.

## Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<bundle>` | positional | no | Bundle directory (defaults to $OKF_BUNDLE, then the current directory) (default: `.`) |

Every command also accepts global `--json` and an optional trailing `bundle` positional.

Output stream: `ontology_type`.
