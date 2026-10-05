# Multi-bundle capability

Status: implemented local capability. The design below records the capability contract and
its deliberate limits; the implemented CLI interface is recorded at the end. Configuration
and output are discoverable through `okf schema` and the generated CLI reference.

## Motivation

Knowledge often spans several domains with different owners and update cycles. A product
bundle may depend on finance policies, engineering standards, and shared definitions.
Keeping these as separate bundles allows each collection to be maintained independently,
while references preserve the relationships between them without copying their content.

The current single-bundle configuration does not express which bundles belong to a working
context or how to identify targets across their boundaries. Directory names alone are poor
identifiers: different repositories can use the same names, and local checkout locations
can differ between users. A catalog makes bundle identity, location, and selection explicit
and gives the CLI enough context to check references and trace dependencies across bundles.

This capability keeps OKF's lightweight authoring model. It adds coordination for tools
without requiring a central service, Git, an ontology, or a new document format. Ordinary
paths and Markdown remain the portable reference surface; the catalog adds local context
for resolving and inspecting them.

## 1. Capability and restrictions

The CLI will work with several independent OKF bundles through one local catalog. Bundles
may be sibling directories in the same repository, directories in separate existing
checkouts, or directories outside Git. Users or other tools make those files available;
the CLI does not acquire them from remote locations.

The capability adds:

- Stable, namespaced bundle IDs independent of directory names and locations.
- Explicit selection of a bundle by ID or path, including selection from the current
  directory when it belongs to a registered bundle.
- References between bundles through ordinary Markdown links and provenance sources.
- Cross-bundle graph queries, backlinks, affected analysis, and reference diagnostics
  within an explicitly selected scope.
- Optional snapshot expectations resolved against available local material, with clearly
  labeled candidates when the requested content cannot be established.

A bundle remains an ordinary OKF directory. It needs no catalog to be read independently,
no ontology to establish its boundary, and no new identity manifest. The catalog supplies
identity and location for this CLI's multi-bundle operations.

### Restrictions introduced by the CLI

These rules apply to the catalog feature, not to OKF conformance:

| Rule | Purpose and effect |
| --- | --- |
| Use namespaced stable IDs, such as `acme.finance` | Distinguishes bundles across repositories; no alias layer is introduced |
| Register one effective local root per bundle ID | Makes selection and reference resolution explicit; machine-local overrides can change that location |
| Reject conflicting registrations | Avoids silently choosing one bundle when identity or location is ambiguous |
| Do not register equal or nested roots | A bundle cannot be a subbundle of another; ordinary subdirectories inside a bundle remain valid |
| Keep reference extensions consistent with ordinary resources | Catalog metadata cannot silently redirect a reference to known different material |
| Make graph/check scope explicit | Results describe selected, available bundles rather than claiming global coverage |

Root overlap is checked after path canonicalization for available directories. Unavailable
roots are reported as unavailable, not treated as verified non-overlapping locations.
Independently maintained forks receive new IDs; mirrors retain the original ID. These are
identity conventions, not proof of publisher authenticity or a global registration system.

### Relationship to OKF

This is a CLI extension above [OKF v0.2](https://github.com/GoogleCloudPlatform/open-knowledge-format/blob/main/SPEC.md),
not a stricter replacement specification. OKF permits additional metadata, ordinary paths
and URLs, and broken links; indexes are optional. Concept IDs remain paths without `.md`.
See §§2–6 and §8.

| Addition | Effect on OKF compatibility |
| --- | --- |
| Catalog and TOML settings | Optional tool configuration; neither becomes a required bundle file |
| Bundle ID | Qualifies CLI targets without changing standard concept IDs |
| `bundle_ref` source metadata | Producer-defined extension; ordinary `resource` retains its meaning |
| Catalog registration and boundary checks | Tool configuration errors, not new concept-conformance rules |
| Missing cross-bundle targets | Warnings by default, separately configurable for CI; broken links remain consumable |
| Optional snapshot expectations | Extra resolution metadata; no new mandatory provenance or verification fields |

The design does not intentionally override an OKF normative requirement or impose additional
conformance requirements. It preserves optional metadata, unknown fields, and ordinary
single-bundle use. Rejecting a catalog configuration must not be reported as invalid OKF
content; the underlying bundle can still be addressed directly.

There is a portability limitation: OKF describes a bundle as a self-contained distribution
unit. Relative references to sibling bundles depend on the surrounding layout. They retain
ordinary path semantics, but distributing just one directory may leave broken references.
Consequently, this mode provides **layout-preserving portability**, not independently
self-contained distribution or a guarantee that every reader permits access outside a
selected bundle root. This limit is explicit rather than addressed by changing path meaning.
No stronger claim of universal reader interoperability is made.

## 2. Files, references, and behavior changes

### Catalog and existing configuration

Keep `okf.toml` and add a separate `okf-catalog.yaml`:

```toml
# okf.toml
catalog = "okf-catalog.yaml"
default_bundle = { id = "acme.product" }
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

  partner.standards:
    location:
      type: directory
      path: ../standards-checkout/knowledge
```

The catalog path is resolved relative to its configuration file. Bundle paths are relative
to the catalog file, not the command's working directory. The catalog may live outside Git.
Initial location support is local directories, including existing checkouts; it does not
introduce a remote Git or web acquisition schema.

The default selector is explicitly typed: `default_bundle = { id = "acme.product" }`
selects a catalog ID, while `default_bundle = { path = "knowledge" }` selects a directory.
Exactly one of `id` and `path` is permitted. Default paths resolve relative to `okf.toml`.
The current `bundle` setting becomes a deprecated path-only compatibility alias;
`bundle = "knowledge"` retains that meaning even when a catalog is added. If both settings
are present, accept only equivalent path selectors and otherwise report a configuration
error. Equal strings in an ID selector and a path selector do not establish equivalence.
New configurations and documentation use the typed `default_bundle`; exact syntax is
provisional, but selectors must not infer identity from directory spelling.

Machine-local overrides can map a stable ID to a different local checkout, with explicit
precedence and effective-location reporting. They cannot silently replace a requested
snapshot. The override storage format is an implementation detail still to be specified.

No `okf-bundle.yaml`, workspace lockfile, or remote-cache/TTL configuration is introduced.
`ontology.yaml` remains optional and is not used as a bundle marker.

### Bundle selection and discovery

For catalog-aware operations, selection follows this order:

1. Explicit bundle path or bundle ID; conflicting selectors are rejected.
2. The existing environment override, with `OKF_BUNDLE` retaining its path meaning.
3. The registered bundle containing the current directory.
4. The configured default bundle.
5. The sole catalog entry, if unambiguous.
6. Otherwise require selection and show the available choices.

Without a catalog, existing single-bundle selection remains available. IDs and paths use
distinct selectors so a directory name cannot accidentally become a bundle ID. Discovery
can suggest candidate directories, but does not register them or infer authoritative roots
from an ontology, index, or repository boundary.

### Cross-bundle references

For two bundles in the same repository:

```text
repo/
  okf.toml
  okf-catalog.yaml
  knowledge/
    product/metrics/revenue.md
    finance/policies/margin-standard.md
```

The source entry in `product/metrics/revenue.md` can be:

```yaml
sources:
  - id: margin-standard
    resource: ../../finance/policies/margin-standard.md
    bundle_ref:
      id: acme.finance
      path: policies/margin-standard.md
```

The body can use an ordinary link:

```markdown
See the [margin standard](../../finance/policies/margin-standard.md).
```

`resource` resolves relative to the containing document. `bundle_ref.path` is explicitly
relative to the target bundle root. Both identify the same material. `sources[].id` remains
the source attribution key; it is not the bundle ID. The target concept's standard ID is
still `policies/margin-standard`.

No copies are made. Readers that ignore `bundle_ref` can follow the ordinary path when
the surrounding layout is available. Separate checkouts can use the same approach if their
relative layout is reproduced. Moving a bundle may therefore require updating ordinary
links as well as its catalog location: stable identity alone does not make paths relocatable.

The CLI resolves an ordinary local path, identifies the registered root containing its
target, and assigns the qualified target. It must not collapse an above-root traversal into
an unrelated concept inside the source bundle. This extends current single-bundle graph
resolution specifically for registered cross-bundle concept targets; it does not grant
general unrestricted artifact reads outside a bundle.

The extension is optional. Ordinary links can become cross-bundle edges through their
resolved target location without additional metadata. Existing internal links retain their
meaning. An unresolved path is never matched to another bundle by filename or title alone.

Standard URLs remain usable references. An explicit declared mapping can associate one
with local material, but this feature does not fetch or verify the remote resource. It must
report that equivalence is unchecked when no local evidence establishes it.

### Resolution evidence and diagnostics

The resolver prefers the strongest readily available evidence:

| Evidence | What can be reported |
| --- | --- |
| Same canonical local file | Both references address the same available file |
| Same repository, commit, and path | Both address the same locally available Git object |
| Matching optional digest | The checked file's bytes match the recorded digest |
| Explicit declared mapping | A correspondence is configured, with equivalence unchecked |

A declared mapping is allowed without independent proof, but it cannot override a known
mismatch. Resource semantics take priority over contradictory catalog metadata. Resolution
evidence does not establish the factual correctness of a concept or create a verification event.

Resource resolution and snapshot satisfaction are separate results. A live ordinary path
remains the ordinary target. Finding different historical bytes does not silently replace
that target: historical content counts as a successful reference match only when the
resource identifies that content or an explicit mapping establishes correspondence without
overriding a known mismatch. Otherwise retain the ordinary target and historical result
separately, with their evidence and status.

Diagnostics distinguish unknown bundle IDs, unavailable local roots or revisions, missing
targets, registration conflicts, reference mismatches, unchecked equivalence, and changed
source fingerprints. These are separate dimensions: a target can exist while its source
has changed. Missing targets warn by default; CI may configure a stricter lint policy.

### Optional snapshots

References may describe a requested snapshot using a Git commit, ref, human label, optional
digest, or a combination. The whole snapshot object and each identifier are optional. An
illustrative shape is:

```yaml
bundle_ref:
  id: acme.finance
  path: policies/margin-standard.md
  snapshot:
    ref: fy2026
    label: FY2026 approved standard
```

An exact commit or digest may be added without requiring both. A digest, when supplied,
describes the referenced file's raw bytes; it is not an unspecified whole-bundle hash.
Exact identifiers constrain the evidence, while labels and mutable refs are lookup hints.
Field names remain provisional.

Initial authored snapshot support is limited to provenance source entries carrying
`bundle_ref`. Body links and custom nested concept-reference strings use ordinary path
resolution; they do not inherit snapshot expectations from a similar source entry. Snapshot
metadata for those occurrences is not included in the initial capability.

The policy is **prefer requested, allow labeled candidate**:

1. Attempt to establish the requested content from available local files or Git material.
2. Report whether the request is matched, unverified, unavailable, or mismatched.
3. If it cannot be established, allow current local content as a separately labeled candidate.
4. Keep the candidate distinct from the resolved request in human output, machine output,
   and graph results. A known mismatch never counts as a successful snapshot match.

A user or skill may inspect and reconcile a candidate. Merely returning it neither accepts
it as a substitute nor changes authored metadata, fingerprints, or verification history.
Local Git operations must not implicitly fetch missing objects, including in partial clones.

An extension-only snapshot expectation does not pin an ordinary reader to historical
content. A portable exact pin also needs a standard resource identifying that same content.
A live relative path plus a historical expectation remains a live path for other readers.

### Graphs and scope

Primary bundle selection and examination scope are separate. The default scope contains
only the selected bundle. Examining additional bundles requires an explicit bundle set or
an explicit catalog-wide scope; catalog registration alone does not authorize traversal.
Cross-bundle references outside that scope remain visible with an out-of-scope status.
Results report requested scope, actual examined bundle/version scope, and unavailable
members. Exact scope-option syntax remains provisional.

Graph nodes are qualified by bundle identity, concept identity, and resolved version where
available. Multiple revisions can coexist. Working-tree content is identified as such,
not mislabeled as an immutable commit when local changes are present.

Edges retain the original reference, its source location, target, and resolution evidence.
Unresolved edges remain visible. Traversal supports cycles through visited-node tracking
and bounded scope. Backlinks and affected analysis report which bundles and versions were
examined; consumers outside that scope remain unknown.

Bundles in the same repository may be updated and committed together. When inspecting a
specific local commit, they use that commit unless a reference explicitly requests another
version. Ordinary working-tree reads do not promise an atomic snapshot across files.

## 3. Out of scope and guarantees not provided

### Deferred functionality

- Remote Git clone/fetch/pull, remote branch checks, and automatic checkout updates.
- Web-bundle acquisition, HTTP archives, publication manifests, and new web authentication.
- Tool-managed remote caches, TTL settings, and force-refresh controls. Visibly labeled
  cached fallback is a future fetching policy, not an initial cache subsystem.
- Catalog imports, implicit catalog merging, and a bundle identity sidecar.
- Cross-bundle computation resources, executors, attesters, and general artifact resolution.
- Automatic cross-bundle rename/update orchestration. Skills may coordinate multiple CLI
  calls and review a combined repository diff.
- Self-contained export, dependency mirroring, and managed catalog/publication adapters.

### Deliberate limits

- No nested bundle roots, alias layer, central ID authority, or workspace lockfiles.
- No guarantee that local content is the latest remote content. Default reads use what is
  available locally; users or other tools maintain checkouts.
- No guarantee that all requested snapshots are available or provable. Digests are optional,
  and a labeled candidate is not evidence that the requested snapshot was retrieved.
- No guarantee of standalone distribution after separating bundles that use relative links.
  Catalog-aware resolution cannot repair portability for readers that do not have the layout.
- No global backlink inventory, transactional update across repositories, or atomic snapshot
  of mutable working trees.
- No automatic trust propagation, source verification, or execution of referenced code.
- No inferred correspondence from matching names and no silent acceptance of known mismatches.

These limits constrain the CLI capability. They do not prohibit ordinary OKF documents from
containing external URLs, additional fields, incomplete knowledge, or broken references.

## Implemented CLI interface

Select a catalog identity with global `--bundle-id <id>` or select an ordinary path with the
trailing bundle positional. These selectors conflict. `OKF_BUNDLE` remains path-only. Add
examined registrations with repeatable `--scope-bundle <id>` or deliberate `--catalog-scope`;
the default examines only the selected bundle. Qualified graph targets use
`acme.finance:/policies/margin-standard`.

`okf catalog` reports effective registrations without selecting a primary. With `--json`, it
emits `bundle-registration` and `effective-settings` records including configured/effective
locations, overrides, availability and interpretation context. Catalog inspection rejects
bundle selection, scope and revision flags.

Scope applies to graph, links, backlinks, resolve, affected, lint, search and list. Other commands
reject additional scope. `--revision <ref>` applies only to graph, links, backlinks, resolve,
affected, search and list. Historical reads identify examined commit material and effective ontology
interpretation; unavailable revisions/settings remain diagnostics. Local Git commands disable
implicit object fetching.

Machine-local override storage is a YAML file selected by `catalog_overrides` in `okf.toml`,
resolved relative to that configuration. `OKF_CATALOG_OVERRIDES` takes precedence and selects
a path from the working directory. Override paths resolve relative to the override file:

```yaml
bundles:
  acme.finance: ../finance-checkout/knowledge
```

`bundle_ref.snapshot` supports `commit`, `ref`, `label` and `digest`. Commits are full immutable
Git object IDs; refs and labels are hints. Digests describe raw referenced file bytes as SHA-256
hex, optionally prefixed `sha256:`. Scope records also retain `snapshot_examined` for separately inspected historical target nodes.
Snapshot results retain requested metadata, status, evidence,
resolved identity and separately labeled candidate identity. Nodes use qualified bundle/concept
identity and version evidence: current content is `working-tree`, commit material is `git:<SHA>`,
and a digest-established snapshot is `sha256:<HEX>`. Explicit standalone subdirectories retain
their own boundary; combining one with an overlapping registered root in scope is rejected.

See [generated CLI reference](okf-cli-reference.md) for actual argument tables and machine record
contracts. Catalog/configuration errors remain separate from OKF content conformance.
