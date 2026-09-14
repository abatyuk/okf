# okf — an agent-agnostic harness for the Open Knowledge Format

`okf` is a single Rust binary for working with [OKF](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
bundles — directories of markdown-with-frontmatter "concepts". It is designed to be driven
equally by **humans** (readable text output) and **agents** (line-delimited JSON), and is the
deterministic "hands" beneath a set of agent skills that do the judgment work.

- **Deterministic core** (`okf-core`) — parse, validate, lint, query, graph, fingerprint, render.
- **Thin CLI** (`okf-cli`) — 36 commands, grouped by verb, with a machine-discoverable schema.
- **Agent skills** (`plugins/okf/skills/`) — packaged as the **`okf` plugin** for Claude Code and Codex:
  migrate, ingest, update, retrieve, manage ontology, etc. (see [Agent plugins](#agent-plugins)).

See `INTENT.md` for the design rationale and `ARCHITECTURE.md` for the crate/module layout.

## Build & install

```sh
cargo xtask install          # build + install `okf` to ~/.cargo/bin, then regenerate docs
```

`cargo xtask install` wraps `cargo install --path crates/okf-cli` and then regenerates the
[generated references](#generated-references) so they match the freshly built CLI. Prefer it
over a bare `cargo install`, which installs the binary but leaves the references untouched.

Build without installing:

```sh
cargo xtask build --release  # cargo build --release, then regenerate docs (binary: target/release/okf)
cargo build --release        # plain build, no doc regeneration
# optional network fingerprints for `url` sources:
cargo build --release -p okf-cli --features url-sources
```

Requires a recent stable Rust. `git` on `PATH` is needed only for git-based source kinds and
`okf diff`; everything else works without it.

### Generated references

Three doc surfaces are generated from `okf schema` and must be regenerated when the CLI changes:
the [complete developer CLI reference](docs/okf-cli-reference.md), focused per-skill references
(`plugins/okf/skills/*/references/cli.md`), and the `## Arguments` section of each command concept
(`knowledge/commands/*.md`). The skill subsets are selected explicitly in
`xtask/skill-scenarios.json`, keeping each skill self-contained without loading unrelated command
documentation.

`cargo xtask install` / `cargo xtask build` regenerate these files; `cargo xtask docs` runs the
generation on its own, and `cargo xtask docs --check` fails (exit 1) if anything is stale—wire
that into CI. A bare `cargo build` / `cargo install` cannot regenerate them (a build script cannot
run the not-yet-built binary), so use the `xtask` wrappers.

## Quickstart

```sh
okf init mybundle                     # create an empty bundle (+ starter ontology.yaml)
okf add notes/hello --type Note --title "Hello"   # scaffold a concept from the ontology
okf browse mybundle                   # read/synthesize the root index.md
okf browse mybundle --directory notes # descend one directory without loading its concepts
okf list mybundle                     # human table: id, type, trust tier, title
okf list mybundle --json              # one JSON object per concept (NDJSON)
okf search mybundle --text "travel policy" --match phrase --limit 10
okf validate mybundle                 # conformance only (exit 1 if nonconformant)
okf lint mybundle                     # advisory findings (broken links, missing desc, …)
okf doctor mybundle                   # compatibility preflight for existing bundles
okf artifact list mybundle            # inspect optional references/ artifacts
okf links notes/hello mybundle --json # list direct normalized outbound links
okf graph mybundle --root notes/hello --direction both --depth 2 # bounded neighborhood
okf docs mybundle --format index      # write progressive-disclosure index.md files
```

Bundle-aware commands take an **optional trailing positional**; when omitted it is resolved via
the [configuration](#configuration) precedence below. Meta commands take no bundle,
`source-scan` takes an explicit arbitrary directory, and graph roots use `--root`.

## Configuration

Which bundle a command operates on is resolved in this order, **highest precedence first**:

1. the explicit **bundle argument** (`okf list ./mybundle`),
2. the **`OKF_BUNDLE`** environment variable (a session-level override),
3. the **`bundle` key of the nearest `okf.toml`** (a project default), then
4. the **current directory**.

The resolved path must exist and be a directory, or the command fails with an environment error
(exit 3). `init` uses the same precedence but permits the selected target not to exist yet.

**Environment variables**

| Variable | Effect |
|----------|--------|
| `OKF_BUNDLE` | Default bundle directory when no bundle argument is given (overridden by an explicit arg, overrides `okf.toml`). |

**`okf.toml`** — a project-scoped config file discovered by walking **up** from the current
directory to the first `okf.toml` found. It lives *outside* the bundle so the bundle stays
spec-pure. Unknown keys are ignored so the format can grow compatibly. Currently one key:

```toml
# okf.toml
bundle = "docs/kb"   # default bundle dir, relative to THIS file's directory (or absolute)
```

A present-but-malformed `okf.toml` is a usage error (exit 2).

## Output: text vs NDJSON

Pass `--json` for machine output: **newline-delimited JSON**, one object per line, streamable.
Graph and rendered-document artifacts are wrapped in a record with their format and content;
`schema` is already NDJSON with or without the flag.
A `concept` record mirrors the concept's frontmatter **verbatim** plus collision-safe computed
identity, trust, lifecycle, generation, and verification views:

```jsonc
{"id":"/metrics/revenue","type":"Metric","title":"Revenue","tags":["finance"],
 "verified":[{"by":"process:dbt","at":"2026-02-01"}],"trust_tier":"machine-confirmed"}
```

`okf search --text ... --json` additionally returns ranked, bounded match evidence under
`search.score` and `search.matches`.

`okf schema` emits schema contract v2 as NDJSON: a header describing global flags and bundle
resolution, followed by commands with mutation capability/conditions, typed arguments, defaults,
possible values, stdin support, and output streams. Flag names use their actual kebab-case CLI
spelling.

## Commands

| Group | Commands |
|-------|----------|
| **meta** | `schema`, `version` |
| **query** | `list`, `search`, `show`, `links`, `backlinks`, `graph`, `resolve`, `artifact list/resolve/show`, `ontology list`, `ontology show` |
| **check** | `scan`, `source-scan`, `validate`, `lint`, `doctor`, `stale`, `affected`, `diff`, `stats`, `computation check` |
| **mutate** | `init`, `add`, `edit`, `mv`, `rm`, `verify`, `refresh`, `ontology add/update/remove` |
| **render** | `docs` (`--format html\|md\|pdf\|graphml\|obsidian\|index`; only `index` mutates) |

Highlights:

- **`validate` vs `lint`** — `validate` enforces only OKF's three conformance rules (parseable
  frontmatter, non-empty `type`, reserved-filename structure) and stays permissive about
  everything else. `lint` is where opinions live, with `error`/`warn`/`info` severities.
- **`doctor`** uses a tolerant physical-tree scan to show conformance blockers and behavior
  changes before an existing bundle adopts corrected v0.2 semantics. It is read-only unless
  allow-listed `--fix-safe` repairs are explicitly confirmed.
- **`artifact`** supports the optional `references/` convention and standard path-valued fields.
  It distinguishes concepts, opaque files, URLs, scope descriptors, missing paths, and blocked
  paths. Retrieval is bounded and never executes code.
- **`mv`** rewrites every inbound link when it renames a concept (a concept's id *is* its path,
  so a naive move would break references).
- **`edit`** changes both frontmatter and body while preserving unknown YAML values and key order.
  YAML comments and scalar presentation may normalize. Frontmatter: `--set key=value`
  (scalar), `--unset key`, `--add key=value` (append a list item, idempotent), `--remove
  key=value` (drop list items). Body: `--set-body` / `--append-body` / `--clear-body`, or the
  section-aware `--set-section <heading> <text>` / `--append-section` / `--remove-section`
  (heading matched by GitHub slug). Body/section text accepts `@file` or `-` (stdin).
- **`stale` / `affected` / `diff`** find what needs updating: `stale` detects drift against
  recorded source fingerprints, `affected --changed <links>` computes the blast radius of known
  changes, `diff <ref>` shows concept-level changes vs a git ref.
- **`verify`** appends a `verified` entry, raising a concept's trust tier
  (unverified → machine-confirmed → human-reviewed).
- **`edit` invalidates verification** by removing prior `verified` entries, returning the
  concept to `unverified` until it is reviewed again.
- **`show --outline` / `show --lines START:END`** expose a document's heading map and retrieve
  only the relevant numbered slice, avoiding full reads of large concepts.
- **`search`** supports repeatable AND phrases, `phrase|all|any|literal` matching,
  reader-visible Markdown, field scopes, deterministic relevance or ID ordering, bounded JSON
  evidence, and result limits. Unfiltered search remains an exact alias of `list`.
- **`links` / `backlinks` / `graph`** provide progressively wider relationship views: direct
  outgoing links, direct incoming concepts, or a rendered neighborhood constrained by
  `--root <concept>`, `--direction incoming|outgoing|both`, and `--depth N`.
- **`scan --fail-on`**, like other discovery checks, can turn non-empty informational results
  into exit 1 for CI.

## Exit codes

`0` success/clean · `1` reportable findings at/above threshold (`validate` nonconformance;
`lint` findings ≥ `--fail-on`, default `error`; discovery commands only with `--fail-on`) ·
`2` usage · `3` environment/IO · `4` internal.

## Trust & sources

Trust tiers are **derived**, never asserted: a concept's `verified` actors decide its tier
(`human:` prefix → human-reviewed; other actors → machine-confirmed; none → unverified).
Source drift can additionally use **typed source kinds** (`git-commit`, `git-path`, `markdown-heading`,
`line-range`, `file`, `url`) recorded in `sources[]` with a `kind` + `fingerprint`; `refresh`
re-records them after you've reconciled a change. Git source paths resolve from the Git
worktree root; file and text-excerpt source paths resolve from the bundle root.

`kind` and `fingerprint` are tool extensions; a standard OKF source needs only `resource`.
The optional `references/` directory may hold ordinary Markdown concepts or opaque artifacts
such as SQL, Python, schemas, and run instructions. Use `okf artifact` to inspect the latter.

## Ontology

A project-local `ontology.yaml` (outside the bundle, so the bundle stays spec-pure) defines
concept types, their typed fields, and typed reference rules with cardinality. `lint` enforces
it advisorily; `add` scaffolds from it. Manage it with `okf ontology add/update/remove`.

## Agent plugins

The agent skills ship as an **`okf`** plugin for both Claude Code and Codex. This repository is a
single-plugin marketplace for each agent. Both marketplaces reference the same self-contained
package under `plugins/okf/`, whose `skills/` directory is the canonical source.

### Claude Code

Claude Code marketplace metadata lives in `.claude-plugin/marketplace.json`; the package manifest
lives in `plugins/okf/.claude-plugin/plugin.json`. Claude auto-discovers the package's shared
`skills/` directory.

Install:

```text
/plugin marketplace add abatyuk/okf   # this repo on GitHub, or a full git URL / local path
/plugin install okf
```

### Codex

Codex marketplace metadata lives in `.agents/plugins/marketplace.json`; its package manifest lives
in `plugins/okf/.codex-plugin/plugin.json`. It uses the same `plugins/okf/skills/` directory as
Claude Code.

Install this repository as a marketplace, then install the plugin:

```sh
codex plugin marketplace add abatyuk/okf   # or a full Git URL / local path
codex plugin add okf@okf-marketplace
```

Start a new Codex thread after installation so the skills are loaded.

The skills are namespaced by the plugin in both agents:

| Skill | Claude Code | Codex |
|-------|-------------|-------|
| Start a new bundle | `/okf:init` | `$okf:init` |
| Migrate existing docs | `/okf:migrate` | `$okf:migrate` |
| Ingest a repo | `/okf:ingest` | `$okf:ingest` |
| Answer questions (retrieval) | `/okf:retrieval` | `$okf:retrieval` |
| Manage the ontology | `/okf:ontology` | `$okf:ontology` |
| Infer an ontology | `/okf:infer-ontology` | `$okf:infer-ontology` |
| Sync docs after changes | `/okf:update` | `$okf:update` |
| Review & verify documents | `/okf:review-verify` | `$okf:review-verify` |
| Repair or upgrade a bundle | `/okf:repair` | `$okf:repair` |
| Reorganize the tree | `/okf:reorganize` | `$okf:reorganize` |

The skills drive the `okf` CLI, so install the binary too (`cargo install --path crates/okf-cli`,
or `cargo xtask install`). Each skill carries a generated `references/cli.md` containing only its
workflow's commands and reads it when exact flags or output shapes are needed. If the installed
binary version differs from the generated reference, command-specific `--help` is authoritative.
Run `cargo xtask skills` and `cargo xtask docs --check` in CI to validate authored command examples,
reference coverage, and generated documentation.

## Development

```sh
cargo test        # ~139 tests across okf-core and okf-cli
cargo run -p okf-cli --bin okf -- <command>
```
