---
type: Skill
trigger: resync concepts after source changes
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
title: okf:update skill
description: Separately triages lifecycle expiry, standard source evidence, fingerprint drift, and content changes, then reconciles concepts without overstating refresh.
sources:
- resource: plugins/okf/skills/update/references/cli.md
  kind: git-path
  fingerprint:
    blob_sha: 36a13edd125265863bb477e613aedd75713f04af
- resource: plugins/okf/skills/update/SKILL.md
  kind: git-path
  fingerprint:
    blob_sha: 8cc677b94e18947e30c18d2621a87b9594471f99
- resource: https://github.com/abatyuk/okf/blob/main/plugins/okf/skills/update/SKILL.md
- resource: plugins/okf/skills/ontology/references/structured-authoring.md
  kind: git-path
  fingerprint:
    blob_sha: ca5061b432fd75493a90a7ec7be73c96d74f1cdc
last_modified: 2026-10-19T03:58:59Z
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

When outbound metadata changes, inspect `links --details --json` and compare occurrence paths,
kinds, inverse labels and attributes rather than only deduplicated targets.

Structured authoring follows the ontology skill's shared example. Use structured declaration
YAML/file inputs and bulk apply for dependent schemas; use concept `--set-yaml`, object paths,
or RFC 6902 patches for frontmatter. Review dry-run and actual diffs, preserve source lineage,
and retain meaningful-change verification consequences. Model validation does not attest claims.
