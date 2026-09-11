---
type: Skill
title: okf:reorganize skill
description: Moves concepts while rebasing body links and standard source resources, preserving opaque artifacts and validating regenerated indexes.
trigger: restructure the bundle layout
uses: [/commands/mv, /commands/backlinks, /commands/artifact-list, /commands/diff]
---
# okf:reorganize skill

Plans explicit mappings, distinguishes Markdown concepts from opaque `references/` artifacts,
and treats link rebasing as an expected referring-document change.
