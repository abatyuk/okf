---
name: okf-reorganize
description: Use this when the user wants to restructure an OKF bundle's layout — "reorganize the bundle", "move these concepts into a new folder", "rename this concept", "restructure the directory tree", "regroup concepts by type/domain", "clean up the bundle structure". Restructures the tree using okf mv so every inbound link is rewritten and nothing dangles. Because a concept id IS its file path, a naive move silently breaks references — always go through okf mv. For editing content or ontology, use okf-update / okf-ontology.
---

# Reorganize / refactor a bundle (links stay intact)

A concept's **id equals its bundle-relative file path**, so moving or renaming a file with `mv`
or an editor silently breaks every reference to it. `okf mv <old-id> <new-id>` moves the concept
**and rewrites every inbound link** deterministically. This skill is about planning a good
structure and executing it exclusively through `okf mv`.

## Tool discipline
**CLI argument reference (read first).** This skill bundles the full argument list for every `okf` command as `okf-cli-reference.md` in **this skill's own directory** — read it there (the skill's absolute directory is provided to you when the skill loads; equivalently `${CLAUDE_SKILL_DIR}/okf-cli-reference.md`). Consult it to learn a command's flags; do **not** run `okf <cmd> --help` or `okf schema` just to discover arguments. Every command also takes global `--json` and an optional trailing `bundle` positional.

Discover and inspect everything in the bundle **only through the `okf` CLI** — `okf search`,
`okf list`, `okf show`, `okf graph`, `okf backlinks`, `okf resolve`, `okf stats` (add `--json`
when parsing). Do **not** use Glob, Grep, `find`, or generic file-content search over the
bundle: `okf` already indexes it and supports progressive disclosure, so grepping it is
wasteful and defeats the design. Read a bundle markdown file directly **only when you already
know its exact path** (from `okf resolve` or an `okf show --json` record), and prefer
`okf show` over a raw read. When a raw read is unavoidable, read the **narrowest slice** needed
— a known line range, a section/heading, or a named symbol — never the whole file speculatively.

## Steps

1. **Map the current structure.** `okf list <bundle> --json` for all concept ids/types and
   `okf graph <bundle> [--format mermaid]` to see how things link. `okf stats <bundle>` shows
   the type distribution. Understand the existing layout before changing it.

2. **Design the target layout (the judgment part).** Decide the new tree with the user — e.g.
   group by concept type, by domain/subtree, or flatten an over-nested area. Produce an explicit
   old-id → new-id mapping. Keep names stable and meaningful; every rename is churn, so move with
   intent, not cosmetically. Confirm the plan before executing.

3. **Establish a clean baseline.** Reorganization is large and mechanical — make sure the bundle
   is committed / clean first so `okf diff <bundle> <git-ref>` can later confirm only moves
   happened. Note current `okf lint` findings so you can tell new breakage from pre-existing.

4. **Execute moves one at a time.** For each mapping entry: `okf mv <old-id> <new-id>`. Each move
   rewrites inbound links across the bundle automatically — do **not** use shell `mv`, an editor
   rename, or hand-edit links. Move incrementally so a mistake is easy to trace. If a target
   subtree doesn't exist yet, `okf mv` should create the path; verify.

5. **Verify integrity after each batch.**
   - `okf lint <bundle>` — must show **no new broken-link findings** vs the baseline; if a link
     dangled, a move didn't rewrite it (investigate before continuing).
   - `okf backlinks <bundle> <new-id>` on a few moved concepts to confirm references followed.
   - `okf validate <bundle>` — conformance still holds.

6. **Regenerate indexes.** Directory layout changed, so refresh progressive-disclosure indexes:
   `okf docs <bundle> --format index`.

7. **Confirm the diff is moves-only.** `okf diff <bundle> <git-ref>` — the changes should be
   renames/relocations, not content edits. Content changes here are a red flag (reorganize moves,
   it doesn't rewrite).

8. **Report.** List the moves performed (old-id → new-id), the resulting structure, and
   confirmation that no links dangle.

## Guardrails
- **Only `okf mv` moves concepts.** Any other rename mechanism breaks the id↔path↔link invariant.
- Don't mix content rewrites into a reorganization — route those through `okf-update` separately
  so the diff stays reviewable.
- Keep the human in the loop on the target layout; structure is subjective and expensive to redo.
- Prefer small batches with a lint check between them over one giant move.
