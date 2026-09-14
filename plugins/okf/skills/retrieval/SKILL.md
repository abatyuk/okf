---
name: retrieval
description: Answer questions from an OKF bundle through progressive disclosure, source lineage, bounded artifact retrieval, and explicit lifecycle/trust caveats. Use before reading bundle files directly; this skill is read-only.
---

# Retrieve knowledge from an OKF bundle

Query the bundle via the CLI rather than bulk-reading it. Use `OKF_BUNDLE` for multi-step work;
explicit examples are `okf show <concept-id> <bundle>` and `okf links <concept-id> <bundle>
--json`. When exact flags or output shapes are needed, read the focused `references/cli.md`. If
its generated tool version differs from the installed binary, use that command's `--help` output
as the runtime authority.

## Progressive disclosure

1. Start with `okf browse <bundle>` unless the request already names a concept or exact filter.
   Descend with `--directory`; search with `okf search <bundle> --text <terms>` or structured
   filters. Text phrases are case-insensitive and match reader-visible Markdown across inline
   formatting and line wraps. Repeat `--text` when every phrase must occur; use `--match all`
   for every token, `--match any` for discovery, and `--match literal` only for exact Markdown
   source. Narrow noisy queries with `--in id,title,description,body,frontmatter`. Keep default
   relevance ordering for discovery; use `--sort id` for stable inventory-style processing and
   `--limit <N>` to bound context.
2. With `--json`, use each result's `search.score` and `search.matches` to choose candidates.
   Match evidence identifies the field, bounded snippet, and a serialized document line when
   applicable; use that line to request a small `show --lines` window. Treat snippets as
   navigation evidence, not enough context to answer from.
3. For candidates, use `okf show <concept-id> <bundle> --outline`, then `--lines <START:END>`.
   Read a full concept only when small.
4. Follow portable body links and standard internal `sources[].resource` lineage even without
   ontology. Start with `okf links <concept-id> <bundle> --json` for direct outbound targets and
   `okf backlinks <concept-id> <bundle>` for direct inbound concepts. Render only the useful
   neighborhood with `okf graph <bundle> --root <concept-id> --direction <outgoing|incoming|both>
   --depth <N> --format mermaid`; omit the root only when the entire graph is genuinely needed.
   Use `okf resolve <link> <bundle> --from <concept-id>` for a particular raw link.
5. Join claim footnotes to `sources[].id`. Cite the declaring concept and relevant source or
   resolved artifact separately. Do not fill bundle gaps with assumptions.

## Trust and lifecycle decision

For every material answer, consider separately:

1. effective `status` (`stable` when absent; distinguish draft and deprecated);
2. whether `stale_after` has expired at the current instant;
3. the latest valid `verified.at` and derived tier;
4. whether `generated.at` is later than that verification;
5. relevant source credibility/usage signals and per-claim source IDs;
6. for computed values, whether this actual run has a passing runtime attestation.

Trust is advisory: stale, deprecated, or unverified knowledge may still be useful, but label each
limitation. An Attested Computation definition can exist and be document-verified without any
run having occurred. `okf computation check <concept-id> <bundle>` is inspect-only and reports
`execution: not-run`; never describe that as a passing attestation.

## Artifact retrieval and `references/`

`references/` is an optional convention, not required. Markdown files beneath it are ordinary
concepts; non-Markdown files are opaque artifacts. Path-valued fields may be URLs,
bundle-relative, or document-relative. Resolve with `okf artifact resolve <resource> <bundle>
--from <concept-id>` and report `concept`, `artifact`, `external`, `scope`, `missing`, and
`blocked` distinctly. Use `okf show` for concepts and bounded `okf artifact show ... --lines
<START:END> --max-bytes <N>` for artifacts. Never force SQL/Python through `okf show`.

Local reads must remain inside the canonical bundle after symlinks. Remote `--fetch` requires
explicit authorization and configured policy and must not send ambient credentials. Reading
code, executor instructions, or an attester never authorizes execution.

Answer with the concepts actually opened, source/artifact citations, precise caveats, and an
explicit statement when the bundle does not contain the answer. Do not mutate the bundle.
