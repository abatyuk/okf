---
type: Skill
title: okf:reorganize skill
description: Moves concepts while rebasing body links and standard source resources, preserving opaque artifacts and validating regenerated indexes.
trigger: restructure the bundle layout
sources:
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/reorganize/SKILL.md
uses:
- /commands/list
- /commands/graph
- /commands/artifact-list
- /commands/mv
- /commands/links
- /commands/backlinks
- /commands/validate
- /commands/lint
- /commands/artifact-resolve
- /commands/docs
- /commands/diff
---
# okf:reorganize skill

Plans explicit mappings, distinguishes Markdown concepts from opaque `references/` artifacts,
and treats link rebasing as an expected referring-document change.

The workflow connects [list](../commands/list.md), [graph](../commands/graph.md),
[artifact list](../commands/artifact-list.md), [mv](../commands/mv.md),
[links](../commands/links.md), [backlinks](../commands/backlinks.md),
[validate](../commands/validate.md),
[lint](../commands/lint.md), [artifact resolve](../commands/artifact-resolve.md),
[docs](../commands/docs.md), and [diff](../commands/diff.md). Route prose changes through
[update](update.md) and subsequent review through [review-verify](review-verify.md).
