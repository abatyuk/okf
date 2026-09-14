---
type: DomainType
title: Config
description: 'The okf config model: project-local settings consulted after an explicit bundle argument and OKF_BUNDLE, before falling back to the current directory.'
module: bundle
defined_in:
- /components/bundle
sources:
- resource: crates/okf-core/src/bundle/config.rs
  kind: git-path
  fingerprint:
    blob_sha: b3e5561afe4b90dd97729a1c1c0081fb497ef767
last_modified: 2026-09-07T18:22:07Z
---
# Config

The [bundle module](../components/bundle.md)'s OKF config model: project-local settings such as
the bundle location, consulted after an explicit argument and `OKF_BUNDLE`, before falling back
to the current directory. `init` uses the same resolution order while allowing the target not to
exist yet.

## Schema

```rust
pub struct Config {
    /// Default bundle dir, relative to the config file's directory (or absolute).
    pub bundle: Option<String>,
}
```

Defined in `okf-core`.
