---
name: update
description: Use this when the user wants to bring OKF documentation back in sync with recent changes — "update the docs for these changes", "which concepts are stale?", "what's affected by this commit/file change?", "refresh the bundle after the refactor", "the code changed, fix the concepts". Uses okf stale (automatic drift), okf affected (blast radius of known changes), and okf diff (vs a git ref) to find what needs work; you rewrite the prose and re-attribute sources, then okf refresh to re-fingerprint.
---

# Update documentation from recent changes

The CLI finds *what* drifted or is impacted deterministically. Your job is *how* to rewrite
the prose and re-attribute sources. Three complementary detectors feed you:

- `okf stale` — drift you didn't know about (recorded fingerprint vs. current artifact).
- `okf affected --changed <link>...` — blast radius of changes you already know about.
- `okf diff <git-ref>` — what changed vs a baseline commit.

## Tool discipline
**CLI argument reference (read first).** This skill bundles the full argument list for every `okf` command as `okf-cli-reference.md` in **this skill's own directory** — read it there (the skill's absolute directory is provided to you when the skill loads; equivalently `${CLAUDE_SKILL_DIR}/okf-cli-reference.md`). Consult it to learn a command's flags; do **not** run `okf <cmd> --help` or `okf schema` just to discover arguments. Every command also takes global `--json` and an optional trailing `bundle` positional.

Discover and inspect everything in the bundle **only through the `okf` CLI** — `okf search`,
`okf list`, `okf show`, `okf graph`, `okf backlinks`, `okf resolve`, `okf stats`, plus the
drift detectors above (add `--json` when parsing). Do **not** use Glob, Grep, `find`, or generic
file-content search over the bundle: `okf` already indexes it and supports progressive
disclosure, so grepping it is wasteful and defeats the design. Read a bundle markdown file
directly **only when you already know its exact path** (from `okf resolve` or an `okf show
--json` record), and prefer `okf show` over a raw read. When a raw read is unavoidable, read the
**narrowest slice** needed — a known line range, a section/heading, or a named symbol — never
the whole file speculatively. (Reading the *external artifacts* a concept sources — the code or
docs it points at — by known path/line-range to check drift is expected; the CLI-only rule is
about bundle content.)

## Steps

1. **Find the work.** Pick the detector(s) that match the situation (use `--json` to parse):
   - Routine sync / CI: `okf stale <bundle>`. It resolves each concept's `resource`/`sources[]`
     and reports every source whose fingerprint no longer matches (dispatching per `kind` —
     git-path blob SHA, markdown-heading section hash, git-commit, line-range, url etag, etc.),
     plus missing artifacts and `stale_after` expiry.
   - You know specific things changed: `okf affected <bundle> --changed <link-or-path>...`
     (args or stdin). Add `--transitive [--depth N]` to follow the cascade through the
     reverse-link graph, not just direct dependents.
   - Working from a commit range: `okf diff <bundle> <git-ref>` for concept-level added /
     removed / modified. Feed changed paths from the diff into `okf affected --changed`.

2. **Triage.** Group the reported concepts. For each, open it with `okf show <concept-id>` and
   read the drifted source (the artifact it points at). Decide whether the prose actually needs
   to change or whether the artifact moved in a way that doesn't affect meaning.

3. **Rewrite the prose (the judgment part).** For concepts whose content is now wrong or
   incomplete, edit the body to reflect reality. Keep changes grounded in the actual diff/artifact
   — don't rewrite beyond what changed. Update frontmatter fields via
   `okf edit <concept-id> --set <key>=<value>` where facts (version, status, owner) changed.

4. **Re-attribute sources if the artifact moved.** If a source file was renamed, split, or a
   heading changed, update the `sources[]` `resource`/`kind` so it points at the new location.
   For renamed *concepts*, use `okf mv` (never a manual rename) so inbound links are rewritten.

5. **Re-fingerprint.** Once a concept is genuinely back in sync, run `okf refresh <concept-id>`.
   This re-reads the artifacts, rewrites `fingerprint`/`last_modified` losslessly, and clears
   `stale` — the "I've reviewed this, it matches again" operation. Do NOT refresh a concept you
   haven't actually reconciled; that would hide real drift.

6. **Re-check.** Run `okf stale <bundle>` again to confirm nothing you touched still drifts, and
   `okf lint <bundle>` to catch links you may have broken while editing.

7. **Report.** List concepts updated (what changed and why), sources re-attributed, concepts
   refreshed, and any drift you deliberately left (e.g. needs a human decision).

## Guardrails
- `refresh` is an acknowledgement, not a fix — only refresh after the prose truly matches.
- Never rename a concept file by hand; `okf mv` keeps backlinks intact.
- Keep the human in the loop on ambiguous drift and on any content whose correct new value you
  can't determine from the artifact.
- Trust tier may drop conceptually after a big rewrite — consider routing through
  okf:review-attest for re-verification.
