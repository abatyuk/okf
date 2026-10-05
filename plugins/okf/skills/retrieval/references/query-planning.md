# Plan bounded, evidence-backed queries

Choose the smallest examination scope that answers the user's question. Inspect a configured
catalog once, retain effective identities/settings, and use explicit bundle IDs thereafter.
Add named bundles when the question needs them; catalog registration alone does not select them.

## Choose the operation

| Evidence needed | Preferred route |
| --- | --- |
| Known concept | Direct `show`; metadata only with `--json`, outline/lines for long prose |
| Known metadata condition | `search --facet-filter` with typed operands |
| Unknown vocabulary/distribution | One query with selected JSON facets; inspect exclusions/completeness |
| A few fields on primary hits | `--project`; handle separate projection records |
| Metadata on outbound targets | Selected `--expand` rules and `--target-field` selectors |
| Incoming assertions and their conditions | `backlinks --details` in the necessary scope |
| Supporting prose | Open only relevant concepts after metadata identifies them |

Read the selected bundle's effective ontology to find actual relationship rule names. An ordinary
path-looking field does not create an expansion rule. Expansion is one hop, outbound only, with
deduplicated target payloads and distinct edge occurrences. It does not fetch target bodies.

For a catalog with `acme.product` and `acme.finance`, and a declared `policy-relations` rule:

```sh
okf catalog --json
okf search --bundle-id acme.product --scope-bundle acme.finance --type Procedure --project '$id' --project 'relations' --expand policy-relations --target-field title --target-field description --limit 20 --offset 0 --json
```

Read `query-summary`, `scope`, `relationship`, `related-concept`, `expansion-summary`, and `warning`
records by kind. Do not treat every NDJSON line as a primary concept. Join related payloads using
qualified bundle/concept/version identity, not an unqualified ID. Projection adds records; it does
not promise to remove all legacy metadata from the stream or reduce scan cost.

## Complete the scan only when the answer needs it

The default budget examines 1,000 eligible documents across the selected scope. The primary bundle
supplies the query-wide configured budget. Filters and `--limit 10` do not promise ten document
reads: exact mode scans to establish counts and ordering, subject to its budget.

1. Inspect `scan_complete`, `partial`, warnings, and examined/unavailable scope before interpreting
   an empty result or a total. `total_matches: null` means unknown, not zero. Observed matches are
   not a complete inventory. An incomplete scan cannot establish a globally ordered page.
2. If the question is already answered by inspected evidence, stop and qualify the scope. If it
   requires absence, exact totals, or exhaustive inference, reduce scope only if that still answers
   the question; otherwise increase `--scan-limit` or deliberately use `--full-scan`.
3. Re-run the same query and inspect completeness again. A larger finite budget may still exhaust.
   Do not increment `--offset` to recover from scan exhaustion; incomplete scans have no
   authoritative continuation. `--full-scan` conflicts with `--scan-limit` and leaves result,
   facet, expansion, and availability bounds in force.

```sh
okf search knowledge/product --type Procedure --scan-limit 10000 --limit 20 --offset 0 --json
okf search knowledge/product --type Procedure --full-scan --limit 20 --offset 0 --json
```

Primary scan completeness, facet aggregation/presentation, and expansion completeness are separate.
A complete scan with a truncated page still has exact totals. A complete scan with truncated
expansion does not mean all related evidence was returned. Missing facet values may reflect a
high-cardinality exclusion or output bound. Increase only the bound needed for the question, or
open the specific unresolved target; retain gaps that remain.

## Avoid repeated work

Each offset request independently scans its scope; there are no shared query caches or saved result
sets. Use pagination for bounded presentation, not as a scan optimization. For exhaustive analysis,
prefer one complete metadata stream with no result limit, aggregate it locally, and bring only the
useful summary into context. Retain its query, scope, revision, interpretation, and warnings.
Do not discard warnings when filtering records for aggregation.

Request facets only when their distributions help. Use selected expansion instead of one `show`
call per target when related metadata suffices. Reuse already inspected results while their
content/settings remain applicable. If pages are necessary, keep filters, scope, sort, and
interpretation fixed. Working trees can change between pages; immutable available revisions help,
but external interpretation settings must also remain unchanged.

## Same-record conditions require inspection

Suppose one procedure contains:

```yaml
norms:
  - bearer: /teams/platform.md
    deadline: {within: 72, unit: hours}
  - bearer: /teams/security.md
    deadline: {within: 24, unit: hours}
```

This query matches the document:

```sh
okf search knowledge/product --facet-filter 'norms[].bearer="/teams/platform.md"' --facet-filter 'norms[].deadline.within=24' --project norms --json
```

It does **not** establish that Platform has a 24-hour deadline. Each condition is existential and
can match a different record; correlated predicates are unsupported. Inspect the projected whole
`norms` records or an expanded semantic edge with its concrete field path and paired attributes.
Here Platform's recorded deadline is 72 hours and Security's is 24. If only sampled evidence is
available, report that limitation before making an exhaustive claim about obligations.

Separate `tags[]="security"` and `tags[]="production"` conditions require both tags. One
`tags[] in ["security","production"]` condition accepts either. `not in` checks the entire list;
missing/null fields do not satisfy it, while a present empty list can. Typed operands do not coerce
strings to numbers or booleans.
