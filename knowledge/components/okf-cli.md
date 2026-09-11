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

The thin binary named okf. Responsible only for clap argument parsing, rendering [okf-core](okf-core.md) results as human text or NDJSON, and mapping results and errors to process exit codes. It holds no domain logic; every command is a thin function that parses args, calls the core, and hands the result to output.

## Command surface

- Meta: [schema](../commands/schema.md), [version](../commands/version.md)
- Query: [list](../commands/list.md), [search](../commands/search.md), [show](../commands/show.md), [browse](../commands/browse.md), [links](../commands/links.md), [backlinks](../commands/backlinks.md), [graph](../commands/graph.md), [resolve](../commands/resolve.md), [artifact list](../commands/artifact-list.md), [artifact resolve](../commands/artifact-resolve.md), [artifact show](../commands/artifact-show.md), [computation check](../commands/computation-check.md)
- Check: [scan](../commands/scan.md), [source-scan](../commands/source-scan.md), [validate](../commands/validate.md), [lint](../commands/lint.md), [stale](../commands/stale.md), [affected](../commands/affected.md), [diff](../commands/diff.md), [stats](../commands/stats.md), [doctor](../commands/doctor.md)
- Mutate: [init](../commands/init.md), [add](../commands/add.md), [edit](../commands/edit.md), [mv](../commands/mv.md), [rm](../commands/rm.md), [verify](../commands/verify.md), [refresh](../commands/refresh.md), [ontology add](../commands/ontology-add.md), [ontology update](../commands/ontology-update.md), [ontology remove](../commands/ontology-remove.md)
- Ontology queries: [ontology list](../commands/ontology-list.md), [ontology show](../commands/ontology-show.md)
- Render: [docs](../commands/docs.md)

Location: `crates/okf-cli`.
