---
name: ontology
description: Author and maintain the optional tool-local ontology.yaml with advisory typed fields and reference rules. Use for local modeling, not OKF conformance; exact Attested Computation semantics remain built into OKF v0.2.
---

# Manage the tool-local ontology

`ontology.yaml` is a spec-external sidecar. It is neither required for an OKF bundle nor a type
registry. Unknown types are conformant; ontology findings are advisory.

Prefer `OKF_BUNDLE`; explicit forms are `okf ontology show <name> <bundle>` and `okf ontology add
<name> <bundle>`. Use ontology commands for deterministic writes rather than editing the sidecar
directly. Before using an unfamiliar command or flag, read its entry in `references/cli.md`. If the
installed version differs or rejects documented syntax, use that command's `--help` output as the
runtime authority.

For example, `okf ontology add "Service" <bundle> --field "owner:string:required" --ref
"depends_on:Service:0..n"` defines a field and a reference rule; choose actual names and
requirements from the reviewed model. To change rules, use `okf ontology update "Service" <bundle>
--field "owner:string" --remove-ref depends_on`; to remove a type, use `okf ontology remove
"Service" <bundle>`. Quote type names containing spaces and rule values containing `|`. Ontology
flags use colon-separated declarations; concept `--set`/`--ref` flags use `key=value` instead. Read
the reference before choosing other types or cardinalities.

## Workflow

1. Inspect `okf ontology list <bundle>`, relevant `okf ontology show <name> <bundle>`, observed
   concepts, and baseline `okf lint <bundle> --fail-on never`.
2. Use the model and constraints already supplied. Design exact type keys, custom typed fields, and
   reference rules/cardinality; ask only about unresolved semantic choices that affect the result.
   Separate portable OKF fields (`type`, title/description/resource/tags, sources,
   generated/verified, lifecycle, computation family) from ontology-only requirements. Recommended
   or optional OKF fields must not be presented as conformance blockers.
3. `Attested Computation` is the only standard computation type, recognized by that exact string and
   its built-in contract. `--attested` may mark only this exact ontology type. Do not grant attested
   semantics to an alias or arbitrary custom type.
4. Apply the authorized change with `okf ontology add`, `okf ontology update`, or `okf ontology
   remove` using the schema-derived flag forms. Removing or tightening rules needs an impact review,
   which can be performed without another approval when the requested change already authorizes it.
   Do not mass-edit concepts merely to silence advisory findings.
5. Run `okf validate <bundle>` for portable spec conformance and then `okf lint <bundle> --fail-on
   never` for ontology/spec recommendations. Report results in separate groups.

If custom fields point to supporting resources, `references/` is optional: Markdown targets are
concepts and other files are artifacts. Use `okf artifact resolve <resource> <bundle> --from
<concept-id>` and consult the artifact sections of `references/cli.md` only when retrieval is
needed. Reading a resource never authorizes execution.

Report sidecar changes, portable versus custom fields, exact computation treatment, and advisory
impact without implying that ontology controls OKF validity.
