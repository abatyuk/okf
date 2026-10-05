---
type: Component
title: bundle module
description: Bundle loading, catalog identity, explicit selection and per-bundle interpretation settings.
layer: core
depends_on:
- /components/parse
- /components/model
sources:
- resource: crates/okf-core/src/bundle/settings.rs
  kind: git-path
  fingerprint:
    blob_sha: b05d13b997af6da4c50cacc8c6018a1d92d9e32c
- resource: crates/okf-core/src/bundle/catalog.rs
  kind: git-path
  fingerprint:
    blob_sha: 1c08a7309b9fb6ed49c9f14713095aee01b641b3
- resource: crates/okf-core/src/bundle/context.rs
  kind: git-path
  fingerprint:
    blob_sha: c5a39cc31c202ec2ffc6da62728fb2423d679caf
- resource: crates/okf-core/src/bundle/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: 8732f45ba6d8182eed14497fa3e8d2d7bd0f4375
- resource: crates/okf-core/src/bundle/resolve.rs
  kind: git-path
  fingerprint:
    blob_sha: 1688e8ef427fe5e9ae85d7041fb4f860d432e32a
- resource: crates/okf-core/src/bundle/config.rs
  kind: git-path
  fingerprint:
    blob_sha: 637a0f33818ff6548c4bd96590bceaecd5fa1b7b
- resource: crates/okf-core/src/bundle/loader.rs
  kind: git-path
  fingerprint:
    blob_sha: facc2975de71300501ddeaa361a0d3c65924e2ad
last_modified: 2026-10-04T20:39:29Z
---
# bundle module

Loads concepts while honoring reserved files, and keeps bundle selection separate from graph
examination scope. Catalog identity qualifies ordinary concept IDs without changing their paths.
Paths and namespaced IDs are distinct selectors; `OKF_BUNDLE` retains path semantics. Typed
`default_bundle` replaces the deprecated path-only `bundle` setting.

The optional catalog registers local directories, canonicalizes available roots, rejects equal or
nested registrations, and reports unavailable material. Config paths are relative to `okf.toml`;
locations are relative to the catalog. Per-bundle settings supply ontology, lint, query, view, and
facet interpretation. Registration does not imply remote acquisition or catalog-wide traversal.

See the [multi-bundle decision](../decisions/multi-bundle-scope.md). Location:
`crates/okf-core/src/bundle`.

File parse errors retain the document path. Full bundle loads report every malformed YAML concept in deterministic file order, while direct reads report only the requested file. No partial bundle is returned after parse failures.
