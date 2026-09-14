---
name: ontology
description: Author and maintain the optional tool-local ontology.yaml with advisory typed fields and reference rules. Use for local modeling, not OKF conformance; exact Attested Computation semantics remain built into OKF v0.2.
---

# Manage the tool-local ontology

`ontology.yaml` is a spec-external sidecar. It is neither required for an OKF bundle nor a type
registry. Unknown types are conformant; ontology findings are advisory.

Prefer `OKF_BUNDLE`; explicit forms are `okf ontology show <name> <bundle>` and `okf ontology add
<name> <bundle>`. Use ontology commands for deterministic writes rather than editing the sidecar
directly. When exact flags or output shapes are needed, read the focused `references/cli.md`. If
its generated tool version differs from the installed binary, use that command's `--help` output
as the runtime authority.

## Workflow

1. Inspect `okf ontology list <bundle>`, relevant `okf ontology show <name> <bundle>`, observed
   concepts, and baseline `okf lint <bundle> --fail-on never`.
2. Design exact type keys, custom typed fields, and custom reference rules/cardinality with the
   user. Separate portable OKF fields (`type`, title/description/resource/tags, sources,
   generated/verified, lifecycle, computation family) from ontology-only requirements.
   Recommended or optional OKF fields must not be presented as conformance blockers.
3. `Attested Computation` is the only standard computation type, recognized by that exact string
   and its built-in contract. `--attested` may mark only this exact ontology type. Do not grant
   attested semantics to an alias or arbitrary custom type.
4. Apply the reviewed change with `okf ontology add`, `okf ontology update`, or
   `okf ontology remove` using the schema-derived flag forms. Removing or tightening rules needs
   an impact review; do not mass-edit concepts merely to silence advisory findings.
5. Run `okf validate <bundle>` for portable spec conformance and then `okf lint <bundle>
   --fail-on never` for ontology/spec recommendations. Report results in separate groups.

If a custom resource field points into optional `references/`, remember that Markdown targets are
ordinary concepts and non-Markdown targets are opaque artifacts. Resolve with document context
through `okf artifact resolve`, retrieve bounded text only, stay within canonical bundle scope,
and never treat inspection as execution permission.

Report sidecar changes, portable versus custom fields, exact computation treatment, and advisory
impact without implying that ontology controls OKF validity.
