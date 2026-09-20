---
name: ingest
description: Research a repository or directory and author grounded OKF v0.2 concepts with standard provenance. Use for source-code or mixed-artifact research; use migrate for pre-written docs and init for an empty bundle.
---

# Research and ingest a repository

Use `okf source-scan <directory>` when an inventory is needed, scoped to the relevant source
directory. Use normal code-search tools on that external source, but query existing bundle content
through the CLI and retrieve only targeted slices.

## CLI and output rules

Prefer `OKF_BUNDLE` during multi-step work; one explicit form is `okf edit <concept-id> <bundle>
--add-source resource=<path>`. Before using an unfamiliar command or flag, read its entry in
`references/cli.md`. If the installed version differs or rejects documented syntax, use that
command's `--help` output as the runtime authority.

Producer rubric:

- Conformance requires parseable frontmatter, non-empty `type`, and valid reserved files.
- Recommended fields are `title`, `description`, applicable `resource`, `tags`, structured Markdown,
  and absolute bundle-relative concept links.
- Provenance, trust, lifecycle, and computation families are optional with defined semantics.
- Ontology and source `kind`/`fingerprint` are tool extensions, never conformance requirements.

## Workflow

1. Start from the requested subsystem or files. Use `okf source-scan <source-directory> --json` when
   their extent is unclear; a full repository scan is unnecessary for a narrow request. The scan
   inventories paths and sizes, not content or secret classifications. Filter its results before
   reading/storing material; exclude secrets and irrelevant generated output.
2. Research meaningful concepts rather than producing one concept per file. Ground every claim in
   inspected evidence. Use an ontology only as advisory classification; unknown types remain valid
   OKF.
3. Search the target for existing concepts using `okf search <bundle> --text "topic" --limit 10` and
   inspect plausible matches. Update an existing concept when appropriate; do not duplicate it. Stop
   research when the requested concepts and their material claims have adequate evidence; report
   unresolved points rather than expanding to unrelated subsystems. Add new concepts with `okf add
   <path> <bundle> --type <Type>`. When generation provenance is enabled, pass `--generated-by
   <actor>`; the CLI supplies the explicit-offset timestamp. Write the body after creation with `okf
   edit <concept-id> <bundle> --set-body @/tmp/concept-body.md` (Markdown body only, no
   frontmatter).
4. Add standard sources with `okf edit <concept-id> <bundle> --add-source
   "resource=<path>,id=source-1"`. For a full source mapping, use `okf edit <concept-id> <bundle>
   --add-source-json @/tmp/source.yaml`. Each entry needs `resource`; use stable `id` values and
   matching Markdown footnotes for per-claim attribution. Add credibility or usage fields only from
   evidence. `kind`/`fingerprint` may be added for local drift tracking. Having sources is a
   recommended ingest policy when known, not an OKF conformance rule.
5. Use direct Markdown links for portable relationships. Ontology reference fields may add a local
   typed view but must not replace standard links or source lineage. Spot-check authored edges with
   `okf links <concept-id> <bundle> --json` so normalized targets and missing concepts are visible
   without traversing the whole graph.
6. Only for a requested computation, use exact `Attested Computation` with `okf add <path> <bundle>
   --attested --runtime <runtime>`. Read the add/computation sections of `references/cli.md` for
   parameters, inline/file forms, and contract resources. Inspect with `okf computation check
   <concept-id> <bundle>`; this does not execute or attest a run.
7. Run `okf validate <bundle>`, advisory `okf lint <bundle> --fail-on never`, and use `okf docs
   <bundle> --format index` only when index bodies are generated or replacement is already
   authorized. It overwrites index prose throughout the bundle; preserve curated indexes otherwise.
   Validate after generating indexes.

## Source locations

`references/` is an optional location for authorized copies; Markdown files are concepts and other
files are artifacts. Preserve the external origin. Repository-relative fingerprint paths and
bundle/document-relative artifact paths use different roots: read the artifact resolution notes in
`references/cli.md` before choosing `sources[].resource` and `kind`. Use `okf artifact resolve
<resource> <bundle> --from <concept-id>` and bounded `okf artifact show` for bundle artifacts.
Inspect authorized external repository sources with normal source tools; do not change provenance or
copy them merely to bypass a blocked artifact read. Inspection is not permission to execute code.

Report created/updated/skipped concepts, source evidence, validation, and any unresolved scope.
Include optional extensions or copied artifacts only when used.
