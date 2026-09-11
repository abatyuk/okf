---
type: 'DesignDecision'
title: 'Semantically preserving, order-preserving frontmatter'
description: 'Frontmatter keeps unknown keys, values, and order while YAML presentation may normalize.'
decision_status: 'decided'
affects:
  - /components/model
  - /components/parse
---
# Semantically preserving, order-preserving frontmatter

[Frontmatter](../types/Frontmatter.md) is an order-preserving mapping that keeps unknown keys and values, not a fixed struct. Everything round-trips through it semantically, which lets the tool add extension keys such as `kind` and `fingerprint` without discarding metadata. YAML comments, quoting, and scalar presentation are not represented and may normalize.
