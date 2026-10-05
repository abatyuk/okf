---
type: Skill
title: okf:ontology skill
description: Manages an optional advisory ontology while keeping portable OKF fields and exact Attested Computation semantics separate from custom rules.
trigger: author and manage the ontology
sources:
- resource: plugins/okf/skills/ontology/references/cli.md
  kind: git-path
  fingerprint:
    blob_sha: 28de3d63d8e2851ff0812d38cacd9a6d1f9c01ee
- resource: plugins/okf/skills/ontology/SKILL.md
  kind: git-path
  fingerprint:
    blob_sha: 28bc0ec6b7e320ec05975fcab74582840cc04e30
- resource: https://github.com/abatyuk/okf/blob/5cf5e6773e137d30f61d21e6a680aec0f8478448/plugins/okf/skills/ontology/SKILL.md
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
- /commands/ontology-list
- /commands/ontology-show
- /commands/lint
- /commands/ontology-add
- /commands/ontology-update
- /commands/ontology-remove
- /commands/validate
- /commands/artifact-resolve
last_modified: 2026-10-05T10:16:06Z
---
# okf:ontology skill

Treats `ontology.yaml` as a tool-local sidecar that cannot make an unknown type nonconformant or
rename another type into a standard Attested Computation.

The workflow connects [ontology list](../commands/ontology-list.md),
[ontology show](../commands/ontology-show.md), [lint](../commands/lint.md),
[ontology add](../commands/ontology-add.md), [ontology update](../commands/ontology-update.md),
[ontology remove](../commands/ontology-remove.md), [validate](../commands/validate.md), and
[artifact resolve](../commands/artifact-resolve.md).

Use [catalog](../commands/catalog.md) inspection to establish effective identity, local locations
and interpretation settings. Additional bundle examination is explicit; report unavailable members
and keep current candidates separate from requested snapshot evidence.

Structured authoring follows the ontology skill's shared example. Use structured declaration
YAML/file inputs and bulk apply for dependent schemas; use concept `--set-yaml`, object paths,
or RFC 6902 patches for frontmatter. Review dry-run and actual diffs, preserve source lineage,
and retain meaningful-change verification consequences. Model validation does not attest claims.
