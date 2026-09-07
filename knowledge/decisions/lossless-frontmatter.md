---
type: 'DesignDecision'
title: 'Lossless, order-preserving frontmatter'
description: 'Frontmatter is an order-preserving mapping that keeps unknown keys, not a fixed struct.'
status: 'decided'
affects:
  - /components/model
  - /components/parse
---
# Lossless, order-preserving frontmatter

Frontmatter is an order-preserving mapping that keeps unknown keys, not a fixed struct. Everything round-trips through it, which is what lets the tool stay spec-pure while adding its own keys (kind, fingerprint) without loss.
