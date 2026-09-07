---
type: 'DesignDecision'
title: 'Pure core; all effects go through ports'
description: 'fs, git, clock, and net are traits with real implementations in production and fakes in tests.'
status: 'decided'
affects:
  - /components/ports
  - /components/fingerprint
  - /components/check
---
# Pure core; all effects go through ports

fs, git, clock, and net are traits with real implementations in production and fakes in tests. Set up on day one so fingerprint, stale, and diff are testable without a real repo or network, and check/query stay deterministic.
