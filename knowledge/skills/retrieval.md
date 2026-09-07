---
type: Skill
title: okf:retrieval skill
description: The primary consumer skill.
trigger: answer questions from a bundle
uses:
- /commands/search
- /commands/show
- /commands/graph
- /commands/backlinks
---
# okf:retrieval skill

The primary consumer skill. Answers questions via progressive disclosure (search, then show, then graph/backlinks) so the whole bundle never has to be loaded into context.
