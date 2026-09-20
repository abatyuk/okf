---
name: init
description: Start a new OKF v0.2 bundle, optionally define a tool-local ontology, and scaffold initial concepts or exact Attested Computations. Use for new bundles; use migrate or ingest for existing material and repair for an existing KB.
---

# Initialize an OKF bundle

Use the CLI for writes and checks. Use the domain, scope, and modeling choices already supplied;
choose ordinary defaults for reversible scaffolding. Ask only about unresolved choices that
materially change the requested bundle.

## CLI and bundle access

In multi-step work, set `OKF_BUNDLE` once. One fully qualified example is `okf add notes/hello
<bundle> --type Note`. Before using an unfamiliar command or flag, read its entry in
`references/cli.md`. If the installed version differs or rejects documented syntax, use that
command's `--help` output as the runtime authority.

Inspect bundle content through `okf browse`, `okf search`, `okf list`, and targeted `okf show`. For
text search, use `okf search <bundle> --text "query terms" --limit 10`; the positional is always the
bundle path, never the query. Use `okf list <bundle>` for an unfiltered inventory. For a large
concept, use `okf show <concept-id> <bundle> --outline`, then `okf show <concept-id> <bundle>
--lines <START:END>`.

## Portable OKF versus local policy

- Required for bundle conformance: parseable frontmatter, non-empty `type`, and valid reserved
  `index.md`/`log.md` structures. Unknown custom type strings are valid.
- Recommended: `title`, `description`, applicable `resource`, `tags`, useful Markdown structure, and
  absolute bundle-relative concept links.
- Optional with defined semantics: `sources`, `generated`, `verified`, lifecycle fields, and the
  Attested Computation family.
- `ontology.yaml`, source `kind`/`fingerprint`, and sync metadata are tool extensions. Ontology lint
  is advisory and never changes OKF conformance.

## Workflow

1. Run `okf init <bundle>`; it creates an empty ontology sidecar by default. Use `okf init <bundle>
   --no-ontology` when no local ontology is wanted. Confirm that its root index is structurally
   valid with `okf validate <bundle>`; an empty bundle still needs a non-empty `#` heading when
   indexed.
2. If local type rules add value, define them with `okf ontology add <name> <bundle>` and inspect
   them with `okf ontology show <name> <bundle>`. Do not present this sidecar as an OKF registry or
   requirement. Keep exact type strings; unknown types remain portable.
3. Create ordinary concepts with `okf add <path> <bundle> --type <Type>`. Add grounded title,
   description with `--title "Title" --description "Summary"`. Add prose after creation with `okf
   edit <concept-id> <bundle> --set-body @/tmp/concept-body.md`; the file contains Markdown body
   only. Add a source with `okf edit <concept-id> <bundle> --add-source
   "resource=<path>,id=source-1"`. If generation provenance is desired, pass `--generated-by
   <actor>` to add; the CLI supplies the timestamp. Never label agent-authored material
   `human:<id>`.
4. Only for a requested computation, use exact `Attested Computation` with `okf add <path> <bundle>
   --attested --runtime <runtime>`. Read the add/computation sections of `references/cli.md` for
   parameters, inline/file forms, and contract resources. Inspect with `okf computation check
   <concept-id> <bundle>`; this does not execute or attest a run.
5. Add portable Markdown links for relationships. Ontology references may supplement them.
6. Run `okf validate <bundle>`, then advisory `okf lint <bundle> --fail-on never`. Generate indexes
   with `okf docs <bundle> --format index` only when their bodies are generated content or
   replacement is already authorized; the command replaces index prose across the bundle. Preserve
   curated indexes otherwise. Confirm generated indexes with `okf browse <bundle>`.

## Supporting material

`references/` is optional: Markdown files there are concepts; other files are artifacts. Copy only
material within the authorized scope and preserve its provenance. Use `okf artifact list <bundle>`
to inventory its default `references/` directory, or add `--directory <directory>` for another
directory. Resolve paths with `okf artifact resolve <resource> <bundle> --from <concept-id>`;
consult the artifact sections of `references/cli.md` for bounded reads and external-source handling.
Inspection never authorizes execution.

Report what was created, validation, and relevant lint findings. Include computation or artifact
details only when those features were used.
