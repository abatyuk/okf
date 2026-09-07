---
type: 'DesignDecision'
title: 'Comment loss in ontology.yaml (v1)'
description: 'serde-based YAML drops comments, but ontology.yaml is hand-edited and commented.'
status: 'open'
affects:
  - /components/ontology
---
# Comment loss in ontology.yaml (v1)

serde-based YAML drops comments, but ontology.yaml is hand-edited and commented. v1 accepts comment loss as a documented limitation; a CST-based surgical editor for ontology.yaml is a later upgrade.
