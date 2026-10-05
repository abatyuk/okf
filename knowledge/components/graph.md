---
type: Component
title: graph module
description: Scoped forward and reverse adjacency from portable and declared structured references.
layer: core
depends_on:
- /components/bundle
- /components/model
- /components/ontology
sources:
- resource: crates/okf-core/src/graph/build.rs
  kind: git-path
  fingerprint:
    blob_sha: 8f079f547a272f303016472e82f4b13d5202fbf1
- resource: crates/okf-core/src/graph/catalog.rs
  kind: git-path
  fingerprint:
    blob_sha: 361ebf706bb821babc7cf87064cd72697b64612a
- resource: crates/okf-core/src/graph/relationships.rs
  kind: git-path
  fingerprint:
    blob_sha: cbd606b8b81010423384005588fa0a4c48af3702
- resource: crates/okf-core/src/graph/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 22b5e7251b42e33ea5f0519cfe6673506bfe1496
- resource: crates/okf-core/src/graph/render.rs
  kind: git-path
  fingerprint:
    blob_sha: d983856c864548bae7ca134b4300434c7d584618
last_modified: 2026-10-04T20:39:29Z
---
# graph module

Builds forward and reverse adjacency from [bundle](bundle.md) concepts, body links, standard
`sources[].resource` lineage, and explicitly declared [ontology](ontology.md) reference selectors.
Path-looking opaque metadata does not become an edge merely because it resembles a link.
Nested selectors retain occurrence paths; semantic kind and attribute selectors pair with the
same list record. Inverse backlinks display incoming assertions rather than target acceptance.

Cross-bundle results retain qualified identity, original reference, resolution evidence and
examined scope. Broken and out-of-scope targets remain visible; a catalog is not a global backlink
inventory. Requested source snapshots and current candidates remain distinct. Resolution does
not create verification events or accept changed source fingerprints.

Powers [links](../commands/links.md), [backlinks](../commands/backlinks.md),
[affected](../commands/affected.md), and bounded [graph](../commands/graph.md) rendering.
Automatic cross-bundle moves and rewriting declared nested-reference occurrences remain deferred.

Location: `crates/okf-core/src/graph`.
