---
name: migrate
description: Convert existing documents or a prior OKF representation into faithful OKF v0.2 concepts with standard provenance. Use for authored docs; use ingest for repository research and repair when upgrading an existing bundle in place.
---

# Migrate documents into OKF v0.2

Preserve meaning and unknown data. The CLI handles writes and validation; classification,
granularity, attribution, and unresolved legacy semantics require judgment.

## CLI and conformance model

Prefer `OKF_BUNDLE` for a multi-step workflow. A fully qualified mutation is `okf add <path>
<bundle> --type <Type>`. When exact flags or output shapes are needed, read the focused
`references/cli.md`. If its generated tool version differs from the installed binary, use that
command's `--help` output as the runtime authority.

Conformance requires parseable frontmatter, a non-empty `type`, and valid reserved files.
`title`, `description`, applicable `resource`, `tags`, structural Markdown, and absolute
bundle-relative concept links are recommended. Provenance, trust, lifecycle, and computation
families are optional. Ontology, source `kind`/`fingerprint`, and sync metadata are extensions;
do not turn them into conformance requirements.

## Workflow

1. Inventory external source documents with normal source tools or `okf source-scan
   <directory>`. Inspect an existing target bundle only through targeted `okf` queries.
2. Choose concept boundaries and exact type strings. An ontology may guide local policy, but an
   unknown type is valid OKF. Preserve source meaning and surface large splits for review.
3. Create each concept and migrate its body without inventing metadata. Treat the original as a
   standard source first: every source entry needs `resource`; add a stable `id` when body
   footnotes attribute claims. Add `title`, `author`, `usage_count`, `last_modified`, and
   `usage_window` only when evidence supports them. Footnote labels must join to `sources[].id`.
4. Add `kind` and `fingerprint` only when local drift tracking is desired; label them extensions.
   `okf refresh <concept-id> <bundle>` records supported fingerprints only. It does not rewrite
   standard source `last_modified` and cannot cure an expired `stale_after`.
5. For v0.1 material, translate `timestamp` to `generated.at` only when the producer actor is
   known. Otherwise preserve the legacy field and report the decision. Translate legacy
   `# Citations` into sources without inventing titles or authors. Preserve lifecycle evidence;
   absent `status` means stable.
6. Reconnect portable body links and standard internal source lineage. Use ontology references
   only as additional local modeling. Check migrated edges with `okf links <concept-id> <bundle>
   --json`; inspect wider impact with a depth-bounded graph rather than loading the full graph.
7. Run `okf validate <bundle>`, advisory `okf lint <bundle> --fail-on never`, and index generation.
   Unresolved optional evidence belongs in the report, not as a fabricated value.

## `references/` and artifacts

Use `references/` only for user-authorized durable local material. It is optional. Non-reserved
Markdown beneath it is an ordinary concept; non-Markdown content is an opaque artifact. Keep the
original external provenance and do not mirror by default. Resolve document-relative and
bundle-relative resources with `okf artifact resolve <resource> <bundle> --from <concept-id>`;
use `okf artifact show` only for the smallest useful line/byte range. Scope descriptors are
provenance descriptions, not missing files. Keep local reads within the canonical bundle,
require explicit policy for remote retrieval, and never execute inspected code.

Migration does not establish verification. Report migrated concepts, source joins, preserved
legacy fields/extensions, lifecycle choices, validation, and advisory findings; route actual
document review to `review-verify`.
