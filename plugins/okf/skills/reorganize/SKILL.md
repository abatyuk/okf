---
name: reorganize
description: Plan and move OKF concepts while rebasing portable body links and standard internal source resources, preserving opaque artifacts and validating indexes. Use for path/layout refactors, not content rewrites.
---

# Reorganize an OKF bundle

A concept id is its bundle-relative Markdown path. Use `okf mv`; it rewrites inbound body links and
internal standard source resources. Link rebasing legitimately edits referring documents, so a
concept diff may show add/remove or modified referrers rather than a pure native rename.

Prefer `OKF_BUNDLE`. Correct explicit forms include `okf mv <old-id> <new-id> <bundle>`, `okf
backlinks <concept-id> <bundle>`, and `okf links <concept-id> <bundle> --json`. Before using an
unfamiliar command or flag, read its entry in `references/cli.md`. If the installed version differs
or rejects documented syntax, use that command's `--help` output as the runtime authority.

## Workflow

1. Establish a recoverable baseline and record existing validation/lint findings. Inventory the
   affected concepts with `okf list <bundle> --json` and direct links/backlinks. Use a rooted graph
   only when relationships need explanation; avoid a full graph for a local move. Inventory
   supporting files with `okf artifact list <bundle>` only if affected.
2. Design and review an explicit old-id → new-id mapping. Include inbound bare relative, `./`/`../`,
   absolute bundle links, standard internal source resources, and other known path-valued fields.
   External URLs and scope descriptors must remain unchanged.
3. Preflight every destination for collisions and ensure each source exists. Order dependent moves
   so destinations are free. For swaps/cycles, use an unused temporary concept ID inside the bundle
   via `okf mv`, then complete the planned mapping. Record completed moves after each successful
   command; on an unexpected result, stop the batch and reconcile actual state before retrying. Do
   not blindly replay completed moves or overwrite a destination.
4. Treat `references/` as an optional convention: its Markdown files are concepts, while SQL,
   Python, schemas, instructions, and binaries are opaque artifacts. Do not pass opaque files to
   `okf mv`. If an artifact must move, separately review and update declaring paths/provenance.
5. Apply small batches with `okf mv <old-id> <new-id> <bundle>`. Do not use a raw file rename.
   Inspect affected referring documents and the implementation's meaningful-change behavior;
   path-only rebasing should not be described as a content rewrite or runtime execution.
6. After each batch, compare direct outbound links and backlinks. Use `okf graph <bundle> --root
   <concept-id> --direction both --depth <N> --format mermaid` only when a wider neighborhood is
   useful. Run `okf validate <bundle>` and advisory `okf lint <bundle> --fail-on never` against the
   baseline. Resolve artifact paths with `okf artifact resolve <resource> <bundle> --from
   <concept-id>` and distinguish missing, blocked, and scope results. Apply reviewed declaring-field
   corrections with `okf edit <concept-id> <bundle> --set "resource=<new-path>"`; consult the edit
   reference for source mappings and body edits.
7. Regenerate indexes with `okf docs <bundle> --format index` only when their bodies are generated
   or replacement is already authorized. It replaces index prose throughout the bundle. Preserve
   curated indexes and make only necessary link corrections otherwise; validate reserved files
   again.
8. Review `okf diff <git-ref> <bundle>` for expected moves and link rebasing. Route genuine prose
   changes through `update` and re-verification.

Artifact reads must be bounded, remain inside the canonical bundle after symlinks, and never
authorize code execution. Consult the artifact reference for external source reads. Report moves,
rewritten referrers/source paths, preserved artifacts/URLs/scopes, lifecycle or verification
effects, index handling, and before/after validation.

Artifact inventory defaults to `references/`. Use `okf artifact list <bundle> --directory
<directory>` for another bundle-relative directory. The positional argument selects the bundle, not
that directory.
