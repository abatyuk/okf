# Architecture: Rust crate & module layout

This document sketches the code structure for the OKF harness described in `INTENT.md`.
It is a design sketch, not final — module boundaries encode contracts decided in the intent,
and open crate-level tensions are called out explicitly at the end.

## Workspace shape

A Cargo **workspace** with a thin binary over a fat library. The split *is* the "hands vs.
brain" boundary from the intent: `okf-core` is pure deterministic logic, `okf-cli` is
arg-parsing + output + exit codes, and the agent skills live entirely outside Rust — they
shell out to `okf-cli`.

```
okf/
  Cargo.toml                 # [workspace]
  crates/
    okf-core/                # library — all deterministic logic, no argv, no stdout
    okf-cli/                 # binary `okf` — clap, output rendering, exit-code mapping
```

Two crates for now. Natural **later** split points, if compile times or reuse demand them:
`okf-model` (types), `okf-ontology`, `okf-fingerprint` (which pulls in git/network). Resist
splitting until there is a reason — a single core crate with clean modules refactors easily.

## `okf-core` module tree

```
okf-core/src/
  lib.rs
  error.rs            # OkfError (thiserror); each variant maps to an exit class (2/3/4)

  model/              # in-memory types, no I/O
    concept.rs        # Concept, ConceptId (= bundle-relative path w/o .md)
    frontmatter.rs    # Frontmatter: ORDER-PRESERVING, keeps unknown keys (the linchpin)
    trust.rs          # TrustTier, Verified/Generated/Status/StaleAfter, Actor (human:/process:/…)
    source.rs         # Source, SourceKind, Fingerprint (kind-specific map)
    link.rs           # Link parse + classify (bundle-relative `/…`, relative `./…`)

  bundle/
    loader.rs         # walk dir → Vec<Concept>; reserved filenames (index.md/log.md)
    resolve.rs        # bundle location: arg → config → env → cwd
    config.rs         # okf config model

  parse/
    markdown.rs       # split frontmatter/body; heading tree; section extraction
    yaml.rs           # lossless load (order-preserving mapping)
    writer.rs         # lossless serialize back (round-trip guarantee lives here)

  ontology/
    schema.rs         # ConceptType, Field, FieldType, ReferenceRule, Cardinality
    field_types.rs    # `extends` resolution + object composition + cycle detection
    load.rs           # parse ontology.yaml
    edit.rs           # add/update/remove concept types (feeds `okf ontology …`)

  graph/
    build.rs          # forward + reverse adjacency from concepts
    backlinks.rs
    affected.rs       # reverse walk; direct default, --transitive [--depth N]
    render.rs         # mermaid / graphml / dot emitters

  fingerprint/
    mod.rs            # trait Fingerprinter; dispatch on SourceKind
    canonicalize.rs   # text normalization (LF, trailing-ws, blank-line rules)
    git.rs            # git-commit, git-path (blob object id)
    text.rs           # line-range, markdown-heading
    url.rs            # etag / last-modified / body   [feature = "url-sources"]

  check/
    validate.rs       # the 3 conformance rules ONLY
    stale.rs          # drift = recorded vs recomputed fingerprint (+ stale_after)
    lint/
      mod.rs          # rule engine, Severity, Finding, --fail-on threshold
      rules/          # broken_link, missing_description, ontology_violation, orphan, …

  query/
    browse.rs         # index.md traversal; synthesize a directory view when absent
    search.rs         # search + list (list = search, no filter)
    show.rs
    resolve.rs        # link/id → path
    stats.rs
    diff.rs           # concept-level diff vs a git ref

  mutate/
    init.rs  add.rs  edit.rs  mv.rs  rm.rs  verify.rs  refresh.rs
    #   add   = scaffold from ontology
    #   mv    = rename + rewrite inbound links (uses graph::backlinks)
    #   verify/refresh = trust + fingerprint writers

  render/
    index.rs          # index.md generation (progressive disclosure)
    docs.rs           # html/md/pdf/graphml/obsidian    [feature = "docs-*"]

  output/
    record.rs         # NDJSON record types: Concept (mirrors frontmatter), Finding, Affected, Change
    ndjson.rs         # serializer

  ports/              # effect boundaries (traits) for hermetic tests
    fs.rs  git.rs  clock.rs  net.rs
    #   git.rs = thin wrapper over the `git` CLI (subprocess), NOT a git library.
    #   Runtime-optional: only git-based source kinds and `okf diff` need it.
```

## `okf-cli` module tree

```
okf-cli/src/
  main.rs
  cli.rs              # clap derive: command tree, groups, global --json/--bundle
  commands/           # one thin fn per command: parse args → call okf-core → hand result to output
    query.rs  check.rs  mutate.rs  render.rs  ontology.rs
    schema.rs         # emits `okf schema` NDJSON
  output.rs           # render a core result as text OR NDJSON (--json)
  exit.rs             # Result/OkfError + findings → exit code (0/1/2/3/4, --fail-on)
```

## Cross-cutting decisions (encoded in the boundaries)

1. **`model::frontmatter::Frontmatter` is the linchpin.** Everything round-trips through it,
   so it must preserve key order and unknown keys (an `IndexMap`-backed mapping, not a fixed
   struct). The NDJSON `Concept` record is this map serialized to JSON plus computed
   `id`/`trust_tier` — exactly the "mirrors frontmatter" decision — so
   `output::record::Concept` should *wrap* `Frontmatter`, not re-declare fields.

2. **Core is pure; all effects go through `ports/`.** `fs`, `git`, `clock`, `net` as traits,
   with real impls in production and fakes in tests. This makes `fingerprint` and `stale`
   testable without a real repo or network, and keeps `check`/`query` deterministic. Set up
   on day one — retrofitting effect injection later is painful.

3. **`okf schema` is *derived* from `cli.rs`, never hand-maintained.** Two sources of truth
   for the command surface will drift. Either introspect the clap `Command` tree or define
   commands in one registry that both clap and `schema.rs` read.

4. **`check` returns findings; `cli::exit` decides the code.** Core never calls
   `std::process::exit`. `Finding { severity }` + the `--fail-on` threshold live in core; the
   0/1/2/3/4 mapping lives in `okf-cli/exit.rs`. Keeps core reusable by other consumers and by
   the future `attest` command.

## Dependencies (starting point)

- **Always:** `clap` (derive), `serde` + `serde_json`, `serde_yaml`, `indexmap`,
  `pulldown-cmark` (markdown/heading extraction), `sha2`, `thiserror`, `ignore`/`walkdir`,
  `regex` (field `pattern`).
- **Git — no library dependency.** Git operations shell out to the `git` **CLI** via
  `ports::git`. This is a *runtime-optional* dependency: only the `git-commit` / `git-path`
  source kinds and `okf diff <ref>` invoke it; if `git` is absent from `PATH`, those specific
  operations fail cleanly (exit 3) while everything else works. Commands used:
  `git hash-object <path>` (git-path blob id), `git log -1 --format=%H -- <path>`
  (git-commit), `git show <ref>:<path>` / `git ls-tree` (diff). No `gix`, no `git2`, no
  libgit2 build step — consistent with "if you can `git clone`, you can ship it."
- **Features:** `url-sources` (network fingerprints; pulls `ureq`), `docs-html` / `docs-pdf` /
  `docs-obsidian` (heavy/optional renderers; keeps the base binary lean and answers the
  "docs formats v1 vs later" open question). `petgraph` is optional — the graph walks are
  simple enough to hand-roll and skip the dependency.

## Open crate-level tensions (decide before scaffolding)

- **Lossless YAML is two problems.** Concept **frontmatter** needs order + unknown-key
  preservation, which an order-preserving `serde_yaml::Mapping` gives. But **`ontology.yaml`
  editing** (`okf ontology add/update`) ideally preserves *comments*, which serde-based YAML
  drops entirely. Options:
  - **(a)** Accept comment loss in `ontology.yaml` for v1 (simplest; serde_yaml everywhere).
  - **(b)** A CST / edit-in-place approach for `ontology.yaml` only (e.g. yaml-rust2-based
    surgical edits) so comments survive — more code, but ontology files are hand-edited and
    commented.
  - *Leaning (a) for v1*, documented as a known limitation; (b) as a later upgrade.

## Resolved

- **Git: CLI, not a library.** No `gix` / `git2` dependency; shell out to the `git` CLI via
  `ports::git`, runtime-optional (git-based source kinds and `okf diff` only). See
  Dependencies above.
