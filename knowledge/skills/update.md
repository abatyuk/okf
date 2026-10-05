---
type: Skill
title: okf:update skill
description: Separately triages lifecycle expiry, standard source evidence, fingerprint drift, and content changes, then reconciles concepts without overstating refresh.
trigger: resync concepts after source changes
sources:
- resource: plugins/okf/skills/update/references/cli.md
  kind: git-path
  fingerprint:
    blob_sha: 1e7c90b6402eb1049ab473676c1854b36839c553
- resource: plugins/okf/skills/update/SKILL.md
  kind: git-path
  fingerprint:
    blob_sha: 4bfb51c82066ffea9f0dfc772ece8afbc201a686
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/update/SKILL.md
- resource: plugins/okf/skills/ontology/references/structured-authoring.md
  kind: git-path
  fingerprint:
    blob_sha: a05ec02fcc363f85ee522cef43b0662a0d05b256
uses:
- /commands/catalog
- /commands/stale
- /commands/affected
- /commands/diff
- /commands/refresh
- /commands/edit
- /commands/show
- /commands/links
- /commands/artifact-resolve
- /commands/artifact-show
- /commands/mv
- /commands/validate
- /commands/lint
- /commands/docs
last_modified: 2026-10-05T10:16:06Z
---
# okf:update skill

Refresh updates supported fingerprint extensions only; meaningful rewrites update generation
state and require a new document review.

The workflow connects [stale](../commands/stale.md), [affected](../commands/affected.md),
[diff](../commands/diff.md), [refresh](../commands/refresh.md), [edit](../commands/edit.md),
[show](../commands/show.md), [links](../commands/links.md),
[artifact resolve](../commands/artifact-resolve.md),
[artifact show](../commands/artifact-show.md), [mv](../commands/mv.md),
[validate](../commands/validate.md), [lint](../commands/lint.md), and
[docs](../commands/docs.md). Meaningful rewrites route to
[review-verify](review-verify.md).

Use [catalog](../commands/catalog.md) inspection to establish effective identity, local locations
and interpretation settings. Additional bundle examination is explicit; report unavailable members
and keep current candidates separate from requested snapshot evidence.

Structured authoring follows the ontology skill's shared example. Use structured declaration
YAML/file inputs and bulk apply for dependent schemas; use concept `--set-yaml`, object paths,
or RFC 6902 patches for frontmatter. Review dry-run and actual diffs, preserve source lineage,
and retain meaningful-change verification consequences. Model validation does not attest claims.
