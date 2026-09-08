---
type: 'DesignDecision'
title: 'Preserve comments in ontology.yaml edits'
description: 'Typed serialization validates ontology edits; a key-path merge restores comments to surviving keys.'
status: 'resolved'
affects:
  - /components/ontology
---
# Preserve comments in ontology.yaml edits

Typed serialization remains the validation boundary for `ontology.yaml`. Before writing, the
editor maps leading and inline comments to their YAML key paths and restores them to keys that
survive the edit. Formatting and blank-line layout may normalize, and comments belonging to a
removed key disappear with that key.
