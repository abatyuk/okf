---
type: Skill
title: okf:ontology skill
description: Manages an optional advisory ontology while keeping portable OKF fields and exact Attested Computation semantics separate from custom rules.
trigger: author and manage the ontology
sources:
- resource: https://github.com/abatyuk/okf/blob/5cf5e6773e137d30f61d21e6a680aec0f8478448/plugins/okf/skills/ontology/SKILL.md
uses:
- /commands/ontology-list
- /commands/ontology-show
- /commands/lint
- /commands/ontology-add
- /commands/ontology-update
- /commands/ontology-remove
- /commands/validate
- /commands/artifact-resolve
---
# okf:ontology skill

Treats `ontology.yaml` as a tool-local sidecar that cannot make an unknown type nonconformant or
rename another type into a standard Attested Computation.

The workflow connects [ontology list](../commands/ontology-list.md),
[ontology show](../commands/ontology-show.md), [lint](../commands/lint.md),
[ontology add](../commands/ontology-add.md), [ontology update](../commands/ontology-update.md),
[ontology remove](../commands/ontology-remove.md), [validate](../commands/validate.md), and
[artifact resolve](../commands/artifact-resolve.md).
