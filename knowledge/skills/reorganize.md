---
type: Skill
title: okf:reorganize skill
description: Moves concepts while rebasing body links and standard source resources, preserving opaque artifacts and validating regenerated indexes.
trigger: restructure the bundle layout
sources:
- resource: plugins/okf/skills/reorganize/references/cli.md
  kind: git-path
  fingerprint:
    blob_sha: 3900fe3f2a6a507bbfc2a97b4088202b26b3bbe2
- resource: plugins/okf/skills/reorganize/SKILL.md
  kind: git-path
  fingerprint:
    blob_sha: 0a76e60c0c512d83818a2bf0b9f510123f607146
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/reorganize/SKILL.md
uses:
- /commands/catalog
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
last_modified: 2026-10-04T20:39:29Z
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

Use [catalog](../commands/catalog.md) inspection to establish effective identity, local locations
and interpretation settings. Additional bundle examination is explicit; report unavailable members
and keep current candidates separate from requested snapshot evidence.
