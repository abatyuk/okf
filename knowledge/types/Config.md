---
type: DomainType
title: Config
description: Project-local catalog, typed bundle selection, and per-bundle interpretation settings.
module: bundle
defined_in:
- /components/bundle
sources:
- resource: crates/okf-core/src/bundle/config.rs
  kind: git-path
  fingerprint:
    blob_sha: 637a0f33818ff6548c4bd96590bceaecd5fa1b7b
last_modified: 2026-10-04T20:39:29Z
---
# Config

The [bundle module](../components/bundle.md)'s project-local configuration. A nearest `okf.toml`
can select a catalog and a typed default (`id` or `path`), retain the deprecated path-only `bundle`
compatibility setting, and apply settings keyed by registered bundle identity. The reserved
`default` settings entry applies to uncataloged use. An unregistered path never borrows another
bundle's named settings.

Catalog and ontology paths resolve relative to the configuration file. Catalog directory
locations resolve relative to the catalog. Explicit options override applicable bundle settings;
these override built-in defaults. Invalid selectors or incompatible legacy/default settings are
configuration errors, not OKF conformance errors. Consult the generated CLI reference for exact
selection flags and effective-settings inspection.

Defined in `crates/okf-core/src/bundle/config.rs`.
