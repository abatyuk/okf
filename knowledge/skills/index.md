# Skill

* [okf-infer-ontology skill](okf-infer-ontology.md) - Derives an ontology from an existing bundle by analyzing observed types, common fields, and reference patterns, then commits the proposal via okf ontology.
* [okf-ingest skill](okf-ingest.md) - Scans a repository, researches its structure and artifacts, and adds concepts attributed back to the source files.
* [okf-init skill](okf-init.md) - Bootstraps an empty bundle, establishes an ontology.yaml, and scaffolds the first concepts.
* [okf-migrate skill](okf-migrate.md) - Classifies each source document into an ontology concept type, writes conformant frontmatter, attributes the original as a typed source, and validates the result.
* [okf-ontology skill](okf-ontology.md) - Shapes concept types (typed fields and typed reference rules with cardinality) and commits them through okf ontology so ontology.yaml stays valid and lossless.
* [okf-reorganize skill](okf-reorganize.md) - Restructures the directory tree with okf mv so every inbound link is rewritten and nothing dangles, because a concept id is its file path.
* [okf-retrieval skill](okf-retrieval.md) - The primary consumer skill.
* [okf-review-attest skill](okf-review-attest.md) - Walks a human or agent through verifying concepts and appends verified entries via okf verify, driving concepts up the trust tiers.
* [okf-update skill](okf-update.md) - Finds what drifted with stale, affected, and diff, then rewrites prose, re-attributes sources, and re-fingerprints with refresh.
