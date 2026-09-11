---
type: Skill
title: okf:update skill
description: Separately triages lifecycle expiry, standard source evidence, fingerprint drift, and content changes, then reconciles concepts without overstating refresh.
trigger: resync concepts after source changes
sources:
- resource: https://github.com/abatyuk/okf/blob/5cf5e6773e137d30f61d21e6a680aec0f8478448/plugins/okf/skills/update/SKILL.md
uses:
- /commands/stale
- /commands/affected
- /commands/diff
- /commands/refresh
- /commands/edit
- /commands/show
- /commands/artifact-resolve
- /commands/artifact-show
- /commands/mv
- /commands/validate
- /commands/lint
- /commands/docs
---
# okf:update skill

Refresh updates supported fingerprint extensions only; meaningful rewrites update generation
state and require a new document review.

The workflow connects [stale](../commands/stale.md), [affected](../commands/affected.md),
[diff](../commands/diff.md), [refresh](../commands/refresh.md), [edit](../commands/edit.md),
[show](../commands/show.md), [artifact resolve](../commands/artifact-resolve.md),
[artifact show](../commands/artifact-show.md), [mv](../commands/mv.md),
[validate](../commands/validate.md), [lint](../commands/lint.md), and
[docs](../commands/docs.md). Meaningful rewrites route to
[review-verify](review-verify.md).
