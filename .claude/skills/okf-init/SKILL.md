---
name: okf-init
description: Use this when the user wants to start a new OKF bundle from scratch — "initialize an OKF bundle", "scaffold a knowledge base", "set up a new okf bundle", "create the ontology and first concepts". Bootstraps an empty bundle, establishes an ontology.yaml, and scaffolds the first concept documents from the ontology's concept types. Not for importing existing docs (use okf-migrate/okf-ingest) or restructuring an existing bundle (use okf-reorganize).
---

# Initialize an OKF bundle

The Rust CLI does the deterministic scaffolding (`okf init`, `okf add`, `okf ontology`).
Your job is the judgment: what domain this bundle covers, which concept types it needs,
and the prose in each first concept.

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

1. **Create the bundle.** Run `okf init <bundle>` to create an empty OKF bundle in the
   target directory (default `.`). This lays down the base structure; it does not invent
   concepts.

2. **Establish the ontology.** Decide the concept types the bundle needs with the user.
   - If the user has no ontology in mind, propose a small starter set of concept types
     (each key is the OKF `type` string verbatim — quote types with spaces, e.g.
     `"BigQuery Table"`). Confirm names and required fields before writing.
   - Commit each type deterministically via `okf ontology add <name> --field <k>=<type> ...
     --ref <key>=<target>:<cardinality> ...`. Cardinality vocabulary is `0..1`, `1..1`,
     `0..n`, `1..n`. Mark attested types (`attested: true`) so `okf add --attested`
     scaffolds them as OKF Attested Computations.
   - Inspect what you defined with `okf ontology list` and `okf ontology show <name>`.
   - If the user already has an `ontology.yaml`, skip authoring and just verify it loads.

3. **Scaffold the first concepts.** For each concept the user wants to seed, run
   `okf add <path> --type <Type> --title "…" --description "…"`. `add` pulls required
   fields and reference keys from the ontology, so the frontmatter starts conformant.
   Use `--attested` for attested computation types.
   - Example: `okf add policies/travel_expenses --type Policy --title "Travel and expense
     policy" --description "Rules and reimbursement rates for business travel."`
   - Write the initial prose body for each concept yourself. Keep it short and truthful;
     do not fabricate specifics the user hasn't provided.

4. **Wire references.** Where the ontology defines reference rules (e.g. a `Policy`
   references `Computation`), set the linking frontmatter keys with
   `okf edit <concept-id> --set <key>=<value>` so the graph is connected from the start.

5. **Check conformance and opinions.**
   - `okf validate <bundle>` — must pass (the spec's three hard rules). Fix any failure.
   - `okf lint <bundle>` — advisory. Resolve or consciously accept broken links / missing
     descriptions / ontology violations. Lint findings are opinions, not blockers.

6. **Generate progressive-disclosure indexes.** Run `okf docs <bundle> --format index` to
   write `index.md` files so later retrieval can navigate the bundle cheaply.

7. **Report.** Summarize the bundle path, the ontology types created, and the concepts
   scaffolded. Point the user at `okf-migrate`/`okf-ingest` to bring in existing material.

## Guardrails
- Keep the human in the loop on type names and required fields — these are subjective and
  hard to change later without churn.
- Never hand-edit `ontology.yaml`; always go through `okf ontology` so it stays valid and
  lossless.
- Prefer `--json` (NDJSON) output when you need to parse results programmatically.
