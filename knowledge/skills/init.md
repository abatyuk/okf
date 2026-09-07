---
type: Skill
title: okf:init skill
description: Bootstraps an empty bundle, establishes an ontology.yaml, and scaffolds the first concepts.
trigger: start a new bundle from scratch
uses:
- /commands/init
- /commands/ontology-add
- /commands/add
- /commands/edit
- /commands/validate
- /commands/lint
- /commands/docs
---
# okf:init skill

Bootstraps an empty bundle, establishes an ontology.yaml, and scaffolds the first concepts. The judgment layer over okf init: it decides the domain, the concept types, and the initial prose.
