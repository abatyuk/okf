---
type: DomainType
title: SnapshotResult
description: Separate requested snapshot status, resolution evidence, resolved node and current candidate.
module: graph
defined_in:
- /components/graph
sources:
- resource: crates/okf-core/src/graph/catalog.rs
  kind: git-path
  fingerprint:
    blob_sha: 361ebf706bb821babc7cf87064cd72697b64612a
last_modified: 2026-10-04T20:39:29Z
---
# SnapshotResult

A result for optional provenance `bundle_ref.snapshot` expectations. Commit, mutable ref, human
label and raw-byte digest are individually optional. Exact identifiers constrain evidence; hints
alone do not establish an immutable snapshot.

Status and evidence remain distinct from ordinary resource resolution. An unresolved request can
have a separately labeled current candidate; a known mismatch never becomes a successful match.
Returning a candidate does not accept it, rewrite authored metadata, refresh source fingerprints,
or create a document verification event.
