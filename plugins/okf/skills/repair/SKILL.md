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

1. Resolve the exact bundle and run the read-only preflight above before normal bulk loading.
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
   Preserve artifacts by default and repair only their declaring paths. Resolve with document
   context and use the artifact reference for bounded/external reads. Never execute inspected code.
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
