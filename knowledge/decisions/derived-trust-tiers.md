---
type: 'DesignDecision'
title: 'Trust tiers are derived, never asserted'
description: 'A concept''s tier is computed from its verified actors (a human: actor gives human-reviewed, other actors give machine-confirmed, none gives unverified).'
decision_status: 'decided'
affects:
  - /components/model
---
# Trust tiers are derived, never asserted

A concept's tier is computed from its verified actors (a human: actor gives human-reviewed, other actors give machine-confirmed, none gives unverified). Trust cannot be claimed in frontmatter, only earned through verification entries.
