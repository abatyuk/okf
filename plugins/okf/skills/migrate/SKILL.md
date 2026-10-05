---
name: migrate
description: Migrate documents or prior OKF material into one or more OKF v0.2 bundles with structured transformations, preserved provenance, and resumable mappings. Use for authored docs; use ingest for repository research and repair when upgrading an existing bundle in place.
---

# Migrate documents into OKF v0.2

Preserve meaning and unknown data. The CLI handles writes and validation; classification,
granularity, attribution, and unresolved legacy semantics require judgment.


## CLI compatibility preflight

Before CLI-dependent work, run `sh <this-skill-directory>/../../scripts/check-cli.sh`,
resolving the path from this installed SKILL.md, not the working directory. This read-only
check uses the plugin's `cli-compatibility.txt` range and reports the executable on PATH.
If missing, incompatible, or unable to run, stop CLI-dependent work and show its installation
instructions. Never invoke the installer, download a CLI, or update it from this skill.
An explicit user request to install should be handled separately from this workflow.
Only after this check passes may command help resolve syntax differences; help cannot waive
compatibility. Recheck if the executable or PATH changes during the task.

## CLI and conformance model

Use `OKF_BUNDLE` for one path-selected destination; for catalog migrations, pass the explicit
`--bundle-id` on each mutation and never combine it with a bundle path. A path-selected mutation
is `okf add <path> <bundle> --type <Type>`. Before using an unfamiliar command or flag, read its entry in
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
2. Resolve selected bundle identities and necessary scope, then map source documents to target IDs
   before writing. Key the migration ledger by source identity/version and destination bundle ID plus
   concept ID; identical paths in different bundles are distinct. Record planned operation,
   completion state, and recovery evidence. See the [migration example](references/multi-bundle-migration.md)
   for mapping, transformations, and interrupted reruns. Search the target for existing equivalents;
   check scan summaries/warnings before
   treating absence as established. Raise `--scan-limit` or use deliberate `--full-scan` when a
   complete duplicate check is required; result limits alone do not complete the scan. On
   reruns, skip unchanged concepts and update intended matches with `okf edit`, rather than
   inventing duplicate IDs. Resolve conflicting destinations before writing; retain the old-to-new
   mapping for links and resumable batches. Choose concept boundaries and exact type strings. An
   ontology may guide local policy, but an unknown type is valid OKF. Preserve source meaning and
   surface large splits for review.
3. Preserve affected current bytes, including uncommitted changes, before overwrites. Create
   destination concepts in small batches before checking their dependent edges; cycles can be
   linked after all participating destinations exist. Record each successful write immediately.
   On failure, reconcile actual files with the ledger before retrying: skip unchanged completed
   writes, finish pending operations, and resolve unexpected edits/collisions without overwriting
   them. Never blindly replay source additions or invent replacement IDs. Multiple bundles have
   no shared migration transaction; continue independent work only when its dependencies are clear.
   Create each concept with `okf add <path> <bundle> --type <Type> --body
   @/tmp/migrated-body.md` (body only, no frontmatter). Unreadable input leaves no skeleton. Add grounded sources with `okf edit
   <concept-id> <bundle> --add-source "resource=<path>,id=source-1"`; use `--add-source-json
   @/tmp/source.yaml` for a full source mapping. Do not invent metadata. Treat the original as a
   standard source first: every source entry needs `resource`; add a stable `id` when body footnotes
   attribute claims. Add `title`, `author`, `usage_count`, `last_modified`, and `usage_window` only
   when evidence supports them. Footnote labels must join to `sources[].id`.
4. Add `kind` and `fingerprint` only when local drift tracking is desired; label them extensions.
   `okf refresh <concept-id> <bundle> --fail-on skipped` records supported fingerprints only,
   after the sources have been checked against the migrated content. Report skipped sources. It does not rewrite
   standard source `last_modified` and cannot cure an expired `stale_after`.
5. Define an old-field → new-selector mapping before structured transformations. Record value
   types, missing/null/empty handling, list occurrence pairing, and whether each field remains
   opaque or becomes a declared concept reference. Preserve record order and duplicate occurrences
   unless an explicit semantic decision says otherwise. Do not infer references from path spelling
   or flatten independently selected target/condition lists into incorrectly paired relationships.
   Preserve unknown nested mappings and lists without flattening them into scalar `--set` values.
   Source mappings have `--add-source-json`; use `--set-yaml` for complete structured values
   or `--patch` for concrete nested edits after inspecting the original and establishing recovery.
   Preview with `--dry-run` and validate the result. For reusable nested types and semantic selectors, follow the ontology skill's
   [structured authoring example](../ontology/references/structured-authoring.md); for cross-bundle
   source mappings use the init skill's [catalog example](../init/references/multi-bundle.md).
   Preserve existing verification as historical evidence outside the active `verified`
   field if conversion changes meaning; do not create a new verification event from old evidence.
   For v0.1 material, translate `timestamp` to `generated.at` only when the producer actor is known.
   Otherwise preserve the legacy field and report the decision. Translate legacy `# Citations` into
   sources without inventing titles or authors. Preserve lifecycle evidence; absent `status` means
   stable.
6. Reconstruct links from the final qualified destination map: rebase body links, standard source
   resources, and declared nested references from each new declaring document. Preserve fragments
   and source attribution IDs. Keep original-document provenance separate from a relationship to
   a migrated counterpart; do not silently replace the former with the latter. For `bundle_ref`,
   update destination identity/root-relative path only where that source is intentionally remapped,
   and ensure its ordinary resource addresses the same material. Preserve snapshot expectations;
   report mismatched/unavailable expectations and current candidates separately, never accept a
   substitution or refresh a baseline just to clear findings. Unknown path-like strings stay opaque.
   Single-bundle move tooling does not perform these nested/cross-bundle rewrites. Use ontology
   references only as additional local modeling. Check migrated edges with
   `okf links <concept-id> <bundle> --json`;
   inspect wider impact with `okf graph <bundle> --root <concept-id> --direction both --depth <N>
   --format mermaid`. Graph depth requires a root.
7. Validate every destination separately with `okf validate <bundle>`; run advisory
   `okf lint <bundle> --fail-on never` and edge checks with the explicit combined examination scope.
   Reconcile every source against completed destinations, intentional splits/merges, skips, and
   unresolved rows; raw source and target counts need not match. Compare representative transformed
   nested records and all exceptional cases for types, unknown extensions, source joins, paired
   relationship conditions, and preserved expectations. Inspect complete inventories and incomplete
   lint/expansion diagnostics before claiming coverage. Conformance alone does not establish
   migration fidelity. Review the combined diff and record scoped acceptance evidence in the ledger.
   Regenerate indexes with `okf docs <bundle> --format index` only for generated index bodies or an already-authorized
   replacement. It replaces index prose throughout the bundle; preserve curated indexes otherwise.
   Validate again after generating indexes. Unresolved optional evidence belongs in the report, not
   as a fabricated value.

When local fingerprint tracking is enabled with a source `kind`, review its evidence and run
`okf refresh <concept-id> <bundle> --fail-on skipped` to record the baseline. `okf stale <bundle>
--fail-on any` distinguishes `unrecorded` (no baseline), `missing` (cannot fingerprint), and
`drifted` (changed baseline). Standard kind-less provenance is not fingerprint-checked. Inspect
findings even when a command succeeds: stale defaults to a successful exit on findings, and
`lint --fail-on never` is reporting only. Use `okf lint <bundle> --fail-on warn` when source warnings
must fail a health gate; conformance validation alone does not establish source health.

## Sources and artifacts

`references/` is optional; its Markdown files are concepts and other files are artifacts. Keep
original provenance and mirror material only within the requested scope. Use `okf artifact resolve
<resource> <bundle> --from <concept-id>` for internal resources. Before recording repository paths
or reading external sources, consult the path-namespace and artifact-read guidance in
`references/cli.md`; a fingerprint path need not be an artifact path. Inspection never authorizes
execution.

For an authorized artifact copy or authored schema, use `okf artifact put
contracts/x/references/schema.json @/tmp/schema.json <bundle>`. The destination is bundle-relative;
the input file is relative to cwd. Creation is the default; use `--replace` only for an intended
replacement. Writes preserve recorded fingerprints, so review citing concepts before refreshing
them. This command does not execute artifacts or grant permission to copy external material.

Migration does not establish verification. Report migrated/skipped concepts, source joins, preserved
extensions, unresolved mappings, per-bundle validation and scoped fidelity checks, and the ledger
location/completion state for resumption. Include lifecycle or artifact details only when
relevant; route a subsequent source-backed document review to `review-verify`.

## Bundle identity and examination scope

Keep bundle paths and catalog IDs distinct. `OKF_BUNDLE` always selects a path; use the
`--bundle-id <id>` for a registered identity. When a catalog is configured, use `okf catalog` to inspect effective local
registrations, overrides, availability, and per-bundle settings. Catalog paths are relative to
`okf.toml`, locations are relative to the catalog, and machine-local overrides change effective
locations rather than accepting different snapshots. Inspect effective settings before relying on
a per-bundle ontology or view. Equal or nested registered roots are configuration errors.

Registration alone does not authorize catalog-wide traversal. Start with the selected bundle;
request additional bundles explicitly with repeatable `--scope-bundle <id>` or deliberate
`--catalog-scope` when the task requires them. Report requested and examined
scope and unavailable members. Out-of-scope references remain unknown. Ordinary relative paths
retain their meaning across registered boundaries; distributing one bundle alone can break them.
Never infer another bundle from a matching filename or title.
