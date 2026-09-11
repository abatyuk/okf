---
type: 'DesignDecision'
title: 'Hands vs. brain: thin CLI over a fat core'
description: 'The workspace splits deterministic logic (okf-core) from arg-parsing, output, and exit-code mapping (okf-cli), and pushes all judgment into agent skills that shell out to the binary.'
decision_status: 'decided'
affects:
  - /components/okf-core
  - /components/okf-cli
---
# Hands vs. brain: thin CLI over a fat core

The workspace splits deterministic logic ([okf-core](../components/okf-core.md)) from argument
parsing, output, and exit-code mapping ([okf-cli](../components/okf-cli.md)), and pushes judgment
into agent skills such as [ingest](../skills/ingest.md), [update](../skills/update.md), and
[repair](../skills/repair.md) that drive the binary. The core is the brain, the CLI is the hands,
and the skills are the will.
