---
type: Skill
title: okf:repair skill
description: Uses okf doctor to diagnose and safely repair an existing bundle for v0.2 while preserving legal extensions and stopping for semantic choices.
trigger: repair, upgrade, or doctor an existing bundle
sources:
- resource: plugins/okf/skills/repair/references/cli.md
  kind: git-path
  fingerprint:
    blob_sha: 94828fb076d1c8fb4fe6c54b4c98c4baa51e7561
- resource: plugins/okf/skills/repair/SKILL.md
  kind: git-path
  fingerprint:
    blob_sha: 1a02ac8b641226442633bf1911f7d9158009db49
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/repair/SKILL.md
- resource: plugins/okf/skills/ontology/references/structured-authoring.md
  kind: git-path
  fingerprint:
    blob_sha: a05ec02fcc363f85ee522cef43b0662a0d05b256
uses:
- /commands/catalog
- /commands/doctor
- /commands/validate
- /commands/lint
- /commands/artifact-list
- /commands/links
- /commands/backlinks
- /commands/graph
- /commands/docs
last_modified: 2026-10-05T10:16:06Z
---
# okf:repair skill

Runs a read-only compatibility preflight, previews allow-listed repairs, preserves unknown data,
and leaves ambiguous identity, lifecycle, link, and computation decisions to explicit review.

The workflow connects [doctor](../commands/doctor.md), [validate](../commands/validate.md),
[lint](../commands/lint.md), [artifact list](../commands/artifact-list.md), and
[links](../commands/links.md), [backlinks](../commands/backlinks.md),
[graph](../commands/graph.md), and [docs](../commands/docs.md).

Use [catalog](../commands/catalog.md) inspection to establish effective identity, local locations
and interpretation settings. Additional bundle examination is explicit; report unavailable members
and keep current candidates separate from requested snapshot evidence.

Structured authoring follows the ontology skill's shared example. Use structured declaration
YAML/file inputs and bulk apply for dependent schemas; use concept `--set-yaml`, object paths,
or RFC 6902 patches for frontmatter. Review dry-run and actual diffs, preserve source lineage,
and retain meaningful-change verification consequences. Model validation does not attest claims.
