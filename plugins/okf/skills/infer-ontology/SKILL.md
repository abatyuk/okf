---
name: infer-ontology
description: Infer an optional advisory ontology.yaml from observed OKF concepts while preserving exact type strings and separating standard v0.2 fields/edges from custom local rules. Use to reverse-engineer local modeling, not to determine conformance.
---

# Infer a local ontology from a bundle

Query bundle content through the CLI, using outlines and line slices for large concepts. Prefer
`OKF_BUNDLE`; one explicit form is `okf ontology add <name> <bundle>`. Before using an unfamiliar
command or flag, read its entry in `references/cli.md`. If the installed version differs or rejects
documented syntax, use that command's `--help` output as the runtime authority.

For example, `okf ontology add "Service" <bundle> --field "owner:string:required" --ref
"depends_on:Service:0..n"` defines a field and a reference rule; choose actual names and
requirements from the reviewed model. To change rules, use `okf ontology update "Service" <bundle>
--field "owner:string" --remove-ref depends_on`; to remove a type, use `okf ontology remove
"Service" <bundle>`. Quote type names containing spaces and rule values containing `|`. Ontology
flags use colon-separated declarations; concept `--set`/`--ref` flags use `key=value` instead. Read
the reference before choosing other types or cardinalities.

## Workflow

1. Enumerate with `okf list <bundle> --json` and `okf stats <bundle>`. Aggregate frontmatter
   occurrence counts from the inventory instead of opening every body. Preserve exact type strings.
2. Open bodies only to clarify field meaning, relationships, or exceptions. For each candidate,
   report the count with the field, the total concepts of that type, and any narrower sample
   inspected; do not report a sample count as whole-bundle support. Separate standard
   families—title/description/resource/tags, sources, generated/verified, lifecycle, and
   computation—from custom candidates. Do not reinvent standard fields as ontology custom types.
3. Infer custom reference rules only for custom frontmatter relationships. Do not duplicate body
   links or standard `sources[].resource` graph edges. Use `okf links <concept-id> <bundle> --json`,
   `okf backlinks <concept-id> <bundle>`, and targeted `okf resolve <link> <bundle> --from
   <concept-id>` calls to inspect targets and cardinality without expanding the full graph.
4. Recognize a standard computation only when the exact type is `Attested Computation`; inspect its
   built-in contract separately with `okf computation check <concept-id> <bundle>`. Do not infer an
   `attested` capability for another type.
5. Present per-rule support, counterexamples, and uncertainty. Even 100% observed presence does not
   establish an intended `required` rule. Use user-supplied domain requirements for that choice;
   otherwise propose it as optional or leave the requirement unresolved. Apply rules already
   authorized by the request; seek confirmation only for remaining consequential semantic choices.
6. Apply approved rules with `okf ontology add` or `okf ontology update`, then run `okf validate
   <bundle>` and `okf lint <bundle> --fail-on never`. Report portable validation separately from
   ontology fit; loosen unsupported rules instead of rewriting the bundle automatically.

If custom fields point to supporting resources, `references/` is optional: Markdown targets are
concepts and other files are artifacts. Use `okf artifact resolve <resource> <bundle> --from
<concept-id>` and consult the artifact sections of `references/cli.md` only when retrieval is
needed. Reading a resource never authorizes execution.

Report exact types, built-in fields/contracts excluded from inference, custom rules with evidence,
user decisions, and remaining advisory findings.
