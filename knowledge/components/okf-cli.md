---
type: Component
title: okf-cli crate
description: The thin binary named okf.
layer: cli
depends_on:
- /components/okf-core
sources:
- resource: crates/okf-cli/src/main.rs
  kind: git-path
  fingerprint:
    blob_sha: ae038ad94dc71ee34b878611777789cd891edbcc
- resource: crates/okf-cli/src/structured.rs
  kind: git-path
  fingerprint:
    blob_sha: 889819283f7bd1c986e0ce78deb39f77fd481633
last_modified: 2026-10-05T10:18:41Z
---
# okf-cli crate

The thin binary named okf. Responsible only for clap argument parsing, rendering [okf-core](okf-core.md) results as human text or NDJSON, and mapping results and errors to process exit codes. It holds no domain logic; every command is a thin function that parses args, calls the core, and hands the result to output.

## Command surface

- Meta: [schema](../commands/schema.md), [version](../commands/version.md), [catalog](../commands/catalog.md)
- Query: [list](../commands/list.md), [search](../commands/search.md), [show](../commands/show.md), [browse](../commands/browse.md), [links](../commands/links.md), [backlinks](../commands/backlinks.md), [graph](../commands/graph.md), [resolve](../commands/resolve.md), [artifact list](../commands/artifact-list.md), [artifact resolve](../commands/artifact-resolve.md), [artifact show](../commands/artifact-show.md), [computation check](../commands/computation-check.md)
- Check: [scan](../commands/scan.md), [source-scan](../commands/source-scan.md), [validate](../commands/validate.md), [lint](../commands/lint.md), [stale](../commands/stale.md), [affected](../commands/affected.md), [diff](../commands/diff.md), [stats](../commands/stats.md), [doctor](../commands/doctor.md)
- Mutate: [artifact put](../commands/artifact-put.md), [init](../commands/init.md), [add](../commands/add.md), [edit](../commands/edit.md), [mv](../commands/mv.md), [rm](../commands/rm.md), [verify](../commands/verify.md), [refresh](../commands/refresh.md), [ontology add](../commands/ontology-add.md), [ontology update](../commands/ontology-update.md), [ontology remove](../commands/ontology-remove.md), [ontology apply](../commands/ontology-apply.md), [field-type add](../commands/ontology-field-type-add.md), [field-type update](../commands/ontology-field-type-update.md), [field-type remove](../commands/ontology-field-type-remove.md)
- Ontology queries: [ontology list](../commands/ontology-list.md), [ontology show](../commands/ontology-show.md)
- Render: [docs](../commands/docs.md)

Location: `crates/okf-cli`.

Global identity, scope and revision flags remain distinct. Schema-derived references document
the supported command combinations, computed record contracts, and optional interpretation
settings. Query projections and semantic relationship payloads preserve legacy authored metadata.

Structured input loading accepts one YAML/JSON document, inline, from a file or from stdin.
It rejects ambiguous keys, unsupported YAML constructs and multiple stdin consumers before
calling core mutation logic. This is input interpretation; object-path and patch semantics remain
in the core mutation module.
