---
name: update
description: Reconcile OKF concepts after source or concept changes using lifecycle, source metadata, fingerprint drift, affected analysis, and git diff. Use for documentation sync; refresh fingerprints only after review and route meaningful rewrites to re-verification.
---

# Update concepts after changes

Prefer `OKF_BUNDLE` for a workflow. Fully qualified examples include `okf diff <git-ref> <bundle>`
and `okf affected <bundle> --changed <resource>`. Before using an unfamiliar command or flag, read
its entry in `references/cli.md`. If the installed version differs or rejects documented syntax, use
that command's `--help` output as the runtime authority.

## Detect and classify

Use the relevant detectors and keep their meanings separate:

- `okf stale <bundle> --fail-on any` reports expiry and source findings: `unrecorded` means no
  baseline, `missing` means fingerprinting failed, and `drifted` means a recorded baseline changed.
  Without `--fail-on any`, findings still exit successfully. Standard kind-less provenance and
  top-level resources are not automatically fingerprinted.
- Standard source `last_modified`, credibility, and usage signals inform review but are not the same
  as local fingerprint drift.
- `okf affected <bundle> --changed <link-or-resource>` follows corrected body and standard source
  graph edges; add `--transitive --depth <N>` when needed.
- `okf diff <git-ref> <bundle>` reports concept-level change versus a Git baseline.

Triage findings as lifecycle expiry, standard source evidence change, tool fingerprint drift, or
actual concept-content change. A concept may have more than one.

## Reconcile

1. Resolve bundle identity and the requested consumer scope before affected analysis; use detailed
   semantic occurrences for nested relationships and retain unknown consumers outside scope. Inspect the concept with targeted `okf show` reads and the changed evidence. For a path-valued
   source, use `okf artifact resolve <resource> <bundle> --from <concept-id>` and bounded `okf
   artifact show`; use `okf show` when it resolves to a concept. Use `okf links <concept-id>
   <bundle> --json` when the concept's direct outbound relationships may have changed.
2. If meaning changed, supply an explicit edit operation: `okf edit <concept-id> <bundle> --set
   "description=Updated summary"` for a scalar, `okf edit <concept-id> <bundle> --set-body
   @/tmp/revised-body.md` for the whole Markdown body, or `okf edit <concept-id> <bundle>
   --set-section "Heading" @/tmp/section.md` for a section. Section edits take two values;
   body/section files contain prose, not frontmatter. A bare edit supplies no content change. The
   CLI updates existing `generated.at` and invalidates verification for meaningful edits; report the
   need for `review-verify`. For nested data use `okf edit <concept-id> <bundle> --set-yaml
   "details=@/tmp/details.yaml"`, `okf edit <concept-id> <bundle> --set-path "deadline.within=72"`,
   or `okf edit <concept-id> <bundle> --patch @/tmp/changes.yaml --dry-run`. Follow the ontology
   skill's [structured authoring example](../ontology/references/structured-authoring.md); review
   replacement versus nested-edit semantics and preserve provenance.
3. After checking changed sources against the final content, run `okf refresh <concept-id> <bundle>
   --fail-on skipped` if local fingerprints are in use. This applies after a substantive rewrite as
   well as after confirming unchanged content. Never refresh an unreviewed source merely to clear
   drift; refresh operates on the concept's supported sources. Inspect skipped/error results and
   keep unresolved sources in the report. Refresh updates the fingerprint extension only. It does
   not alter standard `sources[].last_modified`, regenerate content, change status, or cure an
   expired `stale_after`.
4. Change `stale_after` or lifecycle status only from evidence and an authorized lifecycle decision;
   an existing user instruction can supply that decision. These are distinct from refresh.
5. If a concept moved, use `okf mv <old-id> <new-id> <bundle>`. When an opaque artifact moves,
   update its declaring standard path field and preserve provenance; never execute it.
6. Re-run stale/affected checks, `okf validate <bundle>`, and advisory `okf lint <bundle> --fail-on
   never`. Regenerate indexes with `okf docs <bundle> --format index` only when needed and their
   bodies are generated or replacement is already authorized; preserve curated prose otherwise.
   Validate after index generation. Inspect lint findings despite its reporting-only exit; use
   `okf lint <bundle> --fail-on warn` for a source-health gate. Missing local sources are errors,
   and unrecorded fingerprints are warnings by default; neither is repaired by conformance validation.

For a phrase correction, use `okf edit <concept-id> <bundle> --replace "Old phrase" "New phrase"`;
exactly one literal body match is required. Use `--all` only when every occurrence should change.
Rename a uniquely matched heading with `okf edit <concept-id> <bundle> --rename-section
"Old heading" "New heading"`; its content and level are preserved. Failed matching leaves the
file unchanged. Use `okf show <concept-id> <bundle> --body` if a full body is needed for editing,
without splitting display headers or frontmatter delimiters.

## Source access

`references/` is optional; Markdown files are concepts and other files are artifacts. Keep declaring
context on artifact reads. For repository-relative fingerprints, external source access, or
truncation, use the artifact sections of `references/cli.md`. Existing authorization to inspect a
source counts; do not bypass bundle containment. Reading code does not authorize execution.

For an authorized artifact copy or authored schema, use `okf artifact put
contracts/x/references/schema.json @/tmp/schema.json <bundle>`. The destination is bundle-relative;
the input file is relative to cwd. Creation is the default; use `--replace` only for an intended
replacement. Writes preserve recorded fingerprints, so review citing concepts before refreshing
them. This command does not execute artifacts or grant permission to copy external material.

Report changes, validation, and remaining evidence gaps. Include fingerprint-only refreshes,
unchanged expiry, and verification consequences when relevant to this update.

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

## Scoped reconciliation

Use explicit examination scope for cross-bundle affected analysis. Record which consumers remain
outside scope; a clean result is not a global backlink inventory. Check both ordinary source
resources and optional `bundle_ref` metadata, retaining known mismatches rather than redirecting
resources. Review a labeled current candidate before intentionally changing an expectation;
retrieval alone never accepts it. Snapshot resolution and fingerprint drift are separate findings.

Re-run nested metadata and index-coverage lint after metadata or layout changes. Lint and query
operations do not rewrite curated indexes. Use the declared selectors to locate affected records;
use object paths or concrete JSON Pointer patches for writes, rather than list query selectors. Review the combined repository
diff when changes span bundles; there is no automatic cross-bundle rename transaction.

## Source baseline lint configuration

`source-unrecorded` defaults to `warn`. If the user wants a different policy, configure
`source_unrecorded = "off"`, `"info"`, `"warn"`, or `"error"` under
`[bundle_settings.default.lint]` in `okf.toml` (or `[bundle_settings."<id>".lint]`
for a named bundle). `off` suppresses this lint finding only; missing-source errors
(default `error`, independently configurable via `source_missing`) and `stale` findings remain active. Do not suppress findings merely to pass a gate.

All lint rule settings accept `off`, `info`, `warn`, or `error` in the same table.
Defaults: `broken_link` and `source_missing` are `error`; `missing_title`, `spec_v02`
(finding rule `okf-v02`), `ontology_violation`, `source_unrecorded`, and `index_coverage`
are `warn`; `missing_description` and `orphan` are `info`. Omitted settings keep these
severities. Turning a lint rule off does not change conformance validation or stale checks.
