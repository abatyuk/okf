---
name: migrate
description: Convert existing documents or a prior OKF representation into faithful OKF v0.2 concepts with standard provenance. Use for authored docs; use ingest for repository research and repair when upgrading an existing bundle in place.
---

# Migrate documents into OKF v0.2

Preserve meaning and unknown data. The CLI handles writes and validation; classification,
granularity, attribution, and unresolved legacy semantics require judgment.

## CLI and conformance model

Prefer `OKF_BUNDLE` for a multi-step workflow. A fully qualified mutation is `okf add <path>
<bundle> --type <Type>`. Before using an unfamiliar command or flag, read its entry in
`references/cli.md`. If the installed version differs or rejects documented syntax, use that
command's `--help` output as the runtime authority.

Conformance requires parseable frontmatter, a non-empty `type`, and valid reserved files. `title`,
`description`, applicable `resource`, `tags`, structural Markdown, and absolute bundle-relative
concept links are recommended. Provenance, trust, lifecycle, and computation families are optional.
Ontology, source `kind`/`fingerprint`, and sync metadata are extensions; do not turn them into
conformance requirements.

## Workflow

1. Inventory external source documents with normal source tools or `okf source-scan <directory>`.
   Inspect an existing target bundle only through targeted `okf` queries.
2. Map source documents to target IDs before writing. Search the target for existing equivalents; on
   reruns, skip unchanged concepts and update intended matches with `okf edit`, rather than
   inventing duplicate IDs. Resolve conflicting destinations before writing; retain the old-to-new
   mapping for links and resumable batches. Choose concept boundaries and exact type strings. An
   ontology may guide local policy, but an unknown type is valid OKF. Preserve source meaning and
   surface large splits for review.
3. Create each concept, then write its Markdown body with `okf edit <concept-id> <bundle> --set-body
   @/tmp/migrated-body.md` (body only, no frontmatter). Add grounded sources with `okf edit
   <concept-id> <bundle> --add-source "resource=<path>,id=source-1"`; use `--add-source-json
   @/tmp/source.yaml` for a full source mapping. Do not invent metadata. Treat the original as a
   standard source first: every source entry needs `resource`; add a stable `id` when body footnotes
   attribute claims. Add `title`, `author`, `usage_count`, `last_modified`, and `usage_window` only
   when evidence supports them. Footnote labels must join to `sources[].id`.
4. Add `kind` and `fingerprint` only when local drift tracking is desired; label them extensions.
   `okf refresh <concept-id> <bundle>` records supported fingerprints only. It does not rewrite
   standard source `last_modified` and cannot cure an expired `stale_after`.
5. Preserve unknown nested mappings and lists without flattening them into scalar `--set` values.
   Source mappings have `--add-source-json`; other complex legacy data may require a narrow,
   lossless file edit after inspecting the original and establishing recovery, followed by
   validation. Preserve existing verification as historical evidence outside the active `verified`
   field if conversion changes meaning; do not create a new verification event from old evidence.
   For v0.1 material, translate `timestamp` to `generated.at` only when the producer actor is known.
   Otherwise preserve the legacy field and report the decision. Translate legacy `# Citations` into
   sources without inventing titles or authors. Preserve lifecycle evidence; absent `status` means
   stable.
6. Reconnect portable body links and standard internal source lineage. Use ontology references only
   as additional local modeling. Check migrated edges with `okf links <concept-id> <bundle> --json`;
   inspect wider impact with `okf graph <bundle> --root <concept-id> --direction both --depth <N>
   --format mermaid`. Graph depth requires a root.
7. Run `okf validate <bundle>`, advisory `okf lint <bundle> --fail-on never`, and regenerate indexes
   with `okf docs <bundle> --format index` only for generated index bodies or an already-authorized
   replacement. It replaces index prose throughout the bundle; preserve curated indexes otherwise.
   Validate again after generating indexes. Unresolved optional evidence belongs in the report, not
   as a fabricated value.

## Sources and artifacts

`references/` is optional; its Markdown files are concepts and other files are artifacts. Keep
original provenance and mirror material only within the requested scope. Use `okf artifact resolve
<resource> <bundle> --from <concept-id>` for internal resources. Before recording repository paths
or reading external sources, consult the path-namespace and artifact-read guidance in
`references/cli.md`; a fingerprint path need not be an artifact path. Inspection never authorizes
execution.

Migration does not establish verification. Report migrated/skipped concepts, source joins, preserved
extensions, unresolved mappings, and validation. Include lifecycle or artifact details only when
relevant; route a subsequent source-backed document review to `review-verify`.
