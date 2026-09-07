---
type: DomainType
title: SourceKind
description: 'The typed kind of a source: git-commit, git-path, markdown-heading, line-range, file, or url.'
module: model
defined_in:
- /components/model
sources:
- resource: ../crates/okf-core/src/model/source.rs
  kind: git-path
  fingerprint:
    blob_sha: ae3eabcbf52969391c8eded9a73b9e868720d1cd
last_modified: 2026-09-07T18:22:07Z
---
# SourceKind

The typed kind of a source: git-commit, git-path, markdown-heading, line-range, file, or url. Each kind knows how to compute and compare a fingerprint.

## Schema

```rust
pub enum SourceKind {
    GitCommit, GitPath, MarkdownHeading, LineRange, File, Url,
    Other(String),  // producer-defined; the set is extensible
}
```

Defined in `okf-core`.
