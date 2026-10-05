---
type: DesignDecision
title: Catalog identity and explicit scope
description: Catalog coordination preserves ordinary paths and separates selection, examination and snapshot evidence.
decision_status: accepted
affects:
- /components/bundle
- /components/graph
- /components/query
sources:
- resource: docs/multi-bundle-capability.md
  kind: git-path
  fingerprint:
    blob_sha: de57924f694087a338525d76ab32ad6f60476c5f
- resource: crates/okf-core/src/graph/catalog.rs
  kind: git-path
  fingerprint:
    blob_sha: 361ebf706bb821babc7cf87064cd72697b64612a
- resource: crates/okf-core/src/bundle/catalog.rs
  kind: git-path
  fingerprint:
    blob_sha: 1c08a7309b9fb6ed49c9f14713095aee01b641b3
- resource: https://github.com/abatyuk/okf/blob/main/docs/multi-bundle-capability.md
- resource: crates/okf-core/src/bundle/config.rs
  kind: git-path
  fingerprint:
    blob_sha: 637a0f33818ff6548c4bd96590bceaecd5fa1b7b
last_modified: 2026-10-04T20:39:29Z
---
# Catalog identity and explicit scope

A bundle remains an ordinary directory with path-based concept IDs. The optional catalog adds
namespaced identity and local locations; it does not require an identity manifest or an ontology.
Conflicting or overlapping registrations are configuration errors, not content conformance errors.

Primary selection and examination scope are separate. Catalog registration alone does not imply
traversal. Report requested scope, actual examined material and unavailable members. Ordinary
relative links retain layout semantics across registered roots, so distributing one bundle alone
may break links. Never infer equivalence from matching names.

Optional `bundle_ref` source metadata must agree with ordinary `resource` meaning. Snapshot
expectations and ordinary live targets are separate evidence. A current candidate never fulfills
a known mismatch or accepts itself as a substitute. Local Git inspection must not fetch objects;
resolution does not verify claims, refresh fingerprints, or execute computation artifacts.
