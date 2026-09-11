---
type: Skill
title: okf:repair skill
description: Uses okf doctor to diagnose and safely repair an existing bundle for v0.2 while preserving legal extensions and stopping for semantic choices.
trigger: repair, upgrade, or doctor an existing bundle
uses: [/commands/doctor, /commands/validate, /commands/lint, /commands/artifact-list]
---
# okf:repair skill

Runs a read-only compatibility preflight, previews allow-listed repairs, preserves unknown data,
and leaves ambiguous identity, lifecycle, link, and computation decisions to explicit review.
