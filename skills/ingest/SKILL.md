---
name: ingest
description: Use this when the user wants to research a directory or repository and ingest its knowledge into an OKF bundle — "ingest this repo into okf", "scan the codebase and create concepts", "build a bundle from this project", "analyze this directory and document it as concepts". Scans a repo, researches its structure and artifacts, then adds concepts attributed back to the source files. For pre-written docs that just need converting, prefer okf:migrate; for a fresh empty bundle, use okf:init.
---

# Research a repository and ingest it as concepts

This skill turns a codebase or directory into OKF concepts. The CLI scans and writes
deterministically; you do the research and decide what is worth capturing as a concept.

## Tool discipline
**CLI argument reference (read first).** This skill bundles the full argument list for every `okf` command as `okf-cli-reference.md` in **this skill's own directory** — read it there (the skill's absolute directory is provided to you when the skill loads; equivalently `${CLAUDE_SKILL_DIR}/okf-cli-reference.md`). Consult it to learn a command's flags; do **not** run `okf <cmd> --help` or `okf schema` just to discover arguments. Every command also takes global `--json` and an optional trailing `bundle` positional.

Anything **already in the OKF bundle** is discovered and inspected **only through the `okf`
CLI** — `okf scan` for the source map, `okf search`, `okf list`, `okf show`, `okf graph`,
`okf backlinks`, `okf resolve`, `okf stats`, `okf ontology list`/`show` (add `--json` when
parsing). Do **not** use Glob, Grep, `find`, or generic file-content search over the bundle:
`okf` already indexes it and supports progressive disclosure, so grepping it is wasteful and
defeats the design. When you must read a bundle markdown file directly, do so **only when you
already know its exact path** (from `okf resolve` / an `okf show --json` record), prefer
`okf show`, and read the **narrowest slice** needed — a known line range, section/heading, or
symbol — never the whole file speculatively.

The **external source material** being ingested (the repo/dir, which is *not* yet an OKF
bundle) is different: normal code-search tools (Glob, Grep, `find`, Read) are fine there, but
let `okf scan` be your map and still target reads by known path/symbol/line range rather than
reading whole files speculatively. The CLI-only rule applies to anything already in the bundle.

## Steps

1. **Scan the source.** Run `okf scan <source-dir>` to get a deterministic report of what
   would be analyzed (files, structure, existing markdown). Use `--json` to parse it. This is
   your map — it does not create anything.

2. **Locate/create the target bundle and ontology.** `okf init <bundle>` if none exists.
   Review concept types with `okf ontology list` / `okf ontology show <name>`. If the repo's
   domain isn't covered, run `okf:ontology` or `okf:infer-ontology` first so you classify
   against real types.

3. **Research, then decide what to ingest.** Read the scanned artifacts (source modules,
   configs, existing docs, schemas). For each meaningful unit of knowledge, decide:
   - whether it deserves a concept at all (avoid one-concept-per-file noise; capture concepts,
     not files),
   - which ontology **type** it is,
   - what the accurate title/description/prose should be. Ground every claim in what you
     actually read — do not infer behavior you can't see.

4. **Add each concept.** `okf add <path> --type <Type> --title "…" --description "…"`
   (add `--attested` for attested computation types). Write the body from your research.

5. **Attribute typed sources.** Point each concept back at the code/artifact it describes,
   using the right source **kind** so drift is detectable:
   - `git-path` for a tracked file, `line-range` for a function/span, `markdown-heading` for a
     doc section, `git-commit` to pin a revision, `url` for external references, `file` for
     untracked artifacts.
   - Add each entry with `okf edit <concept-id> --add-source
     resource=<git-root-relative-path>,kind=<kind>` (or use `--add-source` on `okf add`). Then
     run `okf refresh <concept-id> --fail-on any` to record the initial
     `fingerprint`/`last_modified`. This is what lets `okf stale` later notice the code moved.

6. **Wire references.** Connect concepts using the ontology's reference keys via
   `okf edit <concept-id> --set <key>=<link>`, mirroring real dependencies you found (e.g. a
   metric computed by a computation, a policy referencing a computation).

7. **Validate, lint, index.**
   - `okf validate <bundle>` (must pass), `okf lint <bundle>` (resolve/accept findings).
   - `okf docs <bundle> --format index` to write progressive-disclosure `index.md` files.

8. **Report.** Summarize concepts created (path + type), which parts of the repo they cover,
   and notable areas you intentionally skipped.

## Guardrails
- Fidelity over coverage: a small set of accurate, well-sourced concepts beats an exhaustive
  hallucinated one. Keep the human in the loop on scope.
- Every concept must have at least one typed `sources[]` entry so `okf stale`/`okf affected`
  can track it against the code.
- Concepts start `unverified`; use `okf:review-attest` to raise trust.
