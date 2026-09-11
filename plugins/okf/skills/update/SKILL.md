---
name: update
description: Reconcile OKF concepts after source or concept changes using lifecycle, source metadata, fingerprint drift, affected analysis, and git diff. Use for documentation sync; refresh fingerprints only after review and route meaningful rewrites to re-verification.
---

# Update concepts after changes

Read the generated `okf-cli-reference.md` in this skill directory. Prefer `OKF_BUNDLE` for a
workflow. Fully qualified examples include `okf diff <git-ref> <bundle>` and
`okf affected <bundle> --changed <resource>`.

## Detect and classify

Use the relevant detectors and keep their meanings separate:

- `okf stale <bundle>` reports `stale_after` expiry and supported source fingerprint drift. It
  does not claim every top-level resource is fingerprinted.
- Standard source `last_modified`, credibility, and usage signals inform review but are not the
  same as local fingerprint drift.
- `okf affected <bundle> --changed <link-or-resource>` follows corrected body and standard source
  graph edges; add `--transitive --depth <N>` when needed.
- `okf diff <git-ref> <bundle>` reports concept-level change versus a Git baseline.

Triage findings as lifecycle expiry, standard source evidence change, tool fingerprint drift,
or actual concept-content change. A concept may have more than one.

## Reconcile

1. Inspect the concept with targeted `okf show` reads and the changed evidence. For a path-valued
   source, use `okf artifact resolve <resource> <bundle> --from <concept-id>` and bounded
   `okf artifact show`; use `okf show` when it resolves to a concept. Use `okf links <concept-id>
   <bundle> --json` when the concept's direct outbound relationships may have changed.
2. If meaning changed, update body/frontmatter with `okf edit <concept-id> <bundle>`. The CLI
   updates existing `generated.at` and invalidates verification for meaningful edits; report the
   need for `review-verify`.
3. If only the local fingerprint baseline changed and the content remains correct, run
   `okf refresh <concept-id> <bundle>`. Refresh updates the fingerprint extension only. It does
   not alter standard `sources[].last_modified`, regenerate content, change status, or cure an
   expired `stale_after`.
4. Change `stale_after` or lifecycle status only from evidence and with user judgment. These are
   distinct from refresh.
5. If a concept moved, use `okf mv <old-id> <new-id> <bundle>`. When an opaque artifact moves,
   update its declaring standard path field and preserve provenance; never execute it.
6. Re-run stale/affected checks, `okf validate <bundle>`, and advisory `okf lint <bundle>
   --fail-on never`. Regenerate indexes only when appropriate.

## `references/` safety

`references/` is optional. Markdown there is concept content; non-Markdown is opaque. Resolve
document-relative paths with declaring context, distinguish scope/missing/blocked results, and
stay inside the canonical bundle after symlinks. Remote fetch requires explicit policy and no
ambient credentials. Reading changed computation/executor/attester code grants no execution
authority.

Report each trigger category, prose/path changes, fingerprint-only refreshes, unchanged expiry,
generation/verification consequences, validation, and unresolved decisions.
