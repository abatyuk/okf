---
type: Skill
title: okf:infer-ontology skill
description: Infers advisory custom fields and references from support counts while excluding standard OKF families and preserving exact unknown types.
trigger: reverse-engineer an ontology from a bundle
sources:
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/infer-ontology/SKILL.md
uses:
- /commands/list
- /commands/stats
- /commands/show
- /commands/resolve
- /commands/links
- /commands/backlinks
- /commands/computation-check
- /commands/ontology-add
- /commands/ontology-update
- /commands/validate
- /commands/lint
- /commands/artifact-resolve
---
# okf:infer-ontology skill

Reports evidence and confidence, recognizes the exact built-in computation contract, and never
infers required rules from a small sample.

The workflow connects [list](../commands/list.md), [stats](../commands/stats.md),
[show](../commands/show.md), [resolve](../commands/resolve.md),
[links](../commands/links.md), [backlinks](../commands/backlinks.md),
[computation check](../commands/computation-check.md),
[ontology add](../commands/ontology-add.md), [ontology update](../commands/ontology-update.md),
[validate](../commands/validate.md), [lint](../commands/lint.md), and
[artifact resolve](../commands/artifact-resolve.md).
