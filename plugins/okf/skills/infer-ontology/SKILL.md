---
name: infer-ontology
description: Infer an optional advisory ontology.yaml from observed OKF concepts while preserving exact type strings and separating standard v0.2 fields/edges from custom local rules. Use to reverse-engineer local modeling, not to determine conformance.
---

# Infer a local ontology from a bundle

Read the generated `okf-cli-reference.md` in this skill directory. Query bundle content through
the CLI, using outlines and line slices for large concepts. Prefer `OKF_BUNDLE`; one explicit form
is `okf ontology add <name> <bundle>`.

## Workflow

1. Enumerate with `okf list <bundle> --json` and `okf stats <bundle>`. Preserve every observed
   `type` byte-for-byte, including unknown types and spaces.
2. Sample each type with targeted `okf show`. Record field shape and occurrence as support counts,
   not just percentages. Separate standard families—title/description/resource/tags, sources,
   generated/verified, lifecycle, and computation—from custom candidates. Do not reinvent
   standard fields as ontology custom types.
3. Infer custom reference rules only for custom frontmatter relationships. Do not duplicate body
   links or standard `sources[].resource` graph edges. Use `okf resolve <link> <bundle> --from
   <concept-id>` and `okf backlinks <concept-id> <bundle>` to inspect targets and cardinality.
4. Recognize a standard computation only when the exact type is `Attested Computation`; inspect
   its built-in contract separately with `okf computation check <concept-id> <bundle>`. Do not
   infer an `attested` capability for another type.
5. Present the proposal with per-rule support, counterexamples, confidence, and ambiguity. Never
   infer `required` from a small sample. The ontology is advisory; preserve unknown types and get
   user confirmation before writing.
6. Apply approved rules with `okf ontology add` or `okf ontology update`, then run
   `okf validate <bundle>` and
   `okf lint <bundle> --fail-on never`. Report portable validation separately from ontology fit;
   loosen unsupported rules instead of rewriting the bundle automatically.

For optional `references/`, Markdown files remain concepts and opaque files remain artifacts.
When sampled custom fields point there, resolve document-relative paths via `okf artifact resolve`
and inspect only bounded content. Stay inside canonical bundle scope; remote retrieval and any
execution require separate authorization.

Report exact types, built-in fields/contracts excluded from inference, custom rules with evidence,
user decisions, and remaining advisory findings.
