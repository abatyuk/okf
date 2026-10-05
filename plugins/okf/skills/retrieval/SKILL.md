---
name: retrieval
description: Answer questions from OKF bundles using scoped structured queries, progressive disclosure, source lineage, and explicit completeness/trust evidence. Use before reading bundle files directly; this skill is read-only.
---

# Retrieve knowledge from an OKF bundle

Query the bundle via the CLI rather than bulk-reading it. Use `OKF_BUNDLE` for multi-step work;
explicit examples are `okf show <concept-id> <bundle>` and `okf links <concept-id> <bundle> --json`.
Before using an unfamiliar command or flag, read its entry in `references/cli.md`. If the installed
version differs or rejects documented syntax, use that command's `--help` output as the runtime
authority.

Search text must follow `--text`; the positional argument to search is the bundle path. For example,
`okf search <bundle> --text "travel policy" --limit 10`. Structured-only searches are valid, such as
`okf search <bundle> --type Note --tag policy --limit 10`. Without filters, search lists concepts;
use `okf list <bundle> --json` for an intended inventory. Use `okf browse <bundle> --directory
<directory>` to descend; its positional is also the bundle.

## Progressive disclosure

1. Establish the primary bundle and the scope needed for the question before querying. For catalog
   work, inspect registrations once and pass the selected `--bundle-id` on subsequent commands;
   add only the required `--scope-bundle` identities. A request spanning named bundles supplies
   that scope; it does not require another permission question. Reinspect settings only if the
   selection, configuration, or revision changes.
   Use the shortest route that resolves the question: open a named concept directly, search a known
   topic, or use `okf browse <bundle>` when orientation is needed. Descend with `--directory`;
   search with `okf search <bundle> --text "query terms" --limit 10`. Narrow noisy results with
   `--in title,description`; adding `frontmatter` expands metadata coverage. The default matches
   reader-visible phrases. Consult the search reference for repeatable phrases, token modes,
   literal Markdown matching, and sorting.
2. Choose the query from the evidence needed. Use typed filters for known metadata, facets when
   discovering a vocabulary/distribution, and `--project` for selected metadata. Inspect the
   effective ontology with `okf ontology show <name> <bundle>` for relationship rule names, then
   use `--expand` with `--target-field` for
   one-hop outbound metadata instead of opening each target. Use `backlinks --details` for incoming
   assertions; open bodies only for evidence missing from metadata. For worked decisions, scan
   recovery, and correlated-record pitfalls, read [query planning](references/query-planning.md).
   Check query summaries and warnings before using results: the default scan budget is 1,000
   eligible documents across scope, independent of result `--limit`. Incomplete scans cannot
   establish absence, exact totals, complete inventories, or a globally sorted page. Narrow the
   examination scope when appropriate, raise `--scan-limit`, or use `--full-scan` when exhaustive
   coverage is needed. Never use offset continuation to repair an incomplete scan.
3. For text searches with `--json`, use each result's `search.score` and `search.matches` to choose candidates. Match
   evidence identifies the field, bounded snippet, and a serialized document line when applicable;
   use that line to request a small `show --lines` window. Treat snippets as navigation evidence,
   not enough context to answer from.
4. Read a small candidate directly with `okf show <concept-id> <bundle>`. For large concepts, use
   `okf show <concept-id> <bundle> --outline`, then `--lines <START:END>` around relevant sections.
   Plain `show --json` contains metadata, not the body. For a small concept's prose alone, use
   `okf show <concept-id> <bundle> --body`; `--body --json` provides an explicit body record.
   `okf show <concept-id> <bundle> --numbered` (short form `-n`) uses the same document numbering
   as `--lines`, including frontmatter and excluding the plain display header. Keep outline/line windows for large documents.
5. Follow portable body links and standard internal `sources[].resource` lineage when needed to
   support the answer; these work without ontology. Start with `okf links <concept-id> <bundle>
   --json` for direct outbound targets and `okf backlinks <concept-id> <bundle>` for direct inbound
   concepts. Render only the useful neighborhood with `okf graph <bundle> --root <concept-id>
   --direction <outgoing|incoming|both> --depth <N> --format mermaid`; omit the root only when the
   entire graph is genuinely needed. Use `okf resolve <link> <bundle> --from <concept-id>` for a
   particular raw link.
6. Join claim footnotes to `sources[].id`. Cite the declaring concept and relevant source or
   resolved artifact separately. Do not fill bundle gaps with assumptions.

If a search finds nothing, try a broader token query with `--match any`, then a relevant type, tag,
or directory if known. Stop when the question is answered or these bounded alternatives provide no
useful leads. Report the search scope and missing evidence; failed queries alone do not establish
that the entire bundle lacks an answer.

## Trust and lifecycle decision

Use the returned lifecycle/trust metadata for material claims; inspect further only where it could
change the answer. Consider:

1. effective `status` (`stable` when absent; distinguish draft and deprecated);
2. whether `stale_after` has expired at the current instant;
3. the latest valid `verified.at` and derived tier;
4. whether `generated.at` is later than that verification;
5. relevant source credibility/usage signals and per-claim source IDs;
6. for computed values, whether this actual run has a passing runtime attestation.

Trust is advisory: stale, deprecated, or unverified knowledge may still be useful, but report
limitations that materially affect the answer. An Attested Computation definition can exist and be
document-verified without any run having occurred. `okf computation check <concept-id> <bundle>` is
inspect-only and reports `execution: not-run`; never describe that as a passing attestation.

## Artifact retrieval and `references/`

`references/` is an optional convention, not required. Markdown files beneath it are ordinary
concepts; non-Markdown files are opaque artifacts. Path-valued fields may be URLs, bundle-relative,
or document-relative. Resolve with `okf artifact resolve <resource> <bundle> --from <concept-id>`
and report `concept`, `artifact`, `reserved`, `external`, `scope`, `missing`, and `blocked`
distinctly. Use `okf show` for concepts and bounded `okf artifact show <resource> <bundle> --from
<concept-id> --lines <START:END> --max-bytes <N> --json` for artifacts. Never force SQL/Python
through `okf show`.

For path namespaces, external sources, and truncated reads, consult the artifact sections of
`references/cli.md` when needed. Bundle artifact reads stay inside the bundle; existing user
authorization to inspect a named external source also counts. Reading code never authorizes
execution.

Answer with the concepts actually opened, source/artifact citations, precise caveats, and a precise
statement of what could not be established from the material inspected. Do not mutate the bundle.

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

## Structured retrieval and snapshot evidence

Use the shared selector grammar for nested queries: `norms[].deadline.from` traverses explicit
list items, while `["policy.status"]` names a literal dotted key. Existing literal-key field
filters keep their meaning. Read `references/cli.md` for typed conditions, projections, facets,
pages, and related-concept expansion. Conditions combine with AND; list membership alternatives
belong inside one condition. Separate nested conditions can match different list records: bearer
and deadline matches do not establish one obligation with both. Inspect the projected whole record
or semantic edge's concrete occurrence and attributes before making that claim. Apply filters
before pagination and retain reported completeness.
JSON facets describe all examined matching documents, not only the displayed page; missing/null
and overflow notices are evidence gaps, not empty vocabularies. Display cells and computed view
records do not replace authored frontmatter.

Only declared reference selectors create custom metadata edges. Relationship attributes must
come from the same concrete list occurrence as their target. Inverse backlinks present incoming
assertions with their conditions; they do not assert acceptance or fulfillment by the target.
Expansion is one-hop outbound, bounded, and reports missing, out-of-scope, unavailable, or
truncated context. Check expansion completeness separately from primary scan and facet completeness.
Result/page limits bound output, not scan work. Each offset invocation rescans its scope without
shared query caches; avoid many tiny pages for exhaustive analysis. Aggregate a complete metadata
stream locally when needed and retain its scope/settings, rather than loading every record into
context. Facets cost aggregation work; request them only when useful. Working-tree pages can shift.

For source entries carrying `bundle_ref`, distinguish ordinary target resolution from requested
snapshot satisfaction. Report matched, unverified, unavailable, or mismatched expectations and
label current candidates separately. A candidate does not fulfill a known mismatch. Historical
bytes, digest matches, or configured correspondence do not verify a concept's claims. Do not
fetch Git objects, accept a candidate, or refresh fingerprints during retrieval.

Qualified graph targets use `acme.finance:/policies/margin-standard`. Historical inspection uses
`--revision <ref>` on graph, links, backlinks, resolve, affected analysis, search, or list; it is not a general
artifact, mutation, or lint snapshot flag. Mutable working-tree versions remain identified
as such. Each target's interpretation settings and unavailable revisions limit the answer.

For example, `okf search <bundle> --facet-filter 'norms[].deadline.from="account-created"'
--facets --limit 20 --offset 0 --json` selects a typed nested condition and reports the first
page alongside facets. Use `okf search <bundle> --project '$id' --project 'norms[].deadline'
--limit 10 --json` when only those cells are needed. Keep literal JSON strings quoted within
the condition; JSON booleans and numbers keep their types.

An explicitly selected view with `extended_output: true` opts JSON into its configured columns
as separate projection records and its configured expansion. Ordinary views preserve legacy
machine concept output. Confirm that opt-in before interpreting a JSON stream as concept records.

When a read fails to parse concept YAML, use the file paths in its error to locate the failing documents. Full bundle reads report all malformed files they examined; bounded queries report only examined files. Use `okf validate <bundle> --json` for separate per-file conformance findings.
