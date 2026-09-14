---
name: repair
description: Diagnose and repair an existing OKF knowledge base for v0.2 using okf doctor, preserving extensions and requiring review for semantic choices. Use for upgrade, conformance repair, doctor findings, or an older/current bundle that may break under corrected semantics.
---

# Repair or upgrade an existing OKF bundle

`okf doctor` is the deterministic compatibility authority. Do not reimplement its inventory or
checks in prompt logic. Bring the bundle to target-ready state when repairs are mechanical; stop
at an explicit user-decision boundary when meaning or identity is ambiguous.

Prefer `OKF_BUNDLE`; the preflight is `okf doctor <bundle> --target 0.2 --json`. When exact flags
or output shapes are needed, read the focused `references/cli.md`. If its generated tool version
differs from the installed binary, use that command's `--help` output as the runtime authority.

## Workflow

1. Resolve the exact bundle and establish recovery. In Git, record ref/status and require a clean
   worktree or explicit permission to include existing changes. Outside Git, do not make
   multi-file changes without a recoverable backup path. Doctor does not currently create one.
2. Run the read-only preflight above before normal bulk loading. Retain its summary and stable
   finding IDs. Classify its reported `safe`, `review`, and `manual` repair classes and separate
   advisory improvements from target-readiness blockers.
3. Present affected files, current versus target interpretation, preservation promises, proposed
   commands, and unresolved choices. Require direction for ambiguous links, missing types,
   actor/trust changes, lifecycle dates, source credibility, curated reserved-file prose, and
   computation conversion.
4. Preview allow-listed repairs with `okf doctor <bundle> --target 0.2 --fix-safe --dry-run
   --json`. Summarize the exact changes. Apply only after authorization with `okf doctor <bundle>
   --target 0.2 --fix-safe --yes --json`.
5. Apply reviewed repairs selectively with explicit `okf` mutators. The current CLI does not
   support selecting doctor finding IDs for mutation, so keep the stable IDs in the repair
   ledger and do not imply that it does. Never use broad text replacement.
6. Preserve unknown fields, custom types, bodies, source extensions (`kind`, `fingerprint`),
   legacy top-level `last_modified`, ontology rules, and unrelated prose unless a reviewed item
   names the change. Legal extensions are not cleanup targets.
7. Inventory optional supporting files with `okf artifact list <bundle>`. `references/` is an
   optional convention: Markdown files are concepts, non-Markdown files are opaque artifacts.
   Preserve artifacts by default and repair only their declaring paths. Resolve with document
   context, use bounded reads, stay within canonical bundle scope, require explicit remote policy,
   and never execute inspected code.
8. Validate after each batch: rerun doctor, then `okf validate <bundle>`. Run `okf lint <bundle>
   --fail-on never` separately so recommendations are not mistaken for conformance blockers.
9. Regenerate indexes with `okf docs <bundle> --format index` only when approved after showing
   the impact on curated prose. Recheck direct edges with `okf links <concept-id> <bundle>
   --json` and `okf backlinks <concept-id> <bundle>`; use a bounded bidirectional graph only when
   needed. Also recheck lifecycle, trust, source lineage, and exact Attested Computation
   classification against the original doctor report.
10. Run doctor a final time. A second safe-repair preview should propose no writes. Report the
    baseline/recovery trail, stable findings resolved, choices made, extensions preserved,
    remaining manual/advisory items, validation, and diff.

## Hard boundaries

- Doctor is read-only unless `--fix-safe --yes` is explicitly authorized.
- Never infer and apply a missing type from a filename; fabricate `human:` verification; change
  `stale_after` to hide expiry; choose between two plausible link interpretations; or convert a
  custom computation without a complete reviewed standard contract.
- `okf verify` is document verification, not execution attestation. The current computation
  command is inspect-only.
- Stop before changes outside the authorized bundle or when recovery is unavailable.
