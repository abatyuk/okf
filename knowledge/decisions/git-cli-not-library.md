---
type: 'DesignDecision'
title: 'Git via the CLI, not a library'
description: 'Git operations shell out to the git CLI through the git port, with no gix or git2 or libgit2 build step.'
decision_status: 'resolved'
affects:
  - /components/fingerprint
  - /components/ports
---
# Git via the CLI, not a library

Git operations shell out to the Git CLI through the [ports](../components/ports.md) boundary,
with no gix, git2, or libgit2 build step. It is runtime-optional: only Git-based
[SourceKind](../types/SourceKind.md) values in [fingerprinting](../components/fingerprint.md) and
[okf diff](../commands/diff.md) need it, and they fail cleanly when Git is absent.
