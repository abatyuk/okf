---
type: Skill
title: okf:infer-ontology skill
description: Infers advisory custom fields and references from support counts while excluding standard OKF families and preserving exact unknown types.
trigger: reverse-engineer an ontology from a bundle
sources:
- resource: plugins/okf/skills/infer-ontology/references/cli.md
  kind: git-path
  fingerprint:
    blob_sha: 5be6f0a9c9e41eb88385c23a1e50a95a6ada48d8
- resource: plugins/okf/skills/infer-ontology/SKILL.md
  kind: git-path
  fingerprint:
    blob_sha: bb7b23e668ec65768b1b882d44a1f8cca2761610
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/infer-ontology/SKILL.md
- resource: plugins/okf/skills/ontology/references/structured-authoring.md
  kind: git-path
  fingerprint:
    blob_sha: a05ec02fcc363f85ee522cef43b0662a0d05b256
uses:
- /commands/ontology-apply
- /commands/ontology-field-type-add
- /commands/ontology-field-type-update
- /commands/ontology-field-type-remove
- /commands/catalog
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
last_modified: 2026-10-05T10:16:06Z
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

Use [catalog](../commands/catalog.md) inspection to establish effective identity, local locations
and interpretation settings. Additional bundle examination is explicit; report unavailable members
and keep current candidates separate from requested snapshot evidence.

Structured authoring follows the ontology skill's shared example. Use structured declaration
YAML/file inputs and bulk apply for dependent schemas; use concept `--set-yaml`, object paths,
or RFC 6902 patches for frontmatter. Review dry-run and actual diffs, preserve source lineage,
and retain meaningful-change verification consequences. Model validation does not attest claims.
