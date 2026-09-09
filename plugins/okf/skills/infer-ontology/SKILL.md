---
name: infer-ontology
description: Use this when the user wants to derive an ontology from an existing OKF bundle — "infer an ontology from this bundle", "propose an ontology.yaml", "what types/fields does this bundle actually use?", "reverse-engineer the schema", "bootstrap an ontology from existing concepts". Analyzes observed types, common fields, and reference patterns across the bundle, proposes an ontology.yaml, and commits it via okf ontology. The reverse of authoring an ontology by hand; a fast on-ramp for okf:migrate/okf:ingest. To author types from scratch, use okf:ontology.
---

# Infer an ontology from an existing bundle

Some bundles exist without an `ontology.yaml`. This skill observes what the concepts actually
do and proposes a matching ontology, then commits it deterministically. You do the pattern
analysis and the naming; `okf ontology` does the lossless writes.

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
For a large concept, run `okf show <concept-id> <bundle> --outline` first, then fetch only the
relevant inclusive range with `okf show <concept-id> <bundle> --lines <START:END>`.

## Steps

1. **Enumerate the bundle.** `okf list <bundle> --json` (NDJSON) for every concept with its
   `type`, and `okf stats <bundle>` for concept counts by type and the trust-tier distribution.
   This tells you which types exist and how common each is.

2. **Sample each observed type.** For each distinct `type` string (preserve it **verbatim**,
   spaces and all), `okf show` a representative sample of concepts. Because concept records
   mirror frontmatter verbatim, the NDJSON gives you every field each concept carries. Record:
   - which frontmatter keys appear, and how consistently (a key on ~all concepts of a type →
     candidate `required`; sporadic → optional),
   - each field's apparent value shape (string / date / uri / enum-like small value set /
     list / object) → propose a field `type`, using `enum` (+`values`) where a small fixed set
     recurs,
   - which keys hold **links** to other concepts, and what types those links point at →
     candidate reference rules.

3. **Infer reference rules and cardinality.** For each link-bearing key, use `okf resolve` /
   `okf backlinks` to see the target types. Determine target type(s) (a union if it points at
   several) and cardinality from observed counts: always exactly one → `1..1`; optional single
   → `0..1`; zero-or-more → `0..n`; one-or-more → `1..n`.

4. **Draft the proposal (the judgment part).** Assemble a proposed ontology: one `concepts:`
   entry per observed type (quoted key = the verbatim `type`), with inferred `fields`,
   `requires`, and typed `references`. Factor repeated field shapes into `field_types` where it
   reduces duplication. Mark attested types (`attested: true`) if their concepts are OKF
   Attested Computations. **Present this proposal to the user for confirmation** — inference is
   a guess about intent; don't commit silently.

5. **Commit via okf ontology.** Once approved, realize each type with `okf ontology add <name>
   --field ... --ref ...` (and `okf ontology update` to refine). Never hand-write
   `ontology.yaml` — the commands keep it valid and lossless.

6. **Validate the fit.** Run `okf lint <bundle>` to see how well the proposed ontology matches
   reality. High violation counts usually mean a `required`/cardinality guess was too strict —
   loosen it via `okf ontology update` rather than mass-editing concepts. Iterate until the
   ontology describes the bundle rather than fighting it.

7. **Report.** List the types inferred (with field/reference counts), notable ambiguities you
   resolved by choice, and the residual lint findings.

## Guardrails
- Infer from evidence, not aspiration: propose rules the existing concepts already satisfy;
  don't invent stricter requirements the user didn't ask for.
- Preserve type strings exactly (spaces → quoted keys).
- Keep the human in the loop before committing — this sets the rules everything else lints
  against.
