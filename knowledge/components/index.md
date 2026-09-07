# Component

* [bundle module](bundle.md) - Bundle loading and location.
* [check module](check.md) - Read-only diagnostics.
* [fingerprint module](fingerprint.md) - Source fingerprinting, dispatched on SourceKind: git-commit and git-path via the git CLI, line-range and markdown-heading over canonicalized text, and optional url fingerprints behind a feature flag.
* [graph module](graph.md) - Forward and reverse adjacency built from concept links.
* [model module](model.md) - In-memory domain types with no I/O: Concept, ConceptId, the order-preserving Frontmatter, trust types, source types, and Link parsing.
* [mutate module](mutate.md) - Lossless, self-validating writes to the bundle: init, add (scaffold from ontology), edit, mv (rename plus rewrite of every inbound link), rm, and the verify/refresh trust and fingerprint writers.
* [okf-cli crate](okf-cli.md) - The thin binary named okf.
* [okf-core crate](okf-core.md) - The deterministic core library.
* [ontology module](ontology.md) - The tool-local ontology: concept types, typed fields with composition via extends, and typed reference rules with cardinality.
* [output module](output.md) - NDJSON record types and their serializer.
* [parse module](parse.md) - Lossless markdown and YAML handling: split frontmatter from body, build the heading tree, extract sections, load YAML order-preservingly, and serialize back with a round-trip guarantee.
* [ports module](ports.md) - Effect boundaries expressed as traits: fs, git, clock, and net.
* [query module](query.md) - Read-only lookups: search (list is search with no filter), show, resolve a link or id to a path, stats summary, and a concept-level diff against a git ref.
* [render module](render.md) - Derived output artifacts: progressive-disclosure index.md generation and the optional html/md/pdf/graphml/obsidian document renderers behind feature flags.
