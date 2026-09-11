---
name: init
description: Start a new OKF v0.2 bundle, optionally define a tool-local ontology, and scaffold initial concepts or exact Attested Computations. Use for new bundles; use migrate or ingest for existing material and repair for an existing KB.
---

# Initialize an OKF bundle

Use the CLI for deterministic writes and checks. Decide the domain, portable concept content,
and any optional local modeling with the user.

## CLI and bundle access

Read `okf-cli-reference.md` in this skill directory before choosing flags. It is generated from
`okf schema`; do not rediscover arguments with help calls. In multi-step work, set
`OKF_BUNDLE` once. One fully qualified example is `okf add notes/hello <bundle> --type Note`.

Inspect bundle content through `okf browse`, `okf search`, `okf list`, and targeted `okf show`.
For a large concept, use `okf show <concept-id> <bundle> --outline` followed by `--lines`.

## Portable OKF versus local policy

- Required for bundle conformance: parseable frontmatter, non-empty `type`, and valid reserved
  `index.md`/`log.md` structures. Unknown custom type strings are valid.
- Recommended: `title`, `description`, applicable `resource`, `tags`, useful Markdown structure,
  and absolute bundle-relative concept links.
- Optional with defined semantics: `sources`, `generated`, `verified`, lifecycle fields, and the
  Attested Computation family.
- `ontology.yaml`, source `kind`/`fingerprint`, and sync metadata are tool extensions. Ontology
  lint is advisory and never changes OKF conformance.

## Workflow

1. Run `okf init <bundle>`. Confirm that its root index is structurally valid with
   `okf validate <bundle>`; an empty bundle still needs a non-empty `#` heading when indexed.
2. If local type rules add value, define them with `okf ontology add <name> <bundle>` and inspect
   them with `okf ontology show <name> <bundle>`. Do not present this sidecar as an OKF registry
   or requirement. Keep exact type strings; unknown types remain portable.
3. Create ordinary concepts with `okf add <path> <bundle> --type <Type>`. Add grounded title,
   description, prose, and sources. If generation provenance is desired, pass the actual actor
   with `--generated-by`; never label agent-authored material `human:<id>`.
4. For a standard computation, use exact `type: Attested Computation` via `okf add <path>
   <bundle> --attested --runtime <runtime>` and declare parameters. Choose exactly one sanctioned
   computation form: `--computation <resource>` or `--inline-computation <text-or-@file>`.
   Executor, receipt, and attester resources describe a contract; creating or reading it does
   not authorize execution. `okf computation check <concept-id> <bundle>` only inspects it.
5. Add portable Markdown links for relationships. Ontology references may supplement them.
6. Run `okf validate <bundle>`, then advisory `okf lint <bundle> --fail-on never`. Generate
   indexes with `okf docs <bundle> --format index` and confirm them with `okf browse <bundle>`.

## Optional `references/`

Create `references/` only when the user wants authorized local supporting material in the
bundle. It is an optional naming convention, not a conformance profile. Markdown files there
are ordinary concepts; SQL, Python, schemas, run instructions, and binaries are opaque artifacts.
Record the declaring concept and original provenance. Prefer a URL when material should not be
mirrored. Use `okf artifact list <bundle>` and `okf artifact resolve <resource> <bundle> --from
<concept-id>`; retrieve only bounded text with `okf artifact show <resource> <bundle> --lines
<START:END> --max-bytes <N>`.
Local resolution must stay inside the bundle after symlinks; remote fetch requires explicit
authorization and policy. Artifact inspection never grants execution authority.

Report the bundle, types, concepts, computation contracts, validation result, and separately
accepted lint guidance.
