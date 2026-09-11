---
type: Skill
title: okf:ontology skill
description: Manages an optional advisory ontology while keeping portable OKF fields and exact Attested Computation semantics separate from custom rules.
trigger: author and manage the ontology
uses: [/commands/ontology-add, /commands/ontology-update, /commands/ontology-remove, /commands/lint]
---
# okf:ontology skill

Treats `ontology.yaml` as a tool-local sidecar that cannot make an unknown type nonconformant or
rename another type into a standard Attested Computation.
