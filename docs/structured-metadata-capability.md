# Structured metadata capability

Status: implemented local capability. The design below records the capability contract and
its deliberate limits; implemented CLI options and configured bounds are recorded at the end.
Illustrative records in the design are examples; `okf schema` and the generated reference expose
the current machine contract.

This capability complements [multi-bundle capability](multi-bundle-capability.md), retaining
its bundle selection, identity, ordinary path, snapshot, and scope semantics.

## Motivation

Custom metadata often carries the most useful parts of a knowledge base: dependencies,
ownership, relationships, and policy obligations. Accepting that metadata as YAML is only
the first step. Authors need to describe its structure, check nested values, identify which
fields reference concepts, and retrieve related information without reading every document.

For example, onboarding may implement an access policy and assign the Platform team an
access-review obligation. Readers will be able to find the responsible team, action,
trigger, completion condition, and deadline. They will also be able to see the procedures
implementing a policy through backlinks and narrow large result sets using facets and pages.

The ontology already represents reusable types, inheritance, nested objects, and lists;
the main gaps are recursive enforcement, nested reference extraction, semantic graph edges,
and configurable retrieval. Existing index generation also does not detect omissions in
manually maintained indexes.

The capability retains ordinary Markdown and frontmatter. Optional local configuration
adds meaning and checks without introducing a new document format, mandatory ontology,
central service, or policy execution engine.

## 1. Capability and restrictions

The CLI will support structured metadata within one bundle and within the explicitly
selected scope of a multi-bundle catalog. Each bundle owns its ontology and retrieval
settings. Registered bundles retain the identity, path resolution, version evidence, and
snapshot-candidate behavior defined by the multi-bundle capability.

The capability adds:

- Reusable `field_types` containing nested structures, references to other field types,
  and inheritance; effective definitions can be inspected.
- Recursive metadata linting for values, vocabularies, required properties, collection
  bounds, patterns, uniqueness, and explicitly closed custom objects.
- Explicit nested concept-reference selectors and configurable semantic relationship kinds.
- Backlinks presenting named inverse relationships with their authored conditions.
- Read-only checks for concepts and concept-containing directories omitted from indexes.
- Configurable display fields, explicit projections, and bounded related-concept expansion.
- JSON-only facets over all matching documents, typed nested filters with AND and list
  membership semantics, and pagination.

### Restrictions introduced by the CLI

These rules apply to local tooling, not to OKF conformance:

| Supported behavior | Purpose and effect |
| --- | --- |
| Explicitly declared reference fields | Ordinary resource paths and strings do not accidentally become graph edges |
| Exact named types and declared vocabularies | Lint checks local meaning without inferring terms from spelling |
| Configuration errors for invalid schemas and type cycles | Broken configuration cannot masquerade as valid concept checks |
| Metadata open by default | Extensions survive; closure applies only to explicitly closed custom objects |
| Index coverage based on resolved Markdown links | Prose mentions and links only to descendants do not falsely satisfy immediate-child coverage |
| AND across query conditions | Explicit `in` supplies alternative values inside one condition |
| Bounded expansion with visible completeness | Retrieval costs and missing context remain visible |
| Reported bundle/version scope | Results do not claim coverage of unavailable or unexamined bundles |

Ontology and index findings warn by default under the new rules; CI can configure stricter
lint severity. Existing rules retain their compatibility behavior. Reading, linting, and
querying do not rewrite metadata, indexes, fingerprints, or verification events.

### Relationship to OKF

This capability is an optional CLI layer. It does not introduce stricter document conformance
or change standard concept IDs, source resources, or exact `Attested Computation` semantics.

| Addition | Effect on compatibility |
| --- | --- |
| Ontology and bundle settings | Optional tool configuration, never bundle boundary markers |
| Nested constraints | Advisory lint; `okf validate` retains its conformance responsibility |
| Semantic relationship mapping | Interprets declared custom fields without changing authored paths |
| Named inverse backlink | A view of an incoming edge, not a new assertion authored by the target |
| Index completeness | Local navigation policy; optional indexes do not become mandatory OKF content |
| Views, facets, and pages | Computed query output, kept separate from authored frontmatter |

Unknown metadata and ontology extensions remain preserved. Missing or broken references
remain consumable and diagnosable. Unsupported constraints will be identified as unenforced,
never silently reported as checked. Rejecting local configuration does not declare the
underlying content nonconformant.

## 2. Files, references, and behavior changes

### Existing configuration and per-bundle settings

Settings will use `okf.toml` and the catalog described by the multi-bundle capability:

```toml
# okf.toml
catalog = "okf-catalog.yaml"
default_bundle = { id = "acme.product" }

[bundle_settings."acme.product"]
ontology = "tooling/product-ontology.yaml"

[bundle_settings."acme.product".lint]
index_exclude = ["drafts/**"]

[bundle_settings."acme.product".query]
scan_limit = 1000

[bundle_settings."acme.product".views.compact]
columns = ["$id", "stable-id", "description", "tags"]

[bundle_settings."acme.product".facets]
fields = ["type", "tags[]", "norms[].deadline.from"]
max_distinct_values = 100
include_high_cardinality = []
```

```yaml
# okf-catalog.yaml
catalog_version: 1
bundles:
  acme.product:
    location:
      type: directory
      path: knowledge/product
  acme.finance:
    location:
      type: directory
      path: knowledge/finance
```

Catalog and ontology paths resolve relative to `okf.toml`; bundle locations resolve relative
to the catalog. Settings do not register bundles. Explicit command options override applicable
bundle settings, which override built-in defaults. An unregistered path never borrows another
bundle's named settings. For uncataloged single-bundle use, `[bundle_settings.default]` is
the proposed settings table; `default` is reserved for that purpose in the settings namespace.

An unreadable explicitly configured ontology produces a configuration error. Without one,
the optional `<bundle-root>/ontology.yaml` discovery fallback remains available. The ontology
does not establish a bundle boundary. The multi-bundle proposal's distinct ID/path selectors,
`OKF_BUNDLE` path meaning, typed default selector, and legacy path-only `bundle`
configuration alias remain supported.

Effective settings will be inspectable, including the matched bundle identity and ontology
path. Each target bundle uses its own ontology; source definitions are not applied to its
metadata. The explicit sidecar setting and root fallback make both currently documented
tool-local placement and existing loader placement available without relocating files.

Historical reads resolve in-repository ontologies at the examined revision, including a
configured sidecar within that repository. External ontology sidecars use their available
local content and report the effective path and content digest as interpretation context.
Unavailable historical configuration is diagnosed rather than silently replaced by current
configuration. Query output identifies the effective interpretation settings so immutable
concept content is not mistaken for immutable query behavior.

### Shared selector syntax and implementation

Accepted design: use a small custom selector parser and walker operating directly on the
existing `serde_yaml::Value` metadata. No JSONPath, JMESPath, or jq/yq engine dependency or
JSON query copy is introduced. JSON scalar and array filter operands use the existing
`serde_json` parser; filter evaluation and occurrence pairing remain tool-owned logic.

The same shortened selector syntax applies to declared references, relationship kind and
attribute selectors, display columns, projections, facets, and new selector-aware filters:

| Selector | Meaning |
| --- | --- |
| `title` | Top-level property |
| `deadline.within` | Nested object property |
| `tags[]` | Each item of a list |
| `norms[].deadline.from` | Deadline anchor within each norm record |
| `["policy.status"]` | Literal top-level key containing a dot |
| `norms[]["action.name"]` | Literal dotted key within each norm record |

There is no leading JSONPath `$` and no `[*]` spelling. List traversal requires explicit
`[]`; `norms.deadline.from` does not implicitly traverse `norms`. Selecting a list itself,
such as `tags` for a display cell or projection, remains distinct from selecting its items.
Bracket-quoted keys use JSON string escaping. Computed display/projection names such as
`$id` and `$bundle` remain separate from authored-property selectors.

The subset excludes recursive descent, indexing/slices, selector unions, embedded predicates,
functions, pipes, and executable expressions. Malformed or unsupported selector syntax is
diagnosed rather than delegated to another expression engine. The walker retains concrete
occurrence paths and container presence so same-record relationship pairing and missing,
null, and empty-list states are not lost by flattening selected values. Detailed filter
edge cases remain subject to clarification; accepting this syntax does not settle them.
Existing literal-key `--field` behavior and reference rules without a selector remain
compatible.

### Reusable types and nested linting

One ontology can define a relationship object and obligation schema using other named types:

```yaml
# tooling/product-ontology.yaml
okf_ontology: "0.1"
field_types:
  RelationKind:
    base: enum
    values: [implements]
  Relation:
    base: object
    fields:
      kind: {type: RelationKind, required: true}
      target: {type: string, required: true}
      qualifier: {type: string}
  ClosedRelation:
    extends: Relation
    additional_properties: false
  Modality:
    base: enum
    values: [obliges, forbids, permits]
  RelativeDeadline:
    base: object
    additional_properties: false
    fields:
      within: {type: int, required: true, min: 1}
      unit: {type: enum, values: [hours], required: true}
      from: {type: enum, values: [account-created], required: true}
  Norm:
    base: object
    additional_properties: false
    fields:
      modality: {type: Modality, required: true}
      bearer: {type: string, required: true}
      action: {type: string, required: true}
      trigger: {type: string, required: true}
      discharge: {type: string, required: true}
      deadline: {type: RelativeDeadline}
concepts:
  Policy: {}
  Team: {}
  Procedure:
    fields:
      relations: {type: list, item: ClosedRelation, min: 1}
      norms: {type: list, item: Norm}
      tags: {type: list, item: string, unique_items: true}
      evidence-file: {type: string}
    references:
      policy-targets:
        selector: "relations[].target"
        target: Policy
        cardinality: 0..n
      obligation-bearers:
        selector: "norms[].bearer"
        target: Team
        cardinality: 0..n
    relationships:
      policy-relations:
        reference: policy-targets
        kind_selector: "relations[].kind"
        kinds:
          implements: {inverse: implemented-by}
        attributes:
          qualifier: "relations[].qualifier"
      obligations:
        reference: obligation-bearers
        kind_selector: "norms[].modality"
        kinds:
          obliges: {inverse: obligated-by}
          forbids: {inverse: prohibited-by}
          permits: {inverse: permitted-by}
        attributes:
          action: "norms[].action"
          trigger: "norms[].trigger"
          discharge: "norms[].discharge"
          deadline: "norms[].deadline"
```

`Norm` composes `Modality` and `RelativeDeadline`; `ClosedRelation` inherits `Relation`.
Inherited constraints use most-derived precedence. Explicit `fields`, `values`, or `item`
settings replace inherited counterparts rather than silently merging field maps. Cycles
across inheritance and nested references are configuration errors. Referencing a field type
does not itself create a concept reference or relationship edge.

Replacement can remove inherited properties: a derived `fields: {note: {type: string}}`
replaces all inherited fields, including mandatory ones. In a closed derived object, only
the replacement fields are allowed. Authors must repeat inherited fields they wish to keep.
`extends` does not promise substitutability. Changing an inherited base type is a
configuration error. Validate the complete effective definition after inheritance resolution;
for example, inherited `min: 5` combined with derived `max: 3` is invalid.

Named types will support composition and inheritance at every nested level, including inline
objects whose children reference other named types and lists whose items reference named
types. Updating a shared definition changes the effective validation of all its users within
that ontology without rewriting their values. `ontology show` will expose resolved nested
structure, inherited constraints, and the named definitions they came from. Structured YAML CLI inputs support these definitions; see the implemented authoring interface below.

Recursive lint validates primitives without coercion, exact enum values, object children,
list items, patterns, bounds, and uniqueness:

| Declaration | Supported check |
| --- | --- |
| `string`, `text` | YAML string, without number/boolean coercion |
| `int` | Integral YAML number representable as a signed 64-bit integer |
| `bool` | YAML boolean |
| `date` | Valid Gregorian `YYYY-MM-DD` string |
| `datetime` | RFC 3339 string with explicit timezone |
| `uri` | Syntactically valid absolute URI; no network or existence check |
| `enum` | Exact string membership in resolved `values` |
| `object` | Mapping with string keys and recursively checked children |
| `list` | Sequence with recursively checked `item` values |
| `pattern` | Rust regex search semantics; anchors express full-string matching |
| `min`, `max` | Inclusive integer bounds, string Unicode-scalar length, or list length |
| `unique_items: true` | Structural YAML equality, ignoring mapping key order |
| `additional_properties: false` | Undeclared keys reported in this custom object |

Missing optional fields are skipped. A present null fails its declared type. For
`required: true`, absent/null values, whitespace-only strings, and empty lists fail;
empty objects count as present and their child constraints apply. Missing optional parents
do not make their children mandatory. Container type failures stop child checking at that
occurrence, avoiding misleading missing-child findings; independent failures remain visible
and duplicate required/null findings are suppressed. Root metadata remains open.

Effective ontology inspection and lint illustrate the intended behavior:

```sh
okf ontology show Procedure knowledge/product
okf lint knowledge/product --json
```

Illustrative finding if an obligation has `deadline.within: 0`:

```json
{"kind":"finding","rule":"ontology-violation","code":"metadata-range","severity":"warn","bundle":"acme.product","concept":"/procedures/onboarding","field_path":"norms[0].deadline.within","message":"Expected integer >= 1; found 0"}
```

Malformed recognized constraints, incompatible constraint/type combinations, `min > max`,
missing item types, empty enum vocabularies, unknown named types, and cycles produce
configuration errors. Patterns are checked when loading the ontology. Unknown extension
constraints remain preserved with an unenforced notice.

Findings will retain the existing `ontology-violation` rule alongside detailed codes such as
`metadata-required`, `metadata-type`, `metadata-enum`, `metadata-pattern`, `metadata-range`,
`metadata-duplicate`, and `metadata-unknown-property`. They identify the bundle/concept,
concrete occurrence path, expected constraint, and bounded actual value or type. Without an
ontology, ontology lint is skipped. Depth or finding-budget exhaustion produces an
incomplete-check diagnostic; the provisional depth bound is 64, with a configurable finding
budget. Lint never silently reports complete success after stopping.

### Authored references and obligations

Example concept `knowledge/product/procedures/onboarding.md`:

```markdown
---
type: Procedure
title: Account onboarding
stable-id: PROC-017
description: Account creation and access review
tags: [security, production]
relations:
  - kind: implements
    target: /policies/access-control.md
    qualifier: production
norms:
  - modality: obliges
    bearer: /teams/platform.md
    action: review-account-access
    trigger: account-created
    discharge: access-reviewed
    deadline:
      within: 24
      unit: hours
      from: account-created
evidence-file: evidence/onboarding.csv
---
Create the account and arrange its access review.
```

This asserts that Platform must review each created account's access within 24 elapsed
hours of creation. `action` names the required activity; `trigger` starts the obligation;
`discharge` names completion; `deadline.from` names the clock anchor. Creation at 09:00 UTC
Monday makes the review due by 09:00 UTC Tuesday. There is no business-hours exception.
Without a deadline, none is declared. A late review does not establish timely compliance.

These are local policy terms. OKF can check the declaration and retrieve its conditions;
it does not ingest account events, correlate reviews, calculate live due dates, or monitor
fulfillment. A vocabulary check also does not imply a cross-field equality rule between
`trigger` and `deadline.from` unless one is explicitly defined in a future policy mechanism.

Only declared targets become custom concept references. `evidence-file` stays an ordinary
string. Leading `/` is source-bundle-root relative; bare and `../` paths resolve relative
to the containing document. An ordinary path crossing into another registered bundle uses
the shared multi-bundle resolver. Above-root traversal is preserved and never collapsed into
an unrelated local ID.
Missing, unavailable, wrong-type, and out-of-scope targets have distinct diagnostics.

Initial nested references contain ordinary path strings only; they carry no snapshot
expectations and do not inherit them from provenance sources. Snapshot-mismatch and candidate
handling applies where the shared resolver receives an explicitly supported snapshot request,
initially a provenance source entry with `bundle_ref`.

Target type restrictions are qualified by the bundle declaring the type. An unqualified
`target: Policy` denotes the source bundle's declaration; another bundle's `Policy` is not
equivalent merely because its name matches. Cross-bundle restrictions require an explicit
qualified declaration or equivalence mapping. A resolved target satisfies that restriction
through the exact declaration or its declared concept-type inheritance; field-type
inheritance alone does not establish concept subtyping. Qualification and mapping syntax
remain provisional.

Reference rules will support an optional `selector`. Without it, the rule key retains its
literal top-level-field meaning, including dotted keys, and accepts a scalar string or list
of strings. Explicit selectors support nested object keys, `[]` list traversal, and bracket
quoting such as `["a.b"][]`; selected target leaves are nonempty strings. Existing
extensionless concept references remain supported. Cardinality counts authored target
occurrences, including unresolved targets, rather than deduplicated neighbors. Missing
branches contribute no occurrences; nested mandatory values are checked by field declarations.

Missing target warnings under the new reference checks remain configurable for CI; body-link
rules retain their existing severity. One occurrence will not produce duplicate broken-link
and nested-reference findings. Out-of-scope resolution remains distinct from a proven missing
file, and candidate content does not satisfy requested-version target-type checks.

### Semantic edges and backlinks

Relationship mappings pair kind, target, and attributes within the same list record.
List occurrence anchors retain that pairing instead of independently flattening values.
Fixed-kind relationships, such as a declared `depends-on` list, are also supported by the
design. Distinct occurrences and kinds to one target remain distinct semantic edges.

For example, alongside an appropriate top-level reference rule:

```yaml
dependencies:
  reference: depends-on
  kind: depends-on
  inverse: dependency-of
```

Mappings will support either a fixed `kind` or a `kind_selector` with a `kinds` map.
Ambiguous/mismatched list anchors and contradictory mappings produce configuration errors.
Dynamic kind vocabularies agree with declared enums; per-kind target restrictions can narrow
the reference rule's allowed target types. Unknown dynamic kinds produce a finding and an
unresolved semantic occurrence while the underlying declared target remains visible. Initial
support includes one scalar target per nested record and scalar/list top-level fixed-kind
references; multiple targets inside one nested relationship record are deferred.

Each edge retains source identity/version, rule, metadata path, raw reference, kind, inverse,
attributes, and resolution evidence. Semantic self-edges remain available even where legacy
adjacency excludes self-links; concept-neighbor views can continue deduplicating targets.

The `backlinks` command explicitly exposes configured inverse names. Proposed detailed mode:

```sh
okf backlinks /policies/access-control knowledge/product --details
okf backlinks /teams/platform knowledge/product --details
```

Illustrative human output:

```text
/policies/access-control —implemented-by→ /procedures/onboarding
  authored kind: implements
  source field: relations[0].target
  qualifier: production

/teams/platform —obligated-by→ /procedures/onboarding
  authored kind: obliges
  source field: norms[0].bearer
  action: review-account-access
  trigger: account-created
  discharge: access-reviewed
  deadline: within 24 hours from account-created
```

Detailed machine output also retains declaring bundle/rule, source version, attributes, and
resolution evidence and distinct incoming occurrences. Legacy concept-only backlinks remain
compatible. Ordinary body/source backlinks and references without inverse names remain
available, labeled with their forward kind when known, without invented reverse kinds.
Inverse labels are derived views, not reverse links authored into target documents.

Cross-bundle backlinks report examined bundles and versions. Unexamined consumers remain
unknown. Matching kind labels from different bundle ontologies do not establish equivalent
vocabularies. Machine output identifies the declaring bundle/rule so cross-bundle queries can
select the intended declaration. Affected analysis will include declared nested targets.
Relationship-rule selectors, including expansion options, are likewise qualified by declaring
bundle. Short forms select rules in the primary source bundle; multi-bundle queries require
explicit qualification where declarations differ. Exact selector syntax remains provisional.

Moves that rewrite declared structured references are out of scope for this capability.
Any future support must preflight exact occurrences before mutation, preserve surrounding
YAML and attributes, and diagnose unsupported rewrites before writing. Cross-bundle
move/update orchestration remains deferred; this proposal makes no automatic rewrite promise.

### Detecting omissions in existing indexes

The check compares each eligible directory's immediate children with links in its actual
`index.md`. Coverage includes concepts, their ancestor directories, and the root. It excludes
reserved `index.md`/`log.md`, opaque artifacts, `.git`, artifact-only directories, and configured
index exclusions. Hidden and Git-ignored concepts follow the physical-tree walker. Exclusions
affect this check only, not bundle boundaries, search, graphs, or validation.

Index exclusions use bundle-relative POSIX globs. Excluding a directory excludes its subtree
from coverage; eligible ancestors are recomputed afterward so excluded-only subtrees do not
produce parent omissions. Symlinks are not followed. Coverage uses the generator's concept
and ancestor inventory rather than comparing generated and authored text byte for byte.

```text
policies/
  index.md             links only to access-control.md
  access-control.md
  retention.md         omitted
  exceptions/
    temporary.md       makes exceptions/ an eligible directory
```

```sh
okf lint knowledge/product --json
```

Illustrative coverage findings, among other applicable lint findings:

```json
{"kind":"finding","code":"index-missing-entry","severity":"warn","directory":"policies","missing_child":"retention.md"}
{"kind":"finding","code":"index-missing-entry","severity":"warn","directory":"policies","missing_child":"exceptions/"}
```

A descriptive Markdown link counts; a filename mentioned in prose does not. A directory
link or link to its `index.md` covers that child directory; a link only to a deeper concept
does not. Inline and reference-style links count, including `.md`, accepted extensionless
spellings, fragments, and consistently decoded URL-escaped paths. Images, code blocks,
frontmatter strings, remote destinations, and links to another bundle do not establish
immediate-child coverage. Duplicate entries are not completeness failures; stale/broken
entries remain link-check concerns. Findings sort by bundle, directory, child, and code.

A missing eligible index produces one `index-missing` finding instead of one per child.
Unreadable indexes produce I/O diagnostics instead of successful empty coverage.
Synthesized browsing does not satisfy an on-disk index check. Lint does not regenerate
indexes, and existing generation can replace an authored index body.

### Display fields, projections, and expansion

Display fields choose compact table columns. Proposed command and output:

```sh
okf search knowledge/product --type Procedure --view compact
```

```text
ID                      STABLE-ID  DESCRIPTION                         TAGS
/procedures/onboarding   PROC-017   Account creation and access review   security, production
```

Nested selectors use dot notation. `$id` and `$bundle` name computed values; bracket quoting
addresses literal custom keys. Missing human cells show `—`; list values retain order and
cell truncation is visible. Object cells use bounded compact JSON, with newlines and table
delimiters escaped. Named views and explicit `--columns` will support `list`, `search`, and
metadata-only query tables. Full-body `show`, indexes, and diagnostics retain their own views.

Explicit `--project` output will use a separate record containing qualified identity,
selector-to-value mappings, and missing selectors, preserving authored nulls distinctly from
missing fields. Computed-name collisions remain separate from authored metadata. Changing
columns or projection does not change filtering, ranking, or graph semantics; displaying a
reference value as a column does not fetch its target.

Proposed expansion follows selected outbound relationship rules and returns selected target
metadata alongside the primary match:

```sh
okf search knowledge/product --type Procedure --expand policy-relations --expand obligations
```

```text
MAIN: /procedures/onboarding — Account onboarding
implements → /policies/access-control
  title: Access control
  description: Rules for granting and reviewing access
obliges → /teams/platform
  title: Platform team
  action: review-account-access
  deadline: within 24 hours from account-created
```

Targets need not match the search and do not consume primary page slots. Expansion is
explicit, one hop, outbound only. Default target fields are title, type, and description;
target bodies are not returned by default. Provisional bounds are 10 edge occurrences per
hit, 100 distinct targets and 256 KiB of expansion payload per query. Distinct edge occurrences
are preserved while target payloads are deduplicated by qualified bundle/concept/version identity.
Missing targets retain status entries; truncated output identifies bounds and reasons.

Per-bundle named views will also support explicit human expansion defaults, overridden by
`--no-expand`. Machine expansion remains opt-in through explicit expansion options or an
explicit extended-output view. Target field selection uses the same selectors as projection.
Bounds will be configurable within documented limits; identities, values, and status entries
all count toward the byte budget. Ordering follows primary hits, relationship declarations,
authored occurrences, then qualified identity. Output reports emitted counts, applied bounds,
`truncated`, and reasons; totals are exact only when completely measured, otherwise unknown.

Expansion stays within selected bundle/version scope and never fetches remote material.
Unavailable and out-of-scope targets retain their raw reference and status. Initial
relationship expansion follows ordinary string references and does not emit implied snapshot
requests or candidates. Working-tree content is identified as mutable. Visited-node tracking
bounds graph traversal in the presence of cycles.

Legacy JSON concept output remains unchanged without new options. Explicit machine projection
and expansion use separate versioned `projection`, `relationship`, `related-concept`, and
summary records; computed values do not overwrite frontmatter. JSON expansion requires
explicit opt-in, and configured human columns never silently project machine output.
NDJSON records will associate primary hits, edges, and related payloads without repeating
every target. `okf schema` will describe the new query controls and output record shapes.

### Query scan budget and completeness

Exact query mode is the default, subject to a default scan budget of 1,000 eligible concept
documents across the selected examination scope. The budget counts examined documents, not
matches or emitted records. `okf.toml` may override it through applicable query settings;
an explicit positive `--scan-limit N` takes precedence over configuration. Uncataloged use
uses `[bundle_settings.default.query]`. For a multi-bundle query, the primary bundle's query
settings supply the single query-wide budget, rather than adding per-bundle budgets.

Explicit `--full-scan` removes the document-count scan budget for that invocation. It cannot
be enabled through configuration and conflicts with an explicit `--scan-limit`; malformed
configuration attempting to enable it is diagnosed. It does not remove result, facet-output,
or expansion bounds, broaden examination scope, or turn unavailable material into available
content. Exact option and record schemas remain provisional.

```sh
okf search knowledge/product --limit 10 --json
okf search knowledge/product --scan-limit 10000 --limit 10 --json
okf search knowledge/product --full-scan --limit 10 --json
```

A complete primary scan permits exact totals and globally ordered limited results. If
eligible documents remain when the scan budget is exhausted, emit an explicit incomplete-query
result: totals are unknown, observed counts are labeled as observed rather than final, and
any returned hits are partial rather than an authoritative globally sorted page. Do not emit
authoritative pagination continuation for an incomplete scan. Exactly reaching the budget
does not imply incompleteness if there are no remaining eligible documents.

JSON output must include an explicit structured warning whenever known results or requested
payloads are omitted by a result limit or an output bound, and whenever scan exhaustion may
leave matches unexamined. Warnings identify the reason and applicable limit; omitted counts
are exact only when established, otherwise unknown. Scan exhaustion produces a warning even
when no matches have yet been found. Human output also makes omissions and incomplete scans
visible. These warnings cannot be suppressed merely to preserve ordinary concept-only output.

Scan completeness, facet aggregation completeness, and output truncation are separate states.
Complete computation with truncated presentation retains exact computed counts; incomplete
aggregation does not advertise all-match facet counts. Expansion exhaustion does not invalidate
an otherwise complete primary scan. The scan budget is a guard, not an early-stop optimization
after finding enough matches. Early-stop query mode is deferred; the initial capability does
not stop scanning merely because enough matches have been found for the requested page.

### Facets, nested filtering, and pagination

Facet summaries are **JSON only**, computed across **all matching primary documents before
pagination**. Counts are documents per value, deduplicating repeated list values within a document.
Missing branches, nulls, and empty lists do not count; no missing bucket is manufactured.
Scalar types remain distinct. Related concepts returned by expansion do not contribute.

Empty strings remain authored facet values unless local policy excludes them. Scalar leaf
selectors are supported; object-valued results produce diagnostics instead of fabricated
buckets. Repeated values from nested list traversal count once per document. The provisional
100-value threshold excludes only values above 100, not exactly 100; explicit inclusion of a
field remains filterable even when its facet summary is excluded.

Candidate fields are configured or explicitly requested; built-in candidates are `type` and
`tags[]`. High-cardinality fields are excluded by default, with an exclusion reason. The
provisional threshold is more than 100 distinct nonmissing values in the final matching set.
Per-field overrides may include them subject to visible output bounds. Merely requesting
a selector does not bypass the guard.

Every separately supplied condition combines with **AND**. Explicit list operators mean:

| Condition | Meaning |
| --- | --- |
| `tags[]="security"` AND `tags[]="production"` | Both values occur |
| `tags[] in ["security", "production"]` | At least one candidate value occurs |
| `tags[] not in ["draft", "deprecated"]` | None of the candidate values occurs |

An existing empty list fails `in` and passes `not in`; missing/null fields satisfy neither.
Operands are nonempty JSON arrays of scalars. Equality uses a JSON scalar literal. Distinct
values on one scalar field cannot both satisfy separate equality conditions.

`not in` evaluates the whole selected list; an allowed item cannot hide a forbidden item
elsewhere. Duplicate operands and repeated identical conditions are redundant. Object or
nested-array operands and incompatible field types produce diagnostics rather than coercion.
Facet filters work in human and JSON modes even though summaries are JSON-only.

Dot paths start at actual metadata keys, not named field types. `Norm.deadline.from` works
when the concept has a `Norm` property. For the authored `norms` list above,
`norms[].deadline.from` explicitly traverses items; `norms.deadline.from` does not.
Nested equality is existential over selected leaves; separate conditions can match different
records. Same-record correlated query predicates are deferred. Bracket quoting preserves
access to literal dotted keys; existing `--field` literal-key semantics remain compatible.

Paths are case-sensitive. A missing branch in one list item contributes no leaf; a selector
with no non-null leaves does not satisfy a filter unless it establishes an explicitly empty
list under the membership rules above. Relationship extraction preserves same-record
pairing independently of document-level query matching.

Illustrative commands use proposed selector-aware filters and pagination options:

```sh
okf search knowledge/product --type Procedure --facet-filter 'tags[] in ["security","production"]' --facet-filter 'tags[] not in ["draft"]' --facet-filter 'norms[].deadline.from="account-created"' --facets --offset 0 --limit 20 --json
okf search knowledge/product --type Procedure --facet-filter 'tags[] in ["security","production"]' --facet-filter 'tags[] not in ["draft"]' --facet-filter 'norms[].deadline.from="account-created"' --facets --offset 20 --limit 20 --json
```

If 120 documents match, illustrative first-page metadata is:

```json
{"kind":"query-summary","schema_version":1,"total_matches":120,"returned":20,"offset":0,"limit":20,"has_more":true,"next_offset":20}
{"kind":"facet","schema_version":1,"field":"tags[]","basis":"all-matches","complete":true,"values":[{"value":"security","count":80},{"value":"production","count":70}]}
{"kind":"facet","schema_version":1,"field":"norms[].deadline.from","basis":"all-matches","complete":true,"values":[{"value":"account-created","count":120}]}
```

Facet metadata accompanies the page's primary records. The full summary contract also
includes applied filters and examined bundle/version scope; these lines abbreviate it.
Counts may overlap for list fields and are not counts of only the displayed page. Facets
describe the final filtered set, including selected facet filters. Unfiltered search with
facets provides bundle-wide orientation using the same semantics.

Facet fields follow configured order; buckets sort by descending document count with typed
value order as the tie-breaker. Exclusion records identify field, reason, and threshold.
Partial facet output is explicitly marked incomplete/truncated, with unknown counts where
not fully measured. Facet counts remain independent of page size and offset. Unfiltered
bundle facets through `stats` are a possible additional command surface with the same
semantics; exact syntax remains to be specified.

Pagination works in human and JSON modes; facet summaries do not appear in human mode.
Requesting `--facets` without `--json` is a usage error. JSON pagination emits a summary even
without facets. Ordinary JSON output is preserved when none of the new options is requested;
configured facet candidates alone do not add records to every command. Required omission and
incomplete-scan warnings are the explicit exception to concept-only output compatibility.

Initial pagination will support offset/limit, with final option syntax provisional. Explicit
offset uses a positive limit and a nonnegative offset. Beyond-end pages are empty with no
continuation. All filters apply before sorting, aggregation, and slicing; expansion covers
only the selected page. Stable sorting uses qualified identity as a final tie-breaker.
Subsequent pages describe the same query, scope, and sort. `next_offset` is offset plus
returned count when more matches exist;
zero-match and beyond-end results have `has_more: false` and `next_offset: null`. Relevance
sort uses qualified identity as the final tie-breaker; ID sort uses it directly. Each response
reports the actual examined scope. Existing limit-only behavior remains compatible.
Stable pages require immutable available content and unchanged interpretation settings,
query, scope, and sorting; changing working trees or settings
can change totals and cause repeated or skipped entries between requests.

Offset requests are independent invocations. There are no pagination sessions, saved result
sets, or tool-managed metadata/query caches shared between requests. Each invocation examines
its selected scope again within its effective scan budget; page size does not reduce that
budget or promise fewer document reads. Repeating a query is supported, but reusing work or
preserving a snapshot between page requests is not part of this capability.

## 3. Out of scope and guarantees not provided

### Deferred functionality

- Full JSON Schema import, arbitrary selector expressions, and a separate glossary or
  term-management subsystem. Structured ontology inputs accept JSON syntax as YAML; this
  does not provide JSON Schema import. Glob-selected external document schemas,
  recursive-descent selectors, selector expressions, and executable selectors are deferred.
- Broader warning-suppression UX and automatic
  repair of manually maintained indexes or artifact-inclusive index generation.
- Automatic inverse-direction or transitive expansion; explicit inverse backlinks are
  included. Full target-body expansion is not a default retrieval mode. Multiple targets
  within one nested relationship record are also deferred.
- Cursor/session pagination, same-record correlated nested query predicates, alternative
  facet counts with a selected filter removed, and human facet summaries.
- Persistent metadata/query caches and saved results reused across independent page requests.
- Early-stop query mode that stops after enough ordered matches, with different total,
  facet, and sorting guarantees.
- General AI guidance. Structured completeness/page information is
  included because bounded retrieval requires it.
- Account-event ingestion, obligation instances, timestamp correlation, reminders, and
  live overdue/compliance evaluation.
- Remote acquisition, automatic cross-bundle move orchestration, and general artifact or
  computation execution. The multi-bundle capability's deferred features remain deferred.
- Moves that rewrite declared structured-reference occurrences; future preflight and rewrite
  requirements above are design boundaries, not initial supported functionality.

### Deliberate limits

- No guarantee that a modeled relationship is true, a policy is obeyed, or an asserted
  obligation is accepted or fulfilled. Metadata checks validate declarations.
- No inferred concept references from path-looking strings, vocabulary equivalence from
  matching names, or reverse assertions authored on behalf of target owners.
- No global backlink inventory or reads beyond selected registered scope. Out-of-scope
  consumers and unavailable versions remain unknown.
- No silent replacement of requested snapshot content by current candidates and no
  guarantee that locally available content is the latest remote content.
- No atomic snapshot of mutable files or stable pages across working-tree changes. Offset
  pagination does not create a persistent query session.
- No guarantee that bounded lint/expansion/facet output is complete after a limit is reached;
  such cases will be labeled. Counting all matches costs more than stopping at a page.
- No guarantee that custom constraints are enforced by other OKF readers. Optional local
  ontology semantics do not become universal conformance requirements.
- No standalone portability guarantee for cross-bundle relative links after separating
  directories; the multi-bundle design's layout-preserving portability limitation applies.
- No automatic content/index rewriting during queries or lint, trust propagation, source
  verification, or verification events produced by relationship resolution.

These limits constrain the CLI capability. They do not prohibit ordinary documents from
containing additional metadata, opaque paths, optional indexes, or broken references.

## Implemented CLI interface

`list` and `search` share structured query controls. Existing literal-key `--field key=value`
filters remain compatible. Repeatable `--facet-filter` uses the shared selector grammar with
`selector=JSON-scalar`, `selector in [JSON-scalars]`, or `selector not in [JSON-scalars]`.
Conditions combine with AND and do not coerce strings, booleans or numbers. Operands must be
nonnull scalars; membership arrays must be nonempty. Missing/null selections do not satisfy
negative membership; present empty lists can. Nonscalar selections and incompatible operand
types produce usage diagnostics.

```sh
okf search knowledge/product --facet-filter 'norms[].deadline.from="account-created"' --facets --limit 20 --offset 0 --json
okf list knowledge/product --columns '$id,stable-id,description,tags'
okf show procedures/onboarding knowledge/product --project '$id' --project 'norms[].deadline' --json
```

`--project` is repeatable and emits separate versioned projection records; metadata-only `show`
also supports it. `--columns` (alias `--column`) accepts repeatable/comma-separated human columns.
`--view` selects a named per-bundle view. Human view columns do not implicitly project JSON;
explicit `extended_output` views opt into configured machine projections and expansion. `--facets` and
repeatable `--facet` require `--json`; explicit facet selectors do not bypass the configured
high-cardinality guard.

`--offset` is nonnegative and requires positive `--limit`. The default document scan budget is
1,000; `query.scan_limit` changes it and explicit positive `--scan-limit` overrides configuration.
`--full-scan` disables that budget and conflicts with `--scan-limit`. Summaries report observed
matches, exact totals only when complete, returned count, continuation, scope and interpretation.
Facet counts concern matching documents before page slicing, with per-document deduplication.
The facet output cap is 1,000 values and omitted values remain reported. Working-tree pagination
does not establish a persistent immutable query session.

Repeatable `--expand <relationship-rule>` requests one-hop outbound semantic occurrences;
repeatable `--target-field <selector>` chooses related metadata. `--no-expand` disables configured
human expansion. Default bounds are 10 edges per hit, 100 distinct target payloads and 262,144
serialized bytes per query. `--expansion-edges`, `--expansion-targets` and `--expansion-bytes` can
change them within 1–100, 1–1,000 and 1–1,048,576 respectively. Edge conditions retain concrete
occurrence identity; target payloads deduplicate by qualified bundle/concept/version. Bounds,
truncation and reasons remain visible in expansion-summary records.

`backlinks --details` presents individual semantic incoming occurrences and inverse names.
Ontology YAML supports nested reusable definitions, reference selectors and relationship rules;
`ontology show` exposes effective recursive definitions. Structured authoring is implemented: `ontology add/update --field-yaml`, `--ref-yaml`, and
`--relationship-yaml` replace named declarations; `--from` accepts a type definition. Reusable
`ontology field-type add/update/remove` commands manage full definitions, and `ontology apply
--from` merges dependent changes with explicit top-level and per-concept removals. Omitted
named declarations remain, and file/flag overlaps fail rather than choosing precedence.

Concept `add/edit --set-yaml` replaces full values, and `--set-path` updates object properties
with missing intermediate map creation. Existing scalar `--set` remains literal. `edit --unset-path`
removes a property; `--patch` applies all RFC 6902 operations atomically to frontmatter, using
concrete JSON Pointer paths for list edits and guarded changes. YAML/JSON inputs are single
documents with duplicate/non-string keys, tags, anchors, aliases, merge keys and nonfinite numbers rejected. Only one input may consume
stdin. `--dry-run` previews mutations without writing. Formatting may normalize; lifecycle
consequences match existing concept edits. See the
[structured authoring workflow](../plugins/okf/skills/ontology/references/structured-authoring.md)
for complete examples and removal syntax.
Lint reports detailed field paths and codes, unsupported constraints, index coverage and incomplete
checks. Per-bundle lint settings include `index_exclude`, positive `finding_budget`, and severity
settings for every lint rule: `broken_link`, `missing_title`, `missing_description`,
`orphan`, `ontology_violation`, `spec_v02`, `source_unrecorded`, `source_missing`, and
`index_coverage`. Each accepts `off`, `info`, `warn`, or `error`; omitted settings retain
the default severity. `off` suppresses only the selected lint rule; conformance validation
and stale checks remain independent. Recursive value depth is bounded at 64; the default finding budget is 1,000 per concept.
Dynamic kind definitions may narrow allowed reference target types; concept-type `extends`
accepts a string or list for ancestry used by those restrictions. Qualified target types use
`<bundle-id>:<type>` and retain exact type spelling.

Per-bundle view settings use `columns`, `expand`, and `extended_output`; facet settings use
`fields`, `max_distinct_values`, and `include_high_cardinality`. `okf catalog --json` includes
effective ontology path/digest and interpretation settings. Explicit ontology files must be
readable; root ontology discovery remains optional. These are tool configuration and advisory
checks, not additional OKF document conformance requirements.

See [generated CLI reference](okf-cli-reference.md) for argument tables and current versioned
projection, facet, relationship, related-concept, summary and warning record contracts. Multi-bundle
legacy concept output uses separate `concept-identity` records, preserving authored bundle/version
keys rather than overwriting them.
