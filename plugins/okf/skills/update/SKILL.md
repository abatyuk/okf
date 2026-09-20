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

- `okf stale <bundle>` reports `stale_after` expiry and supported source fingerprint drift. It does
  not claim every top-level resource is fingerprinted.
- Standard source `last_modified`, credibility, and usage signals inform review but are not the same
  as local fingerprint drift.
- `okf affected <bundle> --changed <link-or-resource>` follows corrected body and standard source
  graph edges; add `--transitive --depth <N>` when needed.
- `okf diff <git-ref> <bundle>` reports concept-level change versus a Git baseline.

Triage findings as lifecycle expiry, standard source evidence change, tool fingerprint drift, or
actual concept-content change. A concept may have more than one.

## Reconcile

1. Inspect the concept with targeted `okf show` reads and the changed evidence. For a path-valued
   source, use `okf artifact resolve <resource> <bundle> --from <concept-id>` and bounded `okf
   artifact show`; use `okf show` when it resolves to a concept. Use `okf links <concept-id>
   <bundle> --json` when the concept's direct outbound relationships may have changed.
2. If meaning changed, supply an explicit edit operation: `okf edit <concept-id> <bundle> --set
   "description=Updated summary"` for a scalar, `okf edit <concept-id> <bundle> --set-body
   @/tmp/revised-body.md` for the whole Markdown body, or `okf edit <concept-id> <bundle>
   --set-section "Heading" @/tmp/section.md` for a section. Section edits take two values;
   body/section files contain prose, not frontmatter. A bare edit supplies no content change. The
   CLI updates existing `generated.at` and invalidates verification for meaningful edits; report the
   need for `review-verify`.
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
   Validate after index generation.

## Source access

`references/` is optional; Markdown files are concepts and other files are artifacts. Keep declaring
context on artifact reads. For repository-relative fingerprints, external source access, or
truncation, use the artifact sections of `references/cli.md`. Existing authorization to inspect a
source counts; do not bypass bundle containment. Reading code does not authorize execution.

Report changes, validation, and remaining evidence gaps. Include fingerprint-only refreshes,
unchanged expiry, and verification consequences when relevant to this update.
