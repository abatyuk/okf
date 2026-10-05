---
name: repair
description: Diagnose and repair an existing OKF knowledge base for v0.2 using okf doctor, preserving extensions and requiring review for semantic choices. Use for upgrade, conformance repair, doctor findings, or an older/current bundle that may break under corrected semantics.
---

# Repair or upgrade an existing OKF bundle

`okf doctor` is the deterministic compatibility authority. Do not reimplement its inventory or
checks in prompt logic. Bring the bundle to target-ready state when repairs are mechanical; stop at
an explicit user-decision boundary when meaning or identity is ambiguous.

Prefer `OKF_BUNDLE`; the preflight is `okf doctor <bundle> --target 0.2 --json`. Before using an
unfamiliar command or flag, read its entry in `references/cli.md`. If the installed version differs
or rejects documented syntax, use that command's `--help` output as the runtime authority.

## Workflow

1. Resolve the exact bundle and effective settings and run the read-only preflight above before
   normal bulk loading. Classify catalog/ontology configuration failures separately from document
   conformance; inspect each affected bundle with its own settings. For nested ontology repairs,
   use the ontology skill's [structured authoring example](../ontology/references/structured-authoring.md)
   after the recovery step below.
   Diagnosis does not require a clean worktree, backup, or write approval.
2. Before mutation, record Git status and preserve the affected files' current contents so user
   edits can be recovered; HEAD alone does not preserve uncommitted work. Unrelated dirty files need
   not block repair. Outside Git, create a scoped recovery copy before multi-file changes. Doctor
   does not create backups. Retain its summary and stable finding IDs. Classify its reported `safe`,
   `review`, and `manual` repair classes and separate advisory improvements from target-readiness
   blockers.
3. Present affected files, current versus target interpretation, preservation promises, proposed
   commands, and unresolved choices. Existing authorization counts for the requested repairs. Ask
   only when ambiguity remains about identity/meaning, unsupported trust claims, or changes beyond
   that scope; do not re-confirm a type or lifecycle decision the user already supplied.
4. Preview allow-listed repairs with `okf doctor <bundle> --target 0.2 --fix-safe --dry-run --json`.
   Summarize the exact changes. Apply within existing authorization with `okf doctor <bundle>
   --target 0.2 --fix-safe --yes --json`.
5. Apply reviewed repairs selectively, for example `okf edit <concept-id> <bundle> --set
   "type=<reviewed-type>"` for an agreed missing type, or `okf edit <concept-id> <bundle>
   --set-section "Heading" @/tmp/repaired-section.md` for reviewed section content. Read the edit
   reference for other operations; section edits require both heading and text. The current CLI does
   not support selecting doctor finding IDs for mutation, so keep the stable IDs in the repair
   ledger and do not imply that it does. If malformed YAML prevents loading with `okf edit`, inspect
   the specific file directly and make a minimal syntax repair within the authorized scope,
   preserving unknown fields and prose. Re-run doctor/validation before further mutators. Do not
   guess missing semantics or perform broad replacements.
6. Preserve unknown fields, custom types, bodies, source extensions (`kind`, `fingerprint`), legacy
   top-level `last_modified`, ontology rules, and unrelated prose unless a reviewed item names the
   change. Legal extensions are not cleanup targets.
7. Inventory optional supporting files with `okf artifact list <bundle>`. `references/` is an
   optional convention: Markdown files are concepts, non-Markdown files are opaque artifacts.
   Preserve artifacts by default; change their content only when included in the authorized repair.
   Resolve with document context and use the artifact reference for bounded/external reads. Never execute inspected code.
8. Validate after each batch: rerun doctor, then `okf validate <bundle>`. Run `okf lint <bundle>
   --fail-on never` separately so recommendations are not mistaken for conformance blockers.
9. Regenerate indexes with `okf docs <bundle> --format index` only when their bodies are generated
   or replacement is already authorized; otherwise preserve curated prose. The command replaces
   index bodies throughout the bundle. Recheck direct edges with `okf links <concept-id> <bundle>
   --json` and `okf backlinks <concept-id> <bundle>`; use a bounded bidirectional graph only when
   needed. Also recheck lifecycle, trust, source lineage, and exact Attested Computation
   classification against the original doctor report.
10. Run doctor a final time. A second safe-repair preview should propose no writes. Report the
    baseline/recovery trail, stable findings resolved, choices made, extensions preserved, remaining
    manual/advisory items, validation, and diff.

## Hard boundaries

- Doctor is read-only unless `--fix-safe --yes` is used within the authorized repair scope.
- Never infer and apply a missing type from a filename; fabricate `human:` verification; change
  `stale_after` to hide expiry; choose between two plausible link interpretations; or convert a
  custom computation without a complete reviewed standard contract.
- `okf verify` is document verification, not execution attestation. The current computation command
  is inspect-only.
- Stop affected writes when scope or recovery is unresolved; continue read-only diagnosis and
  independent authorized repairs.

## Focused repairs

For a phrase correction, use `okf edit <concept-id> <bundle> --replace "Old phrase" "New phrase"`;
exactly one literal body match is required. Use `--all` only when every occurrence should change.
Rename a uniquely matched heading with `okf edit <concept-id> <bundle> --rename-section
"Old heading" "New heading"`; its content and level are preserved. Failed matching leaves the
file unchanged. Use `okf show <concept-id> <bundle> --body` if a full body is needed for editing,
without splitting display headers or frontmatter delimiters.

If an authorized repair includes artifact content, use `okf artifact put
contracts/x/references/schema.json @/tmp/repaired-schema.json <bundle> --replace`. Preserve
artifacts otherwise. The destination is bundle-relative and the input is cwd-relative. Recorded
fingerprints remain unchanged; review citing concepts before refreshing through the update workflow.
Inspect `source-unrecorded` warnings and `source-missing` errors even with reporting-only lint;
use `okf lint <bundle> --fail-on warn` when a source-health gate is requested.

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

## Local configuration and navigation findings

Separate catalog conflicts, unavailable roots, snapshot mismatches, malformed ontology definitions,
and incomplete recursive checks from OKF conformance failures. Index coverage is advisory and
read-only; missing navigation links do not make an optional index mandatory. Preserve curated
prose when repairing indexes, and do not refresh source baselines to clear unresolved snapshots.

When a read fails to parse concept YAML, use the file paths in its error to locate the failing documents. Full bundle reads report all malformed files they examined; bounded queries report only examined files. Use `okf validate <bundle> --json` for separate per-file conformance findings.
