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
- /commands/browse
---
# okf:retrieval skill

The primary consumer skill. It follows the specification's progressive-disclosure path by browsing `index.md` directory by directory, then uses search for targeted discovery, show for selective concept loading, and graph/backlinks for relationships.
