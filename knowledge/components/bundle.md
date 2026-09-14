---
type: Component
title: bundle module
description: Bundle loading and location.
layer: core
depends_on:
- /components/parse
- /components/model
sources:
- resource: crates/okf-core/src/bundle/mod.rs
  kind: git-path
  fingerprint:
    blob_sha: df6427c20b959576e8a0bccb67cca51835f2d11d
- resource: crates/okf-core/src/bundle/resolve.rs
  kind: git-path
  fingerprint:
    blob_sha: 39b04f6bb501d995aadb58564dc29c37d9668882
- resource: crates/okf-core/src/bundle/config.rs
  kind: git-path
  fingerprint:
    blob_sha: b3e5561afe4b90dd97729a1c1c0081fb497ef767
last_modified: 2026-09-07T18:47:09Z
---
# bundle module

Bundle loading and location. Walks a directory into a set of Concepts (honoring reserved
filenames like index.md and log.md) and resolves which bundle to operate on via explicit
argument, `OKF_BUNDLE`, nearest `okf.toml`, then cwd. Creation uses the same precedence without
requiring the target directory to exist.

Location: `crates/okf-core/src/bundle`.
