---
type: Skill
title: okf:init skill
description: Creates a valid OKF v0.2 bundle and scaffolds portable concepts or exact Attested Computations, with ontology treated as an optional tool sidecar.
trigger: start a new bundle from scratch
sources:
- resource: plugins/okf/skills/init/references/cli.md
  kind: git-path
  fingerprint:
    blob_sha: d8cf239c25f5fe66ce36b69b127d7fa7e26258cd
- resource: plugins/okf/skills/init/SKILL.md
  kind: git-path
  fingerprint:
    blob_sha: 4e6c2f350720756d90ba1d53378a9b4e406a09b3
- resource: https://github.com/abatyuk/okf/blob/5cf5e6773e137d30f61d21e6a680aec0f8478448/plugins/okf/skills/init/SKILL.md
- resource: plugins/okf/skills/ontology/references/structured-authoring.md
  kind: git-path
  fingerprint:
    blob_sha: a05ec02fcc363f85ee522cef43b0662a0d05b256
uses:
- /commands/ontology-apply
- /commands/ontology-field-type-add
- /commands/ontology-field-type-update
- /commands/ontology-field-type-remove
- /commands/catalog
- /commands/browse
- /commands/search
- /commands/list
- /commands/show
- /commands/init
- /commands/validate
- /commands/ontology-add
- /commands/ontology-show
- /commands/add
- /commands/computation-check
- /commands/lint
- /commands/docs
- /commands/artifact-list
- /commands/artifact-resolve
- /commands/artifact-show
last_modified: 2026-10-05T10:16:06Z
---
# okf:init skill

Creates a valid empty bundle, distinguishes required, recommended, optional, and extension
metadata, and uses `references/` only for authorized local artifacts.

The workflow connects [browse](../commands/browse.md), [search](../commands/search.md),
[list](../commands/list.md), [show](../commands/show.md), [init](../commands/init.md),
[validate](../commands/validate.md), [ontology add](../commands/ontology-add.md),
[ontology show](../commands/ontology-show.md), [add](../commands/add.md),
[computation check](../commands/computation-check.md), [lint](../commands/lint.md),
[docs](../commands/docs.md), [artifact list](../commands/artifact-list.md),
[artifact resolve](../commands/artifact-resolve.md), and
[artifact show](../commands/artifact-show.md). Use [migrate](migrate.md) or [ingest](ingest.md)
for existing material and [repair](repair.md) for an existing knowledge base.

Use [catalog](../commands/catalog.md) inspection to establish effective identity, local locations
and interpretation settings. Additional bundle examination is explicit; report unavailable members
and keep current candidates separate from requested snapshot evidence.

Structured authoring follows the ontology skill's shared example. Use structured declaration
YAML/file inputs and bulk apply for dependent schemas; use concept `--set-yaml`, object paths,
or RFC 6902 patches for frontmatter. Review dry-run and actual diffs, preserve source lineage,
and retain meaningful-change verification consequences. Model validation does not attest claims.
