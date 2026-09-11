---
name: ingest
description: Research a repository or directory and author grounded OKF v0.2 concepts with standard provenance. Use for source-code or mixed-artifact research; use migrate for pre-written docs and init for an empty bundle.
---

# Research and ingest a repository

Use `okf source-scan <directory>` for a complete source inventory without parsing files as OKF.
Use normal code-search tools on that external source, but query existing bundle content through
the CLI and retrieve only targeted slices.

## CLI and output rules

Read `okf-cli-reference.md` in this skill directory. Prefer `OKF_BUNDLE` during multi-step work;
one explicit form is `okf edit <concept-id> <bundle> --add-source resource=<path>`.

Producer rubric:

- Conformance requires parseable frontmatter, non-empty `type`, and valid reserved files.
- Recommended fields are `title`, `description`, applicable `resource`, `tags`, structured
  Markdown, and absolute bundle-relative concept links.
- Provenance, trust, lifecycle, and computation families are optional with defined semantics.
- Ontology and source `kind`/`fingerprint` are tool extensions, never conformance requirements.

## Workflow

1. Run `okf source-scan <source-directory> --json`. Exclude secrets, credentials, generated
   build output, and anything the user did not authorize examining or storing.
2. Research meaningful concepts rather than producing one concept per file. Ground every claim
   in inspected evidence. Use an ontology only as advisory classification; unknown types remain
   valid OKF.
3. Add concepts with `okf add <path> <bundle> --type <Type>`. When generation provenance is
   enabled, record the actual agent/tool with `--generated-by` and an explicit-offset time.
4. Author standard sources first. Each entry needs `resource`; use stable `id` values and matching
   Markdown footnotes for per-claim attribution. Add credibility or usage fields only from
   evidence. `kind`/`fingerprint` may be added for local drift tracking. Having sources is a
   recommended ingest policy when known, not an OKF conformance rule.
5. Use direct Markdown links for portable relationships. Ontology reference fields may add a
   local typed view but must not replace standard links or source lineage. Spot-check authored
   edges with `okf links <concept-id> <bundle> --json` so normalized targets and missing concepts
   are visible without traversing the whole graph.
6. Create an exact Attested Computation only for a sanctioned declared computation: use
   `--attested --runtime`, declared `--parameter` values, one inline/file computation, and any
   reviewed executor/receipt/attester contract. Never convert arbitrary executable code merely
   because it can run. `okf computation check` inspects; it does not execute or attest.
7. Run `okf validate <bundle>`, advisory `okf lint <bundle> --fail-on never`, and generate indexes.

## Local supporting artifacts

`references/` is optional. Use it only when the bundle should carry a durable authorized copy;
otherwise keep the canonical repository path or URL in `sources[].resource`. Markdown files in
it are concepts; SQL, Python, schemas, instructions, and binaries are opaque artifacts. Preserve
original provenance. Verify copies with `okf artifact resolve` and bounded `okf artifact show`.
Local resolution must remain within the canonical bundle; remote retrieval needs explicit
authorization and no ambient credentials. Reading executor, attester, or computation code is
not permission to run it.

Report created concepts, standard sources and claim joins, optional extensions, copied artifacts
with provenance, validation, advisory findings, and intentionally skipped areas.
