---
type: DomainType
title: Catalog
description: A local registry of namespaced bundle IDs, effective directory roots and availability.
module: bundle
defined_in:
- /components/bundle
sources:
- resource: crates/okf-core/src/bundle/catalog.rs
  kind: git-path
  fingerprint:
    blob_sha: 1c08a7309b9fb6ed49c9f14713095aee01b641b3
last_modified: 2026-10-04T20:39:29Z
---
# Catalog

Holds the catalog path and registrations keyed by stable bundle ID. Entries expose configured
and effective roots, whether an override applied, and availability. Available roots are
canonicalized and equal or nested roots are rejected. Overrides may relocate registered IDs
but never introduce new identities or satisfy changed snapshot requests.

A catalog adds tool context without requiring an identity manifest, ontology or central authority.
Primary selection and examination scope remain independent.
