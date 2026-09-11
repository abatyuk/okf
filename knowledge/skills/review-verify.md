---
type: Skill
title: okf:review-verify skill
description: Reviews concepts against declared evidence and appends document-level verified events with the real reviewer actor, never as a substitute for runtime attestation.
trigger: review and verify concepts
uses: [/commands/verify, /commands/show, /commands/artifact-resolve, /commands/stale]
---
# okf:review-verify skill

Separates source-backed document verification from execution attestation, validates reviewer
identity, and reports the exact verification event and remaining lifecycle limitations.
