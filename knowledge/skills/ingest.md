---
type: Skill
title: okf:ingest skill
description: Researches a repository through complete source inventory and authors grounded OKF v0.2 concepts using standard provenance before drift extensions.
trigger: research a repo and ingest it as concepts
sources:
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/ingest/SKILL.md
uses:
- /commands/source-scan
- /commands/add
- /commands/edit
- /commands/links
- /commands/computation-check
- /commands/validate
- /commands/lint
- /commands/docs
- /commands/artifact-resolve
- /commands/artifact-show
---
# okf:ingest skill

Uses `source-scan`, standard source fields and footnotes, optional bounded artifacts, and exact
Attested Computation contracts without treating arbitrary code as a sanctioned computation.

The workflow connects [source-scan](../commands/source-scan.md), [add](../commands/add.md),
[edit](../commands/edit.md), [links](../commands/links.md),
[computation check](../commands/computation-check.md),
[validate](../commands/validate.md), [lint](../commands/lint.md), [docs](../commands/docs.md),
[artifact resolve](../commands/artifact-resolve.md), and
[artifact show](../commands/artifact-show.md). Use [migrate](migrate.md) for pre-written
documents and [init](init.md) for an empty bundle.
