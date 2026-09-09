---
name: migrate
description: Use this when the user wants to migrate existing documentation into an OKF bundle — "migrate this spec/plan/README into okf", "convert our docs to concepts", "import this markdown as OKF concepts", "turn these design docs into a bundle". Classifies each source document into an ontology concept type, writes conformant frontmatter, attributes the original as typed sources[], and validates the result. For raw-repo research/ingest use okf:ingest; for fresh bundles use okf:init.
---

# Migrate existing documentation into an OKF bundle

You are turning human-authored docs (specs, plans, READMEs, design notes) into OKF
concepts. The CLI handles the deterministic writes and checks; you supply the classification,
the rewritten frontmatter, and the source attribution.

## Tool discipline
**CLI argument reference (read first).** This skill bundles the full argument list for every `okf` command as `okf-cli-reference.md` in **this skill's own directory** — read it there (the skill's absolute directory is provided to you when the skill loads; equivalently `${CLAUDE_SKILL_DIR}/okf-cli-reference.md`). Consult it to learn a command's flags; do **not** run `okf <cmd> --help` or `okf schema` just to discover arguments. Every command also takes global `--json` and an optional trailing `bundle` positional.

Anything **already in the OKF bundle** is discovered and inspected **only through the `okf`
CLI** — `okf search`, `okf list`, `okf show`, `okf graph`, `okf backlinks`, `okf resolve`,
`okf stats`, `okf ontology list`/`show` (add `--json` when parsing). Do **not** use Glob, Grep,
`find`, or generic file-content search over the bundle: `okf` already indexes it and supports
progressive disclosure, so grepping it is wasteful and defeats the design. When you must read a
bundle markdown file directly, do so **only when you already know its exact path** (from
`okf resolve` / an `okf show --json` record), prefer `okf show`, and read the **narrowest
slice** needed — a known line range, section/heading, or symbol — never the whole file
speculatively.
For a large concept, run `okf show <concept-id> <bundle> --outline` first, then fetch only the
relevant inclusive range with `okf show <concept-id> <bundle> --lines <START:END>`.

The **external source docs** being migrated (the specs/plans/READMEs, which are *not* yet an
OKF bundle) are different: normal file-search tools (Glob, Grep, `find`, Read) are fine for
inventorying and reading them, and `okf scan` can map a source tree — still target reads by
known path/heading/line range rather than reading whole files speculatively. The CLI-only rule
applies to anything already in the bundle.

## Steps

1. **Locate the target bundle and ontology.** Ensure a bundle exists (`okf init <bundle>` if
   not) and inspect the concept types available: `okf ontology list`, `okf ontology show
   <name>`. If no ontology fits the docs, pause and run `okf:ontology` (or `okf:infer-ontology`)
   first — you classify against real types.

2. **Inventory the source docs.** For each markdown/text file to migrate, read it and decide:
   - **Concept type** — which ontology type it maps to (or the closest; OKF tolerates unknown
     types, but prefer a defined one so lint/scaffold work).
   - **Granularity** — one doc may become one concept, or split into several (e.g. a spec with
     independent sections). This is a judgment call; keep the human informed for big splits.

3. **Create each concept.** Run `okf add <path> --type <Type> --title "…" --description "…"`
   to scaffold conformant frontmatter from the ontology. Then move/rewrite the source prose
   into the concept body. Keep the content faithful — migration preserves meaning, it does not
   rewrite opinions.

4. **Attribute sources.** For each concept, record where its content came from in `sources[]`,
   using a typed source **kind** so `okf stale` can track drift later:
   - `git-path` (a tracked file), `line-range` (a specific span), `markdown-heading` (a
     section), `git-commit`, `file` (untracked), or `url`.
   - Each entry carries `resource` plus `kind`; the fingerprint is recorded on first sync —
     run `okf refresh <concept-id> --fail-on any` after setting sources so
     `fingerprint`/`last_modified` get written and unresolved sources fail loudly. Author each
     structured entry with `okf edit <concept-id> --add-source
     resource=<git-root-relative-path>,kind=<kind>` (or the same `--add-source` on `okf add`).
     A `git-path`/`git-commit` resource is relative to the repository root; file/text source
     kinds remain relative to the bundle.

5. **Wire references.** Reconnect cross-references between the migrated docs using the
   ontology's reference keys (`okf edit <concept-id> --set <key>=<link>`), so backlinks and
   the graph reflect the original document relationships.

6. **Validate and lint.**
   - `okf validate <bundle>` — conformance (three hard rules); must pass.
   - `okf lint <bundle>` — advisory; fix broken links, missing `title`/`description`, and
     ontology violations you introduced, or note accepted ones.
   - `okf refresh` any concept whose sources you set, so it starts in-sync rather than stale.

7. **Report.** List each migrated concept (path + type), how the original doc mapped, and any
   docs you deliberately did not migrate.

## Guardrails
- Do not fabricate metadata (owners, dates, versions). Leave optional fields empty rather than
  guessing — OKF permits missing optional fields.
- Every migrated concept should carry at least one `sources[]` entry pointing back at its
  origin, so future drift is detectable.
- Trust tier stays `unverified` after migration; raising it is the job of `okf:review-attest`.
- Prefer `--json` output when parsing lint/validate results programmatically.
