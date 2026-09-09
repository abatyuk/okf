# okf — an agent-agnostic harness for the Open Knowledge Format

`okf` is a single Rust binary for working with [OKF](https://github.com/GoogleCloudPlatform/open-knowledge-format)
bundles — directories of markdown-with-frontmatter "concepts". It is designed to be driven
equally by **humans** (readable text output) and **agents** (line-delimited JSON), and is the
deterministic "hands" beneath a set of agent skills that do the judgment work.

- **Deterministic core** (`okf-core`) — parse, validate, lint, query, graph, fingerprint, render.
- **Thin CLI** (`okf-cli`) — 28 commands, grouped by verb, with a machine-discoverable schema.
- **Agent skills** (`skills/`) — packaged as the **`okf` plugin** for Claude Code and Codex:
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
cargo build --release -p okf-core --features url-sources
```

Requires a recent stable Rust. `git` on `PATH` is needed only for git-based source kinds and
`okf diff`; everything else works without it.

### Generated references

Two doc surfaces are generated from `okf schema` and must be regenerated when the CLI changes:
the per-skill CLI reference (`skills/*/okf-cli-reference.md`) and the `## Arguments`
section of each command concept (`knowledge/commands/*.md`). `cargo xtask install` / `cargo
xtask build` do this for you; `cargo xtask docs` runs it on its own, and `cargo xtask docs
--check` fails (exit 1) if anything is stale — wire that into CI. A bare `cargo build` /
`cargo install` cannot regenerate them (a build script can't run the not-yet-built binary), so
use the `xtask` wrappers.

## Quickstart

```sh
okf init mybundle                     # create an empty bundle (+ starter ontology.yaml)
okf add notes/hello --type Note --title "Hello"   # scaffold a concept from the ontology
okf browse mybundle                   # read/synthesize the root index.md
okf browse mybundle --directory notes # descend one directory without loading its concepts
okf list mybundle                     # human table: id, type, trust tier, title
okf list mybundle --json              # one JSON object per concept (NDJSON)
okf validate mybundle                 # conformance only (exit 1 if nonconformant)
okf lint mybundle                     # advisory findings (broken links, missing desc, …)
okf graph mybundle --format mermaid   # render the link graph
okf docs mybundle --format index      # write progressive-disclosure index.md files
```

The bundle argument is an **optional trailing positional** on every command; when omitted it is
resolved via the [configuration](#configuration) precedence below.

## Configuration

Which bundle a command operates on is resolved in this order, **highest precedence first**:

1. the explicit **bundle argument** (`okf list ./mybundle`),
2. the **`OKF_BUNDLE`** environment variable (a session-level override),
3. the **`bundle` key of the nearest `okf.toml`** (a project default), then
4. the **current directory**.

The resolved path must exist and be a directory, or the command fails with an environment error
(exit 3).

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
A `concept` record mirrors the concept's frontmatter **verbatim** plus a computed `id` and
`trust_tier`:

```jsonc
{"id":"/metrics/revenue","type":"Metric","title":"Revenue","tags":["finance"],
 "verified":[{"by":"process:dbt","at":"2026-02-01"}],"trust_tier":"machine-confirmed"}
```

`okf schema` emits the whole command surface as NDJSON (a header line + one line per command
with its group, `mutates` flag, args, and output stream) so an agent can discover capabilities
without hard-coding them.

## Commands

| Group | Commands |
|-------|----------|
| **meta** | `schema`, `version` |
| **query** | `list`, `search`, `show`, `backlinks`, `graph`, `resolve`, `ontology list`, `ontology show` |
| **check** | `scan`, `validate`, `lint`, `stale`, `affected`, `diff`, `stats` |
| **mutate** | `init`, `add`, `edit`, `mv`, `rm`, `verify`, `refresh`, `ontology add/update/remove` |
| **render** | `docs` (`--format html\|md\|pdf\|graphml\|obsidian\|index`) |

Highlights:

- **`validate` vs `lint`** — `validate` enforces only OKF's three conformance rules (parseable
  frontmatter, non-empty `type`, reserved-filename structure) and stays permissive about
  everything else. `lint` is where opinions live, with `error`/`warn`/`info` severities.
- **`mv`** rewrites every inbound link when it renames a concept (a concept's id *is* its path,
  so a naive move would break references).
- **`edit`** losslessly changes both frontmatter and body. Frontmatter: `--set key=value`
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

## Exit codes

`0` success/clean · `1` reportable findings at/above threshold (`validate` nonconformance;
`lint` findings ≥ `--fail-on`, default `error`; discovery commands only with `--fail-on`) ·
`2` usage · `3` environment/IO · `4` internal.

## Trust & sources

Trust tiers are **derived**, never asserted: a concept's `verified` actors decide its tier
(`human:` prefix → human-reviewed; other actors → machine-confirmed; none → unverified).
Source drift uses **typed source kinds** (`git-commit`, `git-path`, `markdown-heading`,
`line-range`, `file`, `url`) recorded in `sources[]` with a `kind` + `fingerprint`; `refresh`
re-records them after you've reconciled a change. Git source paths resolve from the Git
worktree root; file and text-excerpt source paths resolve from the bundle root.

## Ontology

A project-local `ontology.yaml` (outside the bundle, so the bundle stays spec-pure) defines
concept types, their typed fields, and typed reference rules with cardinality. `lint` enforces
it advisorily; `add` scaffolds from it. Manage it with `okf ontology add/update/remove`.

## Agent plugins

The agent skills ship as an **`okf`** plugin for both Claude Code and Codex. This repository is a
single-plugin marketplace for each agent. `skills/` is the canonical source; `cargo xtask docs`
also mirrors it into the self-contained Codex package.

### Claude Code

Claude Code manifests live in `.claude-plugin/` (`marketplace.json` + `plugin.json`), and Claude
auto-discovers the canonical `skills/` directory.

Install:

```text
/plugin marketplace add abatyuk/okf   # this repo on GitHub, or a full git URL / local path
/plugin install okf
```

### Codex

Codex marketplace metadata lives in `.agents/plugins/marketplace.json`, and its plugin package
lives in `plugins/okf/`.

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
| Review & attest | `/okf:review-attest` | `$okf:review-attest` |
| Reorganize the tree | `/okf:reorganize` | `$okf:reorganize` |

The skills drive the `okf` CLI, so install the binary too (`cargo install --path crates/okf-cli`,
or `cargo xtask install`). Each skill bundles a generated `okf-cli-reference.md` it reads from its
own directory, so it never has to probe the CLI to learn arguments. Run `cargo xtask docs --check`
in CI to verify both plugin packages still contain the same skill content.

## Development

```sh
cargo test        # ~139 tests across okf-core and okf-cli
cargo run -p okf-cli --bin okf -- <command>
```
