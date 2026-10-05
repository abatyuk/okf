---
type: Skill
title: okf:migrate skill
description: Faithfully migrates authored or legacy documents into OKF v0.2 with standard sources, claim footnotes, and preserved extensions.
trigger: convert existing docs into concepts
sources:
- resource: plugins/okf/skills/migrate/references/cli.md
  kind: git-path
  fingerprint:
    blob_sha: a303c062775c0eac252311e06eb34f49d39d2d6f
- resource: plugins/okf/skills/migrate/SKILL.md
  kind: git-path
  fingerprint:
    blob_sha: 4f2b4c04bbea831707c86c75fc0b0eda951ff7f4
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/migrate/SKILL.md
- resource: plugins/okf/skills/ontology/references/structured-authoring.md
  kind: git-path
  fingerprint:
    blob_sha: a05ec02fcc363f85ee522cef43b0662a0d05b256
uses:
- /commands/catalog
- /commands/source-scan
- /commands/add
- /commands/edit
- /commands/links
- /commands/refresh
- /commands/validate
- /commands/lint
- /commands/docs
- /commands/artifact-resolve
- /commands/artifact-show
last_modified: 2026-10-05T10:16:06Z
---
# okf:migrate skill

Migrates documentation without inventing provenance, translates legacy fields only when their
semantics are known, and keeps verification separate from migration.

The workflow connects [source-scan](../commands/source-scan.md), [add](../commands/add.md),
[edit](../commands/edit.md), [links](../commands/links.md),
[refresh](../commands/refresh.md),
[validate](../commands/validate.md), [lint](../commands/lint.md), [docs](../commands/docs.md),
[artifact resolve](../commands/artifact-resolve.md), and
[artifact show](../commands/artifact-show.md). Use [ingest](ingest.md) for repository research,
[repair](repair.md) for an in-place upgrade, and [review-verify](review-verify.md) for sign-off.

Use [catalog](../commands/catalog.md) inspection to establish effective identity, local locations
and interpretation settings. Additional bundle examination is explicit; report unavailable members
and keep current candidates separate from requested snapshot evidence.

Structured authoring follows the ontology skill's shared example. Use structured declaration
YAML/file inputs and bulk apply for dependent schemas; use concept `--set-yaml`, object paths,
or RFC 6902 patches for frontmatter. Review dry-run and actual diffs, preserve source lineage,
and retain meaningful-change verification consequences. Model validation does not attest claims.
