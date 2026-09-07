# Command

* [okf add](add.md) - Adds a new concept document, scaffolded from the ontology so required fields and reference keys start present.
* [okf affected](affected.md) - Impact query: given a set of changed links, computes the blast radius of concepts that may need review via a reverse walk, direct by default or --transitive.
* [okf backlinks](backlinks.md) - Lists the concepts that link to a given concept, using the reverse adjacency from the graph.
* [okf diff](diff.md) - Concept-level diff of the working tree against a git ref: which concepts were added, removed, or changed.
* [okf docs](docs.md) - Generates documentation from a bundle: progressive-disclosure index.md files, or html, md, pdf, graphml, or obsidian output via --format.
* [okf edit](edit.md) - Sets or updates frontmatter fields losslessly with --set key=value (scalars only), preserving key order and unknown keys.
* [okf graph](graph.md) - Renders the link graph, or a subtree rooted at a concept, as mermaid, dot, or graphml.
* [okf init](init.md) - Creates a new empty OKF bundle: base structure plus a starter ontology.yaml.
* [okf lint](lint.md) - Advisory checks where the opinions live: broken links, missing descriptions, orphans, and ontology violations, at error/warn/info severities with a --fail-on threshold.
* [okf list](list.md) - Lists all concepts (search with no filter).
* [okf mv](mv.md) - Moves or renames a concept and rewrites every inbound link.
* [okf ontology add](ontology-add.md) - Defines a new concept type with its typed fields and reference rules, written to ontology.yaml through the lossless ontology editor.
* [okf ontology list](ontology-list.md) - Lists the concept types defined in the tool-local ontology.yaml with their descriptions.
* [okf ontology remove](ontology-remove.md) - Removes a concept type from the ontology.
* [okf ontology show](ontology-show.md) - Shows one concept type's fields and typed reference rules with cardinality.
* [okf ontology update](ontology-update.md) - Modifies the fields or references of an existing concept type.
* [okf refresh](refresh.md) - Re-records source fingerprints after a change has been acknowledged, clearing the drift that stale would report.
* [okf resolve](resolve.md) - Resolves a link or concept id to a concrete bundle-relative file path.
* [okf rm](rm.md) - Removes a concept, refusing if backlinks would dangle unless --force is given.
* [okf scan](scan.md) - Walks a bundle and reports the candidate files that would be analyzed, without loading them as concepts.
* [okf schema](schema.md) - Prints machine-readable CLI metadata as NDJSON: every command with its group, mutates flag, args, and output shape.
* [okf search](search.md) - Searches concepts by type, tag, free text, and/or frontmatter field, returning matching concept records.
* [okf show](show.md) - Shows one concept's full content: frontmatter plus body, or the concept record under --json.
* [okf stale](stale.md) - Drift detection: compares recorded source fingerprints against recomputed ones (plus any stale_after date) to find concepts whose sources have moved.
* [okf stats](stats.md) - Bundle summary: counts by type, trust-tier distribution, and orphan count.
* [okf validate](validate.md) - Conformance validation: the spec's three hard rules only (parseable frontmatter, non-empty type, reserved-filename structure).
* [okf verify](verify.md) - Appends a verified entry, the write side of trust, raising a concept's derived tier (unverified to machine-confirmed to human-reviewed).
* [okf version](version.md) - Prints the CLI version and the OKF spec version(s) it supports.
