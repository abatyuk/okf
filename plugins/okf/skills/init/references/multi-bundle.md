# Set up a local multi-bundle catalog

Use separate bundles when ownership, update/release cycles, or distribution boundaries differ.
Use ordinary subdirectories when those boundaries are shared; extra bundles add selection and
layout costs. Independently maintained forks get new IDs; mirrors retain their ID. Select stable,
namespaced IDs from the user's domain, not machine-specific directory names.

This example has product material referencing a finance policy. Run commands from the workspace
root. Initialize only new roots; preserve existing configuration and concepts when adapting it:

```sh
okf init knowledge/product --no-ontology
okf init knowledge/finance --no-ontology
```

Merge the following config into the workspace's existing files. Do not register equal or nested
roots. Catalog creation/settings currently use narrow YAML/TOML edits, not an inferred catalog
mutation command. Preserve existing entries and inspect the combined diff.

## `okf.toml`

```toml
catalog = "okf-catalog.yaml"
default_bundle = { id = "acme.product" }

[bundle_settings."acme.product".query]
scan_limit = 1000

[bundle_settings."acme.product".views.compact]
columns = ["$id", "title", "description"]

[bundle_settings."acme.product".facets]
fields = ["type", "tags[]"]
max_distinct_values = 100
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

The catalog path resolves from `okf.toml`; bundle locations resolve from the catalog file.
An optional `ontology = "tooling/product-ontology.yaml"` belongs in
`[bundle_settings."acme.product"]` and resolves from `okf.toml`, not the bundle root. Add it only
after creating a readable sidecar. Each bundle uses its own ontology. Uncataloged use has
`[bundle_settings.default]` and a path default such as `default_bundle = { path = "knowledge" }`.
Settings alone do not register bundles. Replace a legacy `bundle` alias when introducing a
non-equivalent typed default; leaving conflicting selectors creates a configuration error.

## `knowledge/finance/policies/margin-standard.md`

```markdown
---
type: Policy
title: Margin standard
description: Defines the illustrative margin calculation.
---
Margin is revenue less direct costs in this example.
```

## `knowledge/product/metrics/revenue.md`

```markdown
---
type: Metric
title: Revenue reporting
description: Uses the finance margin definition.
sources:
  - id: margin-standard
    resource: ../../finance/policies/margin-standard.md
    bundle_ref:
      id: acme.finance
      path: policies/margin-standard.md
---
Use the [margin standard](../../finance/policies/margin-standard.md) for margin reporting.[^margin-standard]

[^margin-standard]: Finance's margin definition.
```

These complete concept files illustrate the resulting content. In normal authoring, use `okf add`
with a body file and `--add-source-json` on `okf edit` for the full source mapping. `resource` is
document-relative; `bundle_ref.path` is relative to the target bundle root. Both must identify the
same material. `sources[].id` is a claim attribution key, distinct from the bundle identity.
Snapshots are optional; do not invent one. An exact portable pin needs a standard resource that
identifies the same historical material, not only a snapshot extension on a live path.

## Inspect and check

```sh
okf catalog --json
okf validate --bundle-id acme.product
okf validate --bundle-id acme.finance
okf links metrics/revenue --bundle-id acme.product --scope-bundle acme.finance --json
okf resolve ../../finance/policies/margin-standard.md --from metrics/revenue --bundle-id acme.product --scope-bundle acme.finance --json
okf backlinks acme.finance:/policies/margin-standard --bundle-id acme.product --scope-bundle acme.finance --details --json
okf lint --bundle-id acme.product --scope-bundle acme.finance --fail-on never --json
```

Check effective roots and availability, requested/examined scope, and the resolved source edge.
Running links without the added scope should retain an out-of-scope target, not establish a missing
policy. Never pass extra scope to single-bundle mutators or `validate`. Use explicit `--bundle-id`
throughout multi-step catalog work; `OKF_BUNDLE` stays path-only and overrides implicit selection.
An explicit bundle path conflicts with `--bundle-id`.

## Optional machine-local override

For an existing finance checkout at `../finance-checkout/knowledge`, a local override file can be:

```yaml
bundles:
  acme.finance: ../finance-checkout/knowledge
```

Select it with `catalog_overrides = "okf-catalog.local.yaml"` at the top level of `okf.toml`, or
`OKF_CATALOG_OVERRIDES` (whose filename resolves from cwd and takes precedence). Paths inside the
override resolve from that file. Keep machine-local paths out of shared configuration when needed;
use the repository's existing local-ignore convention. Inspect `okf catalog --json` again.

An override changes location, not snapshot acceptance or ordinary link meaning. If the effective
layout changes, update and review both ordinary resources/body links and applicable `bundle_ref`
metadata, then recheck edges. A catalog ID does not redirect a known conflicting ordinary path.
Relative links provide layout-preserving portability; distributing one bundle alone may break them.
No checkout acquisition, fetch, or global consumer inventory is provided by this setup.
