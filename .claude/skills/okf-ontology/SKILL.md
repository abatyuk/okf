---
name: okf-ontology
description: Use this when the user wants to manage the OKF ontology — "add a concept type", "define a new type with these fields", "update the ontology", "add a reference rule between types", "remove a concept type", "change field requirements". Shapes concept types (typed fields + typed reference rules with cardinality) and commits them via okf ontology add/update/remove so ontology.yaml stays valid and lossless. To reverse-engineer an ontology from an existing bundle, use okf-infer-ontology instead.
---

# Manage the ontology (ontology.yaml)

The ontology is a tool-local sidecar (`ontology.yaml`, spec-external) that defines concept
types, their typed fields, advisory trust expectations, and typed reference rules. You shape
*what* a type should be; you commit it through the deterministic `okf ontology` commands so
the file stays valid and lossless. **Never hand-edit `ontology.yaml`.**

## Tool discipline
**CLI argument reference (read first).** This skill bundles the full argument list for every `okf` command as `okf-cli-reference.md` in **this skill's own directory** — read it there (the skill's absolute directory is provided to you when the skill loads; equivalently `${CLAUDE_SKILL_DIR}/okf-cli-reference.md`). Consult it to learn a command's flags; do **not** run `okf <cmd> --help` or `okf schema` just to discover arguments. Every command also takes global `--json` and an optional trailing `bundle` positional.

Discover and inspect everything in the bundle **only through the `okf` CLI** — `okf ontology
list`/`show`, `okf search`, `okf list`, `okf show`, `okf graph`, `okf backlinks`, `okf resolve`,
`okf stats` (add `--json` when parsing). Do **not** use Glob, Grep, `find`, or generic
file-content search over the bundle: `okf` already indexes it and supports progressive
disclosure, so grepping it is wasteful and defeats the design. Read a bundle markdown file
directly **only when you already know its exact path** (from `okf resolve` or an `okf show
--json` record), and prefer `okf show` over a raw read. When a raw read is unavoidable, read the
**narrowest slice** needed — a known line range, a section/heading, or a named symbol — never
the whole file speculatively.

## Key facts
- Each key under `concepts:` **is the OKF `type` string verbatim** — types may contain spaces,
  so they're quoted (e.g. `"BigQuery Table"`). Use the exact string the concepts use.
- Field `type` vocabulary: `string`, `text`, `int`, `bool`, `date`, `datetime`, `uri`, `enum`
  (+`values`), `list` (+item type), `object` (+nested `fields`), or any name from `field_types`.
  Every field takes `required` (default false).
- Reference rules map a frontmatter key on the source concept to target type(s) with a
  cardinality from `0..1`, `1..1`, `0..n`, `1..n`. Targets may be a union (a list of types).
- `lint` enforces all of this **advisorily** (findings, never conformance failures); `add`
  uses it to scaffold new concepts.

## Steps

1. **Inspect the current state.** `okf ontology list` for all defined types;
   `okf ontology show <name>` for one type's fields and reference rules. Understand what
   exists before changing it.

2. **Design the change (the judgment part).** With the user, decide:
   - the type name (exact OKF `type` string), and whether it's `attested: true` (scaffolds as
     an OKF Attested Computation via `okf add --attested`),
   - required OKF built-ins (`requires:`) and custom `fields` (name → type, required?),
   - typed `references` (key → target type(s) + cardinality),
   - any advisory `trust.min_tier` expectation.
   Reuse `field_types` for repeated shapes (they compose via `extends`; object types nest
   `fields`) rather than duplicating field definitions.

3. **Commit the change deterministically.**
   - Add a type: `okf ontology add <name> --field <name>=<type>[:required] ...
     --ref <key>=<target>:<cardinality> ...`
   - Modify a type: `okf ontology update <name> ...` (change/add fields and references).
   - Remove a type: `okf ontology remove <name>`.
   Use the flag forms the CLI accepts; the command validates and writes losslessly.

4. **Verify it loaded and check impact.**
   - `okf ontology show <name>` to confirm the result.
   - `okf lint <bundle>` to see how existing concepts now measure up (new required fields or
     reference rules may surface advisory violations). Report these; don't silently mass-edit
     concepts to satisfy a new rule without confirming.

5. **Report.** State the type(s) added/updated/removed, their fields and reference rules, and
   any new lint findings the change introduced across the bundle.

## Guardrails
- Removing a type or tightening `required`/cardinality can invalidate existing concepts —
  surface the blast radius (`okf lint`) and keep the human in the loop before cascading edits.
- Ontology edits may not preserve YAML comments in v1 (known limitation); don't rely on
  comments as load-bearing.
- Keep concept-type keys byte-identical to the `type` strings concepts actually use, spaces
  and all.
