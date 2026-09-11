---
type: Skill
title: okf:review-verify skill
description: Reviews concepts against declared evidence and appends document-level verified events with the real reviewer actor, never as a substitute for runtime attestation.
trigger: review and verify concepts
sources:
- resource: https://github.com/abatyuk/okf/blob/5cf5e6773e137d30f61d21e6a680aec0f8478448/plugins/okf/skills/review-verify/SKILL.md
uses:
- /commands/verify
- /commands/show
- /commands/artifact-resolve
- /commands/stale
- /commands/list
- /commands/stats
- /commands/artifact-show
- /commands/computation-check
---
# okf:review-verify skill

Separates source-backed document verification from execution attestation, validates reviewer
identity, and reports the exact verification event and remaining lifecycle limitations.

The workflow connects [verify](../commands/verify.md), [show](../commands/show.md),
[artifact resolve](../commands/artifact-resolve.md), [stale](../commands/stale.md),
[list](../commands/list.md), [stats](../commands/stats.md),
[artifact show](../commands/artifact-show.md), and
[computation check](../commands/computation-check.md).
