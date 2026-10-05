---
name: init
description: Start OKF v0.2 bundles and local multi-bundle catalogs, configure per-bundle settings, and scaffold concepts or exact Attested Computations. Use for new bundles or catalog setup; use migrate or ingest for existing material and repair for an existing KB.
---

# Initialize an OKF bundle

Use the CLI for writes and checks. Use the domain, scope, and modeling choices already supplied;
choose ordinary defaults for reversible scaffolding. Ask only about unresolved choices that
materially change the requested bundle.

## CLI and bundle access

In multi-step work, set `OKF_BUNDLE` once. One fully qualified example is `okf add notes/hello
<bundle> --type Note`. Before using an unfamiliar command or flag, read its entry in
`references/cli.md`. If the installed version differs or rejects documented syntax, use that
command's `--help` output as the runtime authority.

Inspect bundle content through `okf browse`, `okf search`, `okf list`, and targeted `okf show`. For
text search, use `okf search <bundle> --text "query terms" --limit 10`; the positional is always the
bundle path, never the query. Use `okf list <bundle>` for an unfiltered inventory. For a large
concept, use `okf show <concept-id> <bundle> --outline`, then `okf show <concept-id> <bundle>
--lines <START:END>`.

## Portable OKF versus local policy

- Required for bundle conformance: parseable frontmatter, non-empty `type`, and valid reserved
  `index.md`/`log.md` structures. Unknown custom type strings are valid.
- Recommended: `title`, `description`, applicable `resource`, `tags`, useful Markdown structure, and
  absolute bundle-relative concept links.
- Optional with defined semantics: `sources`, `generated`, `verified`, lifecycle fields, and the
  Attested Computation family.
- `ontology.yaml`, source `kind`/`fingerprint`, and sync metadata are tool extensions. Ontology lint
  is advisory and never changes OKF conformance.

## Workflow

1. Choose bundle boundaries from ownership, update/release cycles, and distribution needs; use
   directories within one bundle when these are shared. For multiple bundles, follow the
   [catalog setup example](references/multi-bundle.md) to create nonoverlapping roots, stable IDs,
   typed defaults, per-bundle settings, and optional machine-local overrides. Keep ordinary paths
   valid; identity alone does not make cross-bundle links relocatable. Run `okf init <bundle>`; it creates an empty ontology sidecar by default. Use `okf init <bundle>
   --no-ontology` when no local ontology is wanted. Confirm that its root index is structurally
   valid with `okf validate <bundle>`; an empty bundle still needs a non-empty `#` heading when
   indexed.
2. If local type rules add value, define them with `okf ontology add <name> <bundle>` and inspect
   them with `okf ontology show <name> <bundle>`. Do not present this sidecar as an OKF registry or
   requirement. Keep exact type strings; unknown types remain portable. For reusable nested types
   or semantic relationships, use the ontology skill's
   [structured authoring example](../ontology/references/structured-authoring.md); structured CLI examples
   describe the YAML inputs and bulk apply workflow. Inspect configured ontology paths before editing.
3. Create ordinary concepts with `okf add <path> <bundle> --type <Type> --body
   @/tmp/concept-body.md`. Add grounded title and description with `--title "Title"
   --description "Summary"`. The file contains Markdown body only;
   unreadable input fails before creating a skeleton. Add a source with `okf edit <concept-id>
   <bundle> --add-source
   "resource=<path>,id=source-1"`. If generation provenance is desired, pass `--generated-by
   <actor>` to add; the CLI supplies the timestamp. Never label agent-authored material
   `human:<id>`.
4. Only for a requested computation, use exact `Attested Computation` with `okf add <path> <bundle>
   --attested --runtime <runtime>`. Read the add/computation sections of `references/cli.md` for
   parameters, inline/file forms, and contract resources. Inspect with `okf computation check
   <concept-id> <bundle>`; this does not execute or attest a run.
5. Add portable Markdown links for relationships. Ontology references may supplement them.
   For cross-bundle sources, preserve the document-relative `resource` and make optional
   `bundle_ref.id`/root-relative `bundle_ref.path` identify the same material. Inspect `okf catalog`
   and check the edge with explicitly scoped `okf links` before relying on it; the setup example
   includes a complete source mapping and checks. Snapshot expectations are optional and must not
   silently replace the ordinary target.
6. Run `okf validate <bundle>`, then advisory `okf lint <bundle> --fail-on never`. Generate indexes
   with `okf docs <bundle> --format index` only when their bodies are generated content or
   replacement is already authorized; the command replaces index prose across the bundle. Preserve
   curated indexes otherwise. Confirm generated indexes with `okf browse <bundle>`.

When local fingerprint tracking is enabled with a source `kind`, review its evidence and run
`okf refresh <concept-id> <bundle> --fail-on skipped` to record the baseline. `okf stale <bundle>
--fail-on any` distinguishes `unrecorded` (no baseline), `missing` (cannot fingerprint), and
`drifted` (changed baseline). Standard kind-less provenance is not fingerprint-checked. Inspect
findings even when a command succeeds: stale defaults to a successful exit on findings, and
`lint --fail-on never` is reporting only. Use `okf lint <bundle> --fail-on warn` when source warnings
must fail a health gate; conformance validation alone does not establish source health.

## Supporting material

`references/` is optional: Markdown files there are concepts; other files are artifacts. Copy only
material within the authorized scope and preserve its provenance. Use `okf artifact list <bundle>`
to inventory its default `references/` directory, or add `--directory <directory>` for another
directory. Resolve paths with `okf artifact resolve <resource> <bundle> --from <concept-id>`;
consult the artifact sections of `references/cli.md` for bounded reads and external-source handling.
Inspection never authorizes execution.

For an authorized artifact copy or authored schema, use `okf artifact put
contracts/x/references/schema.json @/tmp/schema.json <bundle>`. The destination is bundle-relative;
the input file is relative to cwd. Creation is the default; use `--replace` only for an intended
replacement. Writes preserve recorded fingerprints, so review citing concepts before refreshing
them. This command does not execute artifacts or grant permission to copy external material.

Report what was created, validation, and relevant lint findings. Include computation or artifact
details only when those features were used.

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
