---
type: 'DesignDecision'
title: 'okf schema is derived from the command tree'
description: 'The machine-readable command surface is introspected from the single clap definition, never hand-maintained, because two sources of truth for the surface would drift.'
status: 'decided'
affects:
  - /components/okf-cli
---
# okf schema is derived from the command tree

The machine-readable command surface is introspected from the single clap definition, never hand-maintained, because two sources of truth for the surface would drift.
