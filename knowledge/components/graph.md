---
type: Component
layer: core
depends_on:
- /components/bundle
- /components/model
- /components/ontology
title: graph module
description: Scoped forward and reverse adjacency from portable and declared structured references.
sources:
- resource: crates/okf-core/src/graph/build.rs
  kind: git-path
  fingerprint:
    blob_sha: 8f079f547a272f303016472e82f4b13d5202fbf1
- resource: crates/okf-core/src/graph/catalog.rs
  kind: git-path
  fingerprint:
    blob_sha: 0f2cf11638b06741347dc5f93a9f78ab69aec07c
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
    blob_sha: a50943c71aaa5d7c6bb1c94854ca573261d29ceb
last_modified: 2026-10-19T03:58:59Z
---
# graph module

Builds forward and reverse adjacency from [bundle](bundle.md) concepts, body links, standard
`sources[].resource` lineage, and explicitly declared [ontology](ontology.md) reference selectors.
Path-looking opaque metadata does not become an edge merely because it resembles a link.
Nested selectors retain occurrence paths; semantic kind and attribute selectors pair with the
same list record. `links --details` exposes authored outgoing semantic occurrences without
collapsing repeated targets; `backlinks --details` presents their incoming view with inverse
labels. Both preserve record attributes. Inverse backlinks display incoming assertions rather
than target acceptance, and inverse labels do not synthesize additional outgoing edges.

Cross-bundle results retain qualified identity, original reference, resolution evidence and
examined scope. Broken and out-of-scope targets remain visible; a catalog is not a global backlink
inventory. Requested source snapshots and current candidates remain distinct. Resolution does
not create verification events or accept changed source fingerprints.

Powers [links](../commands/links.md), [backlinks](../commands/backlinks.md),
[affected](../commands/affected.md), and bounded [graph](../commands/graph.md) rendering.
Automatic cross-bundle moves and rewriting declared nested-reference occurrences remain deferred.

Location: `crates/okf-core/src/graph`.
