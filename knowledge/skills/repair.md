---
type: Skill
title: okf:repair skill
description: Uses okf doctor to diagnose and safely repair an existing bundle for v0.2 while preserving legal extensions and stopping for semantic choices.
trigger: repair, upgrade, or doctor an existing bundle
sources:
- resource: https://github.com/abatyuk/okf/blob/5cf5e6773e137d30f61d21e6a680aec0f8478448/plugins/okf/skills/repair/SKILL.md
uses:
- /commands/doctor
- /commands/validate
- /commands/lint
- /commands/artifact-list
- /commands/docs
---
# okf:repair skill

Runs a read-only compatibility preflight, previews allow-listed repairs, preserves unknown data,
and leaves ambiguous identity, lifecycle, link, and computation decisions to explicit review.

The workflow connects [doctor](../commands/doctor.md), [validate](../commands/validate.md),
[lint](../commands/lint.md), [artifact list](../commands/artifact-list.md), and
[docs](../commands/docs.md).
