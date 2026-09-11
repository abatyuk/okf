---
type: 'DesignDecision'
title: 'Pure core; all effects go through ports'
description: 'fs, git, clock, and net are traits with real implementations in production and fakes in tests.'
decision_status: 'decided'
affects:
  - /components/ports
  - /components/fingerprint
  - /components/check
---
# Pure core; all effects go through ports

The [ports module](../components/ports.md) expresses fs, Git, clock, and net as traits with real
implementations in production and fakes in tests. That keeps
[fingerprinting](../components/fingerprint.md), [stale](../commands/stale.md), and
[diff](../commands/diff.md) testable without a real repository or network, while
[check](../components/check.md) and [query](../components/query.md) stay deterministic.
