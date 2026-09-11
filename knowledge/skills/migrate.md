---
type: Skill
title: okf:migrate skill
description: Faithfully migrates authored or legacy documents into OKF v0.2 with standard sources, claim footnotes, and preserved extensions.
trigger: convert existing docs into concepts
sources:
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/migrate/SKILL.md
uses:
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
