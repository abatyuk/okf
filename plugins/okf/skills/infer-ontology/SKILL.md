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


## CLI compatibility preflight

Before CLI-dependent work, run `sh <this-skill-directory>/../../scripts/check-cli.sh`,
resolving the path from this installed SKILL.md, not the working directory. This read-only
check uses the plugin's `cli-compatibility.txt` range and reports the executable on PATH.
If missing, incompatible, or unable to run, stop CLI-dependent work and show its installation
instructions. Never invoke the installer, download a CLI, or update it from this skill.
An explicit user request to install should be handled separately from this workflow.
Only after this check passes may command help resolve syntax differences; help cannot waive
compatibility. Recheck if the executable or PATH changes during the task.

## Workflow

1. Select the bundle and necessary examination scope, inspecting each bundle's own effective
   ontology/settings. Enumerate with `okf list <bundle> --json` and `okf stats <bundle>`. Check
   query summaries/warnings before treating list output as an inventory: the default scan budget
   is 1,000 documents, and `--limit` only limits returned hits. For whole-scope inference, complete
   the scan with a sufficient `--scan-limit` or deliberate `--full-scan`; otherwise label all counts
   as sampled/observed. An incomplete scan has no authoritative offset continuation. Aggregate
   complete metadata streams locally instead of loading every record or repeatedly scanning tiny
   offset pages. Use facets for candidate scalar vocabularies and projections for relevant nested
   records; inspect exclusions and truncation independently. Preserve exact type strings.
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
6. Apply approved scalar or structured rules with `okf ontology add` or `okf ontology update`. For reusable nested
   types, selectors, and relationships, follow the ontology skill's
   [structured authoring example](../ontology/references/structured-authoring.md): preserve current
   bytes, preview structured CLI changes, inspect effective definitions, and review unknown-key preservation.
   Then run `okf validate
   <bundle>` and `okf lint <bundle> --fail-on never`. Report portable validation separately from
   ontology fit; loosen unsupported rules instead of rewriting the bundle automatically.

If custom fields point to supporting resources, `references/` is optional: Markdown targets are
concepts and other files are artifacts. Use `okf artifact resolve <resource> <bundle> --from
<concept-id>` and consult the artifact sections of `references/cli.md` only when retrieval is
needed. Reading a resource never authorizes execution.

Report exact types, built-in fields/contracts excluded from inference, custom rules with evidence,
user decisions, and remaining advisory findings.

## Bundle identity and examination scope

Keep bundle paths and catalog IDs distinct. `OKF_BUNDLE` always selects a path; use the
`--bundle-id <id>` for a registered identity. When a catalog is configured, use `okf catalog` to inspect effective local
registrations, overrides, availability, and per-bundle settings. Catalog paths are relative to
`okf.toml`, locations are relative to the catalog, and machine-local overrides change effective
locations rather than accepting different snapshots. Inspect effective settings before relying on
a per-bundle ontology or view. Equal or nested registered roots are configuration errors.

Registration alone does not authorize catalog-wide traversal. Start with the selected bundle;
request additional bundles explicitly with repeatable `--scope-bundle <id>` or deliberate
`--catalog-scope` when the task requires them. Report requested and examined
scope and unavailable members. Out-of-scope references remain unknown. Ordinary relative paths
retain their meaning across registered boundaries; distributing one bundle alone can break them.
Never infer another bundle from a matching filename or title.

## Structured evidence

Inspect recurring nested objects and lists before proposing reusable field types. Declare reference
selectors only where source meaning establishes a concept relationship; path-like values alone do
not suffice. Pair target, kind, and attributes within the same record. Report observed occurrence
counts and candidate vocabularies; absence from a page or truncated facets does not establish a
closed vocabulary. Inferred inverse names describe incoming assertions rather than target consent.
