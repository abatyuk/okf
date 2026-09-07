---
type: 'DesignDecision'
title: 'Hands vs. brain: thin CLI over a fat core'
description: 'The workspace splits deterministic logic (okf-core) from arg-parsing, output, and exit-code mapping (okf-cli), and pushes all judgment into agent skills that shell out to the binary.'
status: 'decided'
affects:
  - /components/okf-core
  - /components/okf-cli
---
# Hands vs. brain: thin CLI over a fat core

The workspace splits deterministic logic (okf-core) from arg-parsing, output, and exit-code mapping (okf-cli), and pushes all judgment into agent skills that shell out to the binary. The core is the brain, the CLI is the hands, the skills are the will.
