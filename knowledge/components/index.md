# Component

* [bundle module](bundle.md) - Bundle loading and location.
* [check module](check.md) - Conformance, optional-family, lifecycle, drift, and compatibility diagnostics.
* [fingerprint module](fingerprint.md) - Source fingerprinting, dispatched on SourceKind: git-commit and git-path via the git CLI, line-range and markdown-heading over canonicalized text, and optional url fingerprints behind a feature flag.
* [graph module](graph.md) - Forward and reverse adjacency built from concept links.
* [model module](model.md) - In-memory domain types with no I/O: Concept, ConceptId, the order-preserving Frontmatter, trust types, source types, and Link parsing.
* [mutate module](mutate.md) - Atomic, self-validating concept writes that preserve unknown YAML meaning and body text while applying bundle mutations.
* [okf-cli crate](okf-cli.md) - The thin binary named okf.
* [okf-core crate](okf-core.md) - The deterministic core library.
* [ontology module](ontology.md) - The tool-local ontology: concept types, typed fields with composition via extends, and typed reference rules with cardinality.
* [output module](output.md) - NDJSON record types and their serializer.
* [parse module](parse.md) - Semantic markdown and YAML handling that preserves unknown keys, values, order, and body text while YAML presentation may normalize.
* [ports module](ports.md) - Effect boundaries expressed as traits: fs, git, clock, and net.
* [query module](query.md) - Read-only concept queries, corrected path resolution, and bounded artifact/computation inspection.
* [render module](render.md) - Derived output artifacts, including deterministic specification-conformant index.md generation.
