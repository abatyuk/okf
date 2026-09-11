---
type: Skill
title: okf:retrieval skill
description: Answers from a bundle through progressive disclosure, source lineage, bounded artifact retrieval, and explicit lifecycle, verification, and runtime caveats.
trigger: answer questions from a bundle
sources:
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/retrieval/SKILL.md
uses:
- /commands/browse
- /commands/search
- /commands/show
- /commands/links
- /commands/backlinks
- /commands/list
- /commands/graph
- /commands/resolve
- /commands/computation-check
- /commands/artifact-resolve
- /commands/artifact-show
---
# okf:retrieval skill

The primary read-only consumer skill. It follows concepts, claim sources, and optional artifacts
without bulk loading or confusing a verified computation definition with an attested run.

The workflow connects [browse](../commands/browse.md), [search](../commands/search.md),
[show](../commands/show.md), [links](../commands/links.md),
[backlinks](../commands/backlinks.md), [list](../commands/list.md),
[graph](../commands/graph.md), [resolve](../commands/resolve.md),
[computation check](../commands/computation-check.md),
[artifact resolve](../commands/artifact-resolve.md), and
[artifact show](../commands/artifact-show.md).
