---
type: Skill
title: okf:retrieval skill
description: Answers from a bundle through progressive disclosure, source lineage, bounded artifact retrieval, and explicit lifecycle, verification, and runtime caveats.
trigger: answer questions from a bundle
sources:
- resource: plugins/okf/skills/retrieval/references/cli.md
  kind: git-path
  fingerprint:
    blob_sha: 022d50d9673aa8a414d8d59c0ef64baf6a9dc7e9
- resource: plugins/okf/skills/retrieval/SKILL.md
  kind: git-path
  fingerprint:
    blob_sha: 6bfd416438c83207f47a082ac870c7e21d4ab0c1
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/retrieval/SKILL.md
uses:
- /commands/catalog
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
last_modified: 2026-10-04T20:39:29Z
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

Use [catalog](../commands/catalog.md) inspection to establish effective identity, local locations
and interpretation settings. Additional bundle examination is explicit; report unavailable members
and keep current candidates separate from requested snapshot evidence.
