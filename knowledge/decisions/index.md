# DesignDecision

* [Check returns findings; the CLI decides the exit code](check-returns-findings.md) - Core produces Findings with severities and never calls process::exit.
* [Git via the CLI, not a library](git-cli-not-library.md) - Git operations shell out to the git CLI through the git port, with no gix or git2 or libgit2 build step.
* [Hands vs. brain: thin CLI over a fat core](hands-vs-brain.md) - The workspace splits deterministic logic (okf-core) from arg-parsing, output, and exit-code mapping (okf-cli), and pushes all judgment into agent skills that shell out to the binary.
* [Lossless, order-preserving frontmatter](lossless-frontmatter.md) - Frontmatter is an order-preserving mapping that keeps unknown keys, not a fixed struct.
* [Preserve comments in ontology.yaml edits](lossless-ontology-yaml.md) - Typed serialization validates ontology edits; a key-path merge restores comments to surviving keys.
* [Pure core; all effects go through ports](ports-boundary.md) - fs, git, clock, and net are traits with real implementations in production and fakes in tests.
* [Trust tiers are derived, never asserted](derived-trust-tiers.md) - A concept's tier is computed from its verified actors (a human: actor gives human-reviewed, other actors give machine-confirmed, none gives unverified).
* [okf schema is derived from the command tree](derived-schema.md) - The machine-readable command surface is introspected from the single clap definition, never hand-maintained, because two sources of truth for the surface would drift.
* [validate is conformance; lint is opinion](validate-vs-lint.md) - validate enforces only OKF's three hard rules and stays permissive about everything else.
