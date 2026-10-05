# okf — an agent-agnostic harness for the Open Knowledge Format

`okf` is a single Rust binary for working with [OKF](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
bundles — directories of markdown-with-frontmatter "concepts". It is designed to be driven
equally by **humans** (readable text output) and **agents** (line-delimited JSON), and is the
deterministic "hands" beneath a set of agent skills that do the judgment work.

- **Deterministic core** (`okf-core`) — parse, validate, lint, query, graph, fingerprint, render.
- **Thin CLI** (`okf-cli`) — commands grouped by verb, with a machine-discoverable schema.
- **Agent skills** (`plugins/okf/skills/`) — packaged as the **`okf` plugin** for Claude Code and Codex:
  migrate, ingest, update, retrieve, manage ontology, etc. (see [Agent plugins](#agent-plugins)).

See `INTENT.md` for the design rationale and `ARCHITECTURE.md` for the crate/module layout.

## Install a release (no Rust required)

Download the archive for your platform from [GitHub Releases](https://github.com/abatyuk/okf/releases):
macOS Apple Silicon (`aarch64-apple-darwin`), macOS Intel (`x86_64-apple-darwin`),
Linux ARM64 (`aarch64-unknown-linux-musl`), or Linux Intel/AMD (`x86_64-unknown-linux-musl`).
Download `SHA256SUMS` from the same release and compare the archive's SHA-256 using
`shasum -a 256 ARCHIVE` (macOS) or `sha256sum ARCHIVE` (Linux). Extract the archive,
then copy `okf` into a directory on PATH, for example:

```sh
mkdir -p "$HOME/.local/bin"
cp okf "$HOME/.local/bin/okf"
chmod +x "$HOME/.local/bin/okf"
export PATH="$HOME/.local/bin:$PATH"
okf version
```

Alternatively, download the standalone installer from a **published release**, inspect it,
then run it with an explicit version. This example installs or updates to 0.3.2:

```sh
curl --fail --location --proto '=https' --proto-redir '=https' \
  https://github.com/abatyuk/okf/releases/download/v0.3.2/install.sh -o install.sh
# Review install.sh before running it.
sh install.sh 0.3.2
```

The optional installer detects your platform, verifies the archive checksum, checks that the
binary runs, and installs to `~/.local/bin`. It needs curl, tar, and sha256sum or shasum;
it does not need Rust, Python, GitHub CLI, or an existing OKF installation. Supply a second
argument to choose another installation directory. To update or roll back, select that
release's version. It does not edit shell configuration or use administrator privileges.
Add `~/.local/bin` to PATH in your shell configuration if needed. An older `okf` elsewhere
on PATH can take precedence; check `command -v okf` and `okf version`.

Skills **only check compatibility** and show these instructions when necessary. They never
run the installer or update the CLI. The current plugin supports stable CLI versions
`>=0.3.2, <0.4.0`; the OKF document specification remains v0.2.

Initial macOS releases are not Developer ID signed or notarized, so Gatekeeper may block
a downloaded executable. Signing/notarization is planned for a later release. macOS builds
use deployment target 13.0; see release notes for tested platforms and limitations.
Git-based operations require `git` on PATH.

Official binaries include `url-sources` for explicit remote artifact retrieval with
`artifact show --fetch`. URL fingerprint refresh is not yet wired to the CLI network client.
Local operations work offline. See [release maintenance](docs/releases.md) for the publishing process.

## Build & install from source

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
# optional remote artifact retrieval (`artifact show --fetch`):
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

Bundle paths and catalog IDs use distinct selectors. Selection follows this order:

1. An explicit trailing bundle path or `--bundle-id` (using both is an error).
2. `OKF_BUNDLE`, which always means a directory path.
3. The registered bundle containing the current directory.
4. `default_bundle` in the nearest `okf.toml`.
5. The sole catalog entry, if unambiguous.
6. The current directory when no catalog is configured; otherwise select a bundle explicitly.

The selected path must exist and be a directory. `init` permits a new target. The nearest
`okf.toml` is discovered by walking upward from the working directory. New configurations use
a typed default; the legacy `bundle = "knowledge"` remains a deprecated path-only alias. If both
are supplied, only equivalent path defaults are accepted.

```toml
# okf.toml
catalog = "okf-catalog.yaml"
default_bundle = { id = "acme.product" }
# For single-bundle use: default_bundle = { path = "knowledge" }

[bundle_settings."acme.product"]
ontology = "tooling/product-ontology.yaml"
```

```yaml
# okf-catalog.yaml
catalog_version: 1
bundles:
  acme.product:
    location: {type: directory, path: knowledge/product}
  acme.finance:
    location: {type: directory, path: knowledge/finance}
```

Catalog and configured ontology paths are relative to `okf.toml`; directory locations are
relative to the catalog. Available registered roots are canonicalized and must not overlap.
Unavailable directories remain visible diagnostics. Each bundle owns its interpretation settings;
`[bundle_settings.default]` applies to uncataloged use. The optional root `ontology.yaml` remains
the discovery fallback and never establishes a bundle boundary.

Local checkout overrides use `catalog_overrides` in `okf.toml`, or `OKF_CATALOG_OVERRIDES`
(the environment value takes precedence). The YAML override file maps `bundles` to paths:

```yaml
bundles:
  acme.finance: ../finance-checkout/knowledge
```

Override paths are relative to that file. Overrides cannot register new IDs or accept different
snapshot content. The CLI reads available local files and Git objects without cloning or fetching.

Selection and examination scope are separate. Catalog registration alone does not include another
bundle in a query, graph or check. Use repeatable `--scope-bundle <id>` or deliberate `--catalog-scope` as described in the [CLI reference](docs/okf-cli-reference.md)
and retain reported examined scope and unavailable members. Cross-bundle ordinary paths preserve
the surrounding layout; copying one directory alone can break those references. Optional
`bundle_ref` source metadata adds target identity and snapshot expectations while retaining
ordinary `resource` meaning. Current candidates stay separate from requested snapshot matches;
resolving a reference never verifies its claims or refreshes its fingerprints.

`okf catalog --json` inspects effective registrations without selecting a primary. Graph targets
can use qualified IDs such as `acme.finance:/policies/margin-standard`. `--revision <ref>` is
available for graph, links, backlinks, resolve, affected, search and list; other reads and writes use
current local content. Additional scope is supported by those commands plus lint; other commands reject scope options.

See [multi-bundle design](docs/multi-bundle-capability.md) and
[structured metadata design](docs/structured-metadata-capability.md) for compatibility and limits.

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
| **meta** | `schema`, `version`, `catalog` |
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
- **`mv`** rebases inbound body links and standard source resources when it renames a concept.
  It refuses unsupported declared nested-reference rewrites; cross-bundle moves require coordinated review.
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

An optional tool-local ontology defines concept types, reusable field types, recursive objects and
lists, and declared reference rules. Configure its path per bundle or use the root `ontology.yaml`
fallback. `lint` checks it advisorily; `validate` retains only OKF conformance rules. Concept types can be managed with `okf ontology add/update/remove`; structured declarations use
`--field-yaml`, `--ref-yaml`, `--relationship-yaml`, or a definition file with `--from`.
`okf ontology field-type add/update/remove` manages reusable definitions. `okf ontology apply`
validates dependent changes together, preserving omitted declarations and using explicit removals.

Concept `add/edit --set-yaml` replaces a complete structured value; `--set-path` sets an object
property while creating missing intermediate objects. `edit --unset-path` removes an object
property, and `edit --patch` applies RFC 6902 operations to frontmatter using concrete JSON Pointer
paths, including list edits and guarded changes. Existing `--set` keeps literal scalar keys.
Use `--dry-run` to review mutations before writing. See the
[structured authoring workflow](plugins/okf/skills/ontology/references/structured-authoring.md)
for file inputs, replacement rules, and bulk removal syntax.

Selectors such as `relations[].target` and `norms[].deadline.from` traverse explicit list records;
`["policy.status"]` selects a literal dotted key. Custom path-looking strings remain opaque unless
a reference is declared. Relationship kinds and attributes pair within the same record. Named
inverse backlinks display incoming assertions, without proving consent or policy compliance.
Recursive lint diagnoses exact YAML types, bounds, patterns, vocabularies, uniqueness and explicitly
closed custom objects. Unknown extensions survive and unsupported constraints remain visible.

Read-only index coverage checks report missing navigation links. Query views, projections, facets,
pagination and related expansion preserve authored frontmatter and report completeness and scope.
Consult the generated CLI reference for exact options and configuration examples. Declared nested
reference rewriting and automatic cross-bundle move orchestration remain deferred.

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

### Source baseline lint setting

Sources with a fingerprint `kind` but no baseline produce `source-unrecorded` warnings by default.
To suppress this finding, add to `okf.toml`:

```toml
[bundle_settings.default.lint]
source_unrecorded = "off"
```

Accepted values are `off`, `info`, `warn`, and `error` (default `warn`). For a named bundle,
use `[bundle_settings."<id>".lint]`. This setting leaves `source-missing` errors and
`okf stale` checks active; it does not record or refresh fingerprints.

All lint rules accept `off`, `info`, `warn`, or `error` under the same settings table.
Omitted settings retain these defaults:

| Setting | Finding rule | Default |
|---------|--------------|---------|
| `broken_link` | broken-link | error |
| `missing_title` | missing-title | warn |
| `missing_description` | missing-description | info |
| `orphan` | orphan | info |
| `ontology_violation` | ontology-violation | warn |
| `spec_v02` | okf-v02 | warn |
| `source_unrecorded` | source-unrecorded | warn |
| `source_missing` | source-missing | error |
| `index_coverage` | index-coverage | warn |

`off` suppresses only the selected lint rule. Conformance validation and `stale` checks
remain independent. `index_exclude` accepts glob patterns for index coverage exclusions.
`finding_budget` is a positive ontology finding limit (default 1,000 per concept).
`--fail-on` controls only the exit threshold; it does not change finding severities.
