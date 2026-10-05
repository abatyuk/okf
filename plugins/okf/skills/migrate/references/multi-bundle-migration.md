# Migrate structured documents across bundles

Use this example for migrations with multiple destinations, structured transformations, or resumed
batches. Choose a scoped recovery location outside the concept inventory; the ledger is a working
artifact, not a new required OKF file. Preserve source documents throughout migration.

## Map before writing

Use `(source identity, source version)` and `(destination bundle ID, concept ID)` as distinct keys.
For uncataloged destinations, record their canonical root instead of inventing catalog identities.
Store a content digest or inspected source version so a changed source cannot silently reuse an
earlier completion decision. Splits have multiple destination rows; merges retain every source
identity and require an explicit conflict/attribution decision.

| Source | Destination bundle | Destination concept | Operation | State |
| --- | --- | --- | --- | --- |
| legacy/product.md | acme.product | notes/shared | create | planned |
| legacy/finance.md | acme.finance | notes/shared | create | planned |

The two `/notes/shared` IDs are distinct. Also record effective root, source digest/version,
transformation decisions, recovery copy, last successful operation, resulting content digest, and
acceptance evidence. Suggested states are planned, content-written, references-checked, accepted,
skipped-with-reason, and unresolved. These are local ledger states, not concept lifecycle values.

## Define transformations

| Source field | Destination | Rule |
| --- | --- | --- |
| requirements | relations | Rename the list; retain order, duplicates, and each complete record |
| requirements[].target | relations[].target | Declared reference; map its source document identity to the final destination |
| requirements[].deadline | relations[].deadline | Preserve the integer and its pairing with that target |
| x-import | x-import | Opaque extension; preserve nested types, nulls, empty lists, and path-looking strings |

Requiredness and reference declarations come from source meaning or supplied requirements, not
observed presence alone. Preserve unmapped fields unless an explicit conversion decision names
them. Separate missing, null, empty, false, zero, and string-valued numbers. When a field is renamed,
update the destination ontology selectors consistently. Read each target bundle's own ontology;
cross-bundle target restrictions use qualified type names.

## Fixture files

The complete files below form the starting workspace. Their filename headings are consumed by the
executable workflow test. For real migrations, merge configuration into existing files and preserve
uncommitted content; do not replace an existing workspace with this fixture.

## `okf.toml`

```toml
catalog = "okf-catalog.yaml"
default_bundle = { id = "acme.product" }
```

## `okf-catalog.yaml`

```yaml
catalog_version: 1
bundles:
  acme.product:
    location: {type: directory, path: knowledge/product}
  acme.finance:
    location: {type: directory, path: knowledge/finance}
```

## `knowledge/product/ontology.yaml`

```yaml
okf_ontology: "0.1"
field_types:
  Relation:
    base: object
    fields:
      target: {type: string, required: true}
      deadline: {type: int, required: true}
concepts:
  Procedure:
    fields:
      relations:
        type: list
        item: Relation
    references:
      policies:
        selector: relations[].target
        target: acme.finance:Policy
        cardinality: 1..n
    relationships:
      implementations:
        reference: policies
        kind: implements
        inverse: implemented-by
        attributes:
          deadline: relations[].deadline
```

## `knowledge/finance/ontology.yaml`

```yaml
okf_ontology: "0.1"
concepts:
  Policy: {}
```

## `legacy/product.md`

```markdown
---
type: Procedure
title: Product review
requirements:
  - target: finance.md
    deadline: 24
  - target: finance.md
    deadline: 72
x-import:
  enabled: false
  code: "001"
  nullable: null
  empty: []
  attachment: finance.md
---
Apply the [finance policy](finance.md#review).
```

## `legacy/finance.md`

```markdown
---
type: Policy
title: Finance policy
---
# Review

Review the submitted report.
```

## Write and resume

Create `notes/shared` in each explicitly selected bundle using `okf add` with body files. Add
original-document provenance once using `okf edit --add-source-json`. From either destination
document, its original source resource is `../../../legacy/product.md` or
`../../../legacy/finance.md`. Preserve that origin rather than replacing it with a migrated copy.
Use `--set-yaml` for complete custom nested values or concrete `--patch` operations for selective
changes. Preview with `--dry-run`; scalar `--set` does not traverse nested keys.

In the product destination, rename `requirements` to `relations`. Its declared targets and body
link become `../../finance/notes/shared.md`, retaining `#review` on the body link. The opaque
`x-import.attachment` remains `finance.md`. Preserve both relationship occurrences with their own
24/72 deadlines; do not collapse them because the target is shared.

If the product write succeeds but finance creation fails, record product as content-written and
finance as planned/unresolved. Product's pending edge is not yet an acceptance failure requiring a
new destination ID. On restart, inspect product's current bytes against the recorded result and
the source version; skip it only if unchanged. Inspect finance's actual state before retrying.
If a crash occurred after a write but before logging it, reconcile the file before replaying it.
Do not append the same source again or overwrite intervening user edits. Complete destination
creation before final edge checks; this also permits cyclic references.

## Reconstruct sources and snapshot expectations

For each source entry, distinguish original provenance from an intentional reference to a migrated
concept. Only the latter follows the destination map. Rebase its ordinary resource from the new
declaring document, update optional `bundle_ref.id` and bundle-root-relative `bundle_ref.path`, and
preserve `sources[].id` joins. A copied opaque artifact has its own reviewed destination mapping.

Preserve authored snapshot `commit`, `ref`, `label`, and `digest` unless the task explicitly changes
the expectation. Reformatting a migrated target can legitimately invalidate an old byte digest.
Report that mismatch or unavailable historical content; a current candidate does not satisfy it.
Do not remove the expectation, replace the original evidence, or refresh fingerprints to force a
clean report. Either retain the original pinned source or leave a documented unresolved decision.
Resolution, snapshot satisfaction, source drift, and document verification remain separate checks.

## Accept the scoped migration

```sh
okf validate --bundle-id acme.product --json
okf validate --bundle-id acme.finance --json
okf lint --bundle-id acme.product --scope-bundle acme.finance --fail-on never --json
okf list --bundle-id acme.product --scope-bundle acme.finance --full-scan --json
okf links notes/shared --bundle-id acme.product --scope-bundle acme.finance --json
okf backlinks acme.finance:/notes/shared --bundle-id acme.product --scope-bundle acme.finance --details --json
okf search --bundle-id acme.product --scope-bundle acme.finance --type Procedure --project relations --expand implementations --full-scan --json
```

Expect two qualified destination concepts, two paired semantic occurrences to the finance policy,
one preserved body fragment, and separate original-source entries. Compare the extension values
and untouched source bytes, and check that a completed rerun adds neither concepts nor sources.
Inspect lint findings, scope, scan completeness, expansion bounds, and all unresolved ledger rows;
a zero exit code or matching raw counts alone is insufficient. Reconcile every source with its
outputs or an explicit split/merge/skip reason. Compare representative records for each conversion
rule and all exceptional cases; report sampling honestly rather than claiming exhaustive fidelity.

Only mark rows accepted after their checks pass. A recoverable migration may finish with explicit
unresolved rows, but it is not fully accepted. Report their blockers, affected destinations, and
the safe restart point. Review the combined repository diff, or per-repository diffs when separate
checkouts are involved. Consumers outside the selected scope remain unknown.
