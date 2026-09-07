---
type: 'DesignDecision'
title: 'Git via the CLI, not a library'
description: 'Git operations shell out to the git CLI through the git port, with no gix or git2 or libgit2 build step.'
status: 'resolved'
affects:
  - /components/fingerprint
  - /components/ports
---
# Git via the CLI, not a library

Git operations shell out to the git CLI through the git port, with no gix or git2 or libgit2 build step. It is runtime-optional: only git-based source kinds and okf diff need it, and they fail cleanly when git is absent.
