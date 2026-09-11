---
type: Skill
title: okf:repair skill
description: Uses okf doctor to diagnose and safely repair an existing bundle for v0.2 while preserving legal extensions and stopping for semantic choices.
trigger: repair, upgrade, or doctor an existing bundle
sources:
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/repair/SKILL.md
uses:
- /commands/doctor
- /commands/validate
- /commands/lint
- /commands/artifact-list
- /commands/links
- /commands/backlinks
- /commands/graph
- /commands/docs
---
# okf:repair skill

Runs a read-only compatibility preflight, previews allow-listed repairs, preserves unknown data,
and leaves ambiguous identity, lifecycle, link, and computation decisions to explicit review.

The workflow connects [doctor](../commands/doctor.md), [validate](../commands/validate.md),
[lint](../commands/lint.md), [artifact list](../commands/artifact-list.md), and
[links](../commands/links.md), [backlinks](../commands/backlinks.md),
[graph](../commands/graph.md), and [docs](../commands/docs.md).
