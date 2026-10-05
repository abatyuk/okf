---
type: Skill
title: okf:ingest skill
description: Researches a repository through complete source inventory and authors grounded OKF v0.2 concepts using standard provenance before drift extensions.
trigger: research a repo and ingest it as concepts
sources:
- resource: plugins/okf/skills/ingest/references/cli.md
  kind: git-path
  fingerprint:
    blob_sha: dec5c8a9a8b552050f0b9377387e78763ea31e48
- resource: plugins/okf/skills/ingest/SKILL.md
  kind: git-path
  fingerprint:
    blob_sha: e2967e7bb3c022019ec02cac654346cf33566fc8
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/ingest/SKILL.md
- resource: plugins/okf/skills/ontology/references/structured-authoring.md
  kind: git-path
  fingerprint:
    blob_sha: a05ec02fcc363f85ee522cef43b0662a0d05b256
uses:
- /commands/catalog
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
last_modified: 2026-10-05T10:16:06Z
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

Use [catalog](../commands/catalog.md) inspection to establish effective identity, local locations
and interpretation settings. Additional bundle examination is explicit; report unavailable members
and keep current candidates separate from requested snapshot evidence.

Structured authoring follows the ontology skill's shared example. Use structured declaration
YAML/file inputs and bulk apply for dependent schemas; use concept `--set-yaml`, object paths,
or RFC 6902 patches for frontmatter. Review dry-run and actual diffs, preserve source lineage,
and retain meaningful-change verification consequences. Model validation does not attest claims.
