---
type: Skill
title: okf:init skill
description: Creates a valid OKF v0.2 bundle and scaffolds portable concepts or exact Attested Computations, with ontology treated as an optional tool sidecar.
trigger: start a new bundle from scratch
sources:
- resource: https://github.com/abatyuk/okf/blob/5cf5e6773e137d30f61d21e6a680aec0f8478448/plugins/okf/skills/init/SKILL.md
uses:
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
