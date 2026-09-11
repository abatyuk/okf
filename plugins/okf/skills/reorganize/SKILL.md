---
name: reorganize
description: Plan and move OKF concepts while rebasing portable body links and standard internal source resources, preserving opaque artifacts and validating indexes. Use for path/layout refactors, not content rewrites.
---

# Reorganize an OKF bundle

A concept id is its bundle-relative Markdown path. Use `okf mv`; it rewrites inbound body links
and internal standard source resources. Link rebasing legitimately edits referring documents,
so a concept diff may show add/remove or modified referrers rather than a pure native rename.

Read `okf-cli-reference.md` in this skill directory. Prefer `OKF_BUNDLE`. Correct explicit forms
include `okf mv <old-id> <new-id> <bundle>`, `okf backlinks <concept-id> <bundle>`, and
`okf links <concept-id> <bundle> --json`.

## Workflow

1. Establish a recoverable baseline and record existing validation/lint findings. Inventory
   concepts with `okf list <bundle> --json`, structure with `okf graph <bundle> --format mermaid`,
   and optional supporting files with `okf artifact list <bundle>`.
2. Design and review an explicit old-id → new-id mapping. Include inbound bare relative,
   `./`/`../`, absolute bundle links, standard internal source resources, and other known
   path-valued fields. External URLs and scope descriptors must remain unchanged.
3. Treat `references/` as an optional convention: its Markdown files are concepts, while SQL,
   Python, schemas, instructions, and binaries are opaque artifacts. Do not pass opaque files to
   `okf mv`. If an artifact must move, separately review and update declaring paths/provenance.
4. Apply small batches with `okf mv <old-id> <new-id> <bundle>`. Do not use a raw file rename.
   Inspect affected referring documents and the implementation's meaningful-change behavior;
   path-only rebasing should not be described as a content rewrite or runtime execution.
5. After each batch, compare direct outbound links and backlinks. Use `okf graph <bundle>
   <concept-id> --direction both --depth <N> --format mermaid` only when a wider neighborhood is
   useful. Run `okf validate <bundle>` and advisory `okf lint <bundle> --fail-on never` against
   the baseline. Resolve artifact paths with declaring document context and distinguish missing,
   blocked, and scope results.
6. Regenerate reserved indexes with `okf docs <bundle> --format index` only after reviewing any
   curated index prose that could be overwritten. Validate reserved files again.
7. Review `okf diff <git-ref> <bundle>` for expected moves and link rebasing. Route genuine prose
   changes through `update` and re-verification.

Artifact reads must be bounded, remain inside the canonical bundle after symlinks, and never
authorize code execution; remote retrieval requires explicit policy. Report moves, rewritten
referrers/source paths, preserved artifacts/URLs/scopes, lifecycle or verification effects,
index handling, and before/after validation.
