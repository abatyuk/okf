# Intent: an agent-agnostic harness for OKF

I want to build an agent-agnostic harness to support **OKF** (Open Knowledge Format,
spec: https://raw.githubusercontent.com/GoogleCloudPlatform/knowledge-catalog/refs/heads/main/okf/SPEC.md).

The harness is deliberately **two layers**:

1. **Core (Rust CLI) — the deterministic "hands."** Pure functions over markdown + YAML:
   parse, validate, lint, query, render. No LLM. Single static binary, `cat`-friendly,
   fast. Its job is to be a clean, stable *tool API* that both humans and agents call.
2. **Skills — the agent "brain."** Inherently LLM-driven tasks (migrate, ingest, update)
   that *call the core commands*. They never reimplement core logic; they orchestrate it.

The core must be usable by both humans and agents, so every command supports
human output (text/markdown) and machine output (JSON).

---

## Design principles (decided)

- **Respect OKF's permissiveness.** The spec says consumers MUST NOT reject a bundle for
  missing optional fields, unknown `type` values, unknown keys, broken links, or missing
  `index.md`. The harness honors this. Our opinions are *advisory*, never conformance.
- **Conformance vs. opinion are separate commands.**
  - `validate` = the spec's **three hard rules only** (parseable frontmatter, non-empty
    `type`, reserved-filename structure for `index.md`/`log.md`). It MUST exit 0 on a
    spec-conformant bundle even if it has broken links and unknown types.
  - `lint` = all of *our* opinions (broken links, missing `title`/`description`, staleness,
    ontology violations), each with a severity (`error`/`warn`/`info`) and configurable via
    the sidecar config.
- **Trust is computed, not executed (v1).** The derived trust tier
  (unverified / machine-confirmed / human-reviewed) is a deterministic function of the
  `verified` field's actor prefixes — cheap and safe, so we compute it. We do **not** run
  `Attested Computation` code (`computation`/`executor`/`attester`) in v1: that means
  arbitrary code execution (bigquery/dbt/python), sandboxing, and credentials. Execution is
  a later, opt-in, explicitly-sandboxed command (`okf attest --run`), never a default.
- **Ontology lives in a sidecar `ontology.yaml`, not in the bundle.** OKF intentionally has
  no central type registry. Our ontology (allowed types/capabilities and their fields) lives
  in a tool-local `ontology.yaml` used for `lint` and `new` scaffolding. References are
  **typed constraints**: a rule declares which type may reference which (e.g. a `Policy` may
  reference a `Computation`), with cardinality (0..1, 0..n, 1..n). `lint` enforces these as
  advisory findings. The bundle stays spec-pure and portable; the ontology travels separately.
- **Writes are idempotent and semantically preserving.** Any command that writes files (`add`, `edit`,
  `mv`, `rm`, `verify`, `refresh`, `lint --fix`, `docs --format index`, `ontology` edits)
  MUST preserve unknown frontmatter keys, values, and key order on round-trip (spec recommends
  preserving unknown keys). YAML comments/scalar presentation may normalize.
- **Machine output is NDJSON that mirrors frontmatter.** With `--json`, every command emits
  newline-delimited JSON, one object per line. A concept record carries **the same fields as
  the concept's frontmatter verbatim** (if a doc has a `version` key, the record has it too),
  plus a small set of computed fields (`id`, derived `trust_tier`). No fixed/whitelisted
  concept schema — the record is the frontmatter. Streams cleanly for
  large bundles, no need to buffer a whole array. Human output stays text/markdown.
- **Stable machine contract.** Stable exit codes, stable per-line JSON shapes, and
  `okf schema` so an agent can discover the whole surface.

---

## Core CLI (Rust, deterministic)

`<bundle>` is an optional directory: local dir (default), or from config file, or from an
environment variable. Record-producing commands use the global `--json` flag for NDJSON;
`schema` is always NDJSON, while graph/document artifacts are wrapped as records under
`--json`. Commands are grouped by verb category.

> **Implemented convention (v1):** `<bundle>` is the **optional trailing positional** on every
> command. Resolution precedence (highest first): explicit arg → `$OKF_BUNDLE` env →
> `bundle` key of the nearest `okf.toml` (found by walking up from cwd; its path is relative
> to the config file) → cwd. Where a command has a required argument,
> it comes first and the bundle trails, e.g. `okf show <concept-id> [bundle]`,
> `okf diff <ref> [bundle]`; `okf graph [bundle] [--root <concept>] --format …`. The `<bundle> <arg>`
> orderings shown in some examples below are illustrative — the trailing-bundle form is
> authoritative. Also note two v1 behaviors: `lint --fix` is a no-op (no rule is auto-fixable
> yet), and `--fail-on` on discovery commands (`stale`/`affected`/`diff`/`stats`/`scan`) fails
> on any non-empty result for a non-`never` threshold, since those items carry no per-item severity.

### META (discovery)
- `okf schema` — print machine-readable CLI metadata (all commands, args, NDJSON output
  shapes) as JSON. Primary agent-discovery surface.
- `okf version` — CLI version and the OKF spec version(s) it supports.

### QUERY (read-only lookups)
- `okf browse <bundle> [--directory <path>]` — read that directory's checked-in `index.md`,
  or synthesize the same view in memory when absent. This is the specification-native
  progressive-disclosure entry point.
- `okf search <bundle> [--tag] [--type] [--text] [--field]` — ranked search by tag, type,
  repeatable text, or field. Text supports phrase/all/any/literal modes, field scopes,
  reader-visible Markdown, deterministic relevance or ID ordering, bounded match evidence,
  and result limits.
- `okf list <bundle>` — **alias of `okf search` with no filter**; lists all concepts (ID,
  type, title, status, trust tier).
- `okf show <concept-id> [bundle]` — show one concept's full content.
- `okf links <concept-id> [bundle]` — normalized direct outbound concept links, including
  whether each target exists.
- `okf backlinks <concept-id> [bundle]` — concepts that link to a given concept.
- `okf graph <bundle> [--root <concept>] [--direction outgoing|incoming|both] [--depth N]
  [--format mermaid|...]` — render the whole graph or a bounded rooted neighborhood.
- `okf resolve <link> [bundle]` — resolve a link/concept-ID to a concrete file path (agent utility).
- `okf artifact list/resolve/show` — inventory, resolve, and retrieve bounded path-valued
  artifacts, including the optional `references/` convention. Opaque artifacts are never
  executed; remote retrieval is explicit and policy-gated.
- `okf ontology list` / `okf ontology show <name>` — inspect defined types and their rules.

### CHECK (read-only diagnostics)
- `okf scan <bundle>` — recursively scan a repository and report what would be analyzed.
- `okf source-scan <directory>` — inventory all source files without concept parsing.
- `okf doctor <bundle>` — compatibility-first preflight for existing bundles; safe fixes are
  opt-in, atomic, and never invent semantic metadata.
- `okf validate <bundle>` — **conformance only** (the spec's three hard rules).
- `okf lint <bundle>` — advisory checks (broken links, missing descriptions, ontology
  violations, orphaned concepts) with configurable `error`/`warn`/`info` severities.
  (`--fix` moves it into MUTATE, below.)
- `okf stale <bundle>` — **drift detection (automatic).** For each concept, resolve typed
  `sources[]` fingerprint extensions and compare a recorded fingerprint against the current artifact;
  report what drifted. Each source declares a **kind** (`git-commit`, `git-path`,
  `markdown-heading`, `line-range`, `file`, `url`, …) and the checker dispatches on it — a
  `git-path` compares blob SHA, a `markdown-heading` compares the hash of that section's
  content, a `git-commit` compares the recorded hash against the current one, etc. The kind
  set is extensible. The core discovers drift with no input.
- `okf affected <bundle> --changed <link>...` — **impact query (explicit).** Given a list of
  updated links/concept-IDs/resources (args or stdin), walk the reverse-link graph
  (backlinks, `sources`) to report every concept that needs review. Direct dependents by
  default; `--transitive [--depth N]` follows the cascade (unbounded, or capped at N hops).
- `okf diff <git-ref> [bundle]` — concept-level diff vs a git ref: which concepts were
  added / removed / modified. Feeds the update workflow.
- `okf stats <bundle>` — summary: concept counts by type, trust-tier distribution, stale
  count, orphan count. A dashboard for humans and agents.
- `okf computation check <concept-id>` — inspect a standard computation contract and artifact
  resolution without executing it.

`stale` finds drift you didn't know about; `affected` computes the blast radius of changes
you already know; `diff` tells you what changed vs a baseline. Together they are the
deterministic foundation of the "update docs" skill.

### MUTATE (writes to bundle / ontology — lossless, self-validating)
- `okf init <bundle>` — create a new empty OKF bundle.
- `okf add <path>` — add a new concept document (scaffolded from the ontology).
  - `okf add policies/travel_expenses --type Policy --title "Travel and expense policy" --description "Rules and reimbursement rates for business travel."`
  - `okf add computations/mileage_calc --attested --runtime python --computation references/computations/mileage.py --title "Mileage reimbursement calculator"`
- `okf edit <concept-id> --set <key>=<value>...` — set/update frontmatter fields losslessly.
- `okf mv <old-id> <new-id>` — move/rename a concept **and rewrite every inbound link**
  (concept ID = file path, so a naive rename silently breaks references).
- `okf rm <concept-id>` — remove a concept; refuse (or warn) if backlinks would dangle unless `--force`.
- `okf verify <concept-id> --by <actor>` — append a `verified` entry (the write-side of
  trust; records human/machine review with actor + timestamp).
- `okf refresh <concept-id>` — re-record extension fingerprints after a change is acknowledged
  without overwriting standard source `last_modified`; an expired `stale_after` remains expired.
- `okf lint <bundle> --fix` — apply auto-fixable lint findings.
- `okf ontology add <name> [--field ...] [--ref ...]` — define a new concept type with its
  fields and typed reference rules.
- `okf ontology update <name> ...` — modify fields/references of an existing concept type.
- `okf ontology remove <name>` — remove a concept type.

### RENDER (derive output artifacts)
- `okf docs <bundle> --format html|md|pdf|graphml|obsidian|index` — generate documentation
  from a bundle. `--format index` writes `index.md` files into the bundle (progressive
  disclosure); the other formats emit external artifacts. (Absorbs the former `okf index`.)
- `okf pack <bundle>` — package the bundle as a tarball for distribution *(candidate)*.

### Trust / attestation
- Trust tier is surfaced by `okf list` / `okf show` (derived from document-level `verified`);
  written by `okf verify` after checking a concept against its sources.
- Runtime attestation is separate. `okf computation check` only inspects a standard contract;
  execution and receipt attestation remain a later, opt-in, sandboxed capability.

---

## Skills (agent-driven, call the core)

- **Initialize a bundle** — scaffold from ontology.
- **Migrate existing documentation** into a bundle — from whatever form (spec, plan, README).
- **Research a directory/repository and ingest** documentation as concepts.
- **Update documentation from recent changes** — driven by `okf stale` / `okf affected`: the
  core finds *what* drifted or is impacted deterministically; the agent decides *how* to
  rewrite the prose and re-attribute `sources`.
- **Manage the ontology** — add / update / remove a concept type (its fields and typed
  reference rules). The agent shapes *what* a concept type should look like; it commits the
  change via the deterministic `okf ontology` commands so `ontology.yaml` stays valid and lossless.
- **Answer questions over a bundle (retrieval)** — the agent-facing payoff of OKF: use
  `search` → `show` → `links`/`backlinks` → a bounded `graph` for progressive disclosure
  without loading the whole bundle. Arguably the primary consumer skill.
- **Infer an ontology** — analyze an existing bundle and *propose* an `ontology.yaml`
  (observed types, common fields, reference patterns). The reverse of authoring it by hand;
  a fast on-ramp for the migrate/ingest skills.
- **Review & verify** — check concepts against their declared sources and append document-level
  `verified` entries via `okf verify`; never substitute this for runtime attestation.
- **Repair / upgrade** — run compatibility-first `okf doctor`, preview allow-listed safe repairs,
  preserve extensions, and stop for user decisions where target semantics are ambiguous.
- **Reorganize / refactor a bundle** — restructure the directory tree using `okf mv` so all
  inbound links stay intact.
- **Bundle health report** — run `stats` + `lint` + `stale` and summarize with prioritized
  recommendations *(candidate)*.
- **Deduplicate / merge** — find near-duplicate concepts and merge them, preserving
  `sources` and links *(candidate)*.

---

## Ontology shape (`ontology.yaml`) — draft

A tool-local, spec-external file. It defines **concepts** (concept types), their typed
**fields** (in addition to OKF built-ins), advisory **trust** expectations, and typed
**reference rules**. Each key under `concepts:` **is the OKF `type` string verbatim** — types
may contain spaces, so quote them (`"BigQuery Table":`). No separate alias is needed. A
reference rule maps a frontmatter key on the source concept (e.g. `computations:`) to a list
of links, and constrains the target type(s) and cardinality. `lint` enforces all of this
advisorily; `add` uses it to scaffold.

```yaml
okf_ontology: "0.1"          # version of THIS ontology file's schema (our tool), not OKF's

# Optional: reusable typed field definitions (DRY). Composable via `extends`.
field_types:
  money:
    base: string
    pattern: '^\d+(\.\d{2})? [A-Z]{3}$'    # e.g. "42.00 USD"
  positive_money:
    extends: money                         # inherits base + pattern…
    min: 0                                  # …and adds a constraint
  contact:
    base: object                           # object types compose sub-fields
    fields:
      name:  { type: string, required: true }
      email: { type: uri }

# Source kinds `okf stale` knows how to fingerprint. Built-ins shown;
# producers MAY register additional kinds (each needs a fingerprint + compare rule).
source_kinds: [git-commit, git-path, markdown-heading, line-range, file, url]

concepts:

  Policy:
    description: A rule or standard people are expected to follow.
    requires: [description]            # OKF built-in fields this concept type must carry
    fields:
      owner:        { type: string, required: true }
      effective_date: { type: date, required: true }
      review_cycle: { type: enum, values: [monthly, quarterly, annual] }
    trust:
      min_tier: human-reviewed         # lint warns if the concept's derived tier is lower
    references:
      computations: { target: "Attested Computation", cardinality: 0..n }
      supersedes:   { target: Policy, cardinality: 0..1 }

  "Attested Computation":
    description: A sanctioned way to compute a value.
    attested: true                     # reserved for this exact standard OKF type
    fields:
      runtime: { type: enum, values: [bigquery, dbt, python], required: true }
    references:
      inputs: { target: [BigQuery Table, Metric], cardinality: 1..n }   # union of targets

  "BigQuery Table":                    # OKF type with a space — quoted key, used verbatim
    description: A BigQuery table.
    fields:
      resource: { type: uri, required: true }

  Metric:
    description: A named measurable quantity.
    references:
      computed_by: { target: "Attested Computation", cardinality: 1..1 }
```

**Field `type` vocabulary (draft):** `string`, `text`, `int`, `bool`, `date`, `datetime`,
`uri`, `enum` (+ `values`), `list` (+ item `type`), `object` (+ nested `fields`), plus any
name defined under `field_types`. Every field takes `required` (default false).

**Composition (decided):** `field_types` compose. A definition may `extends` another (merge
base + constraints, most-derived wins on conflict) and `object` types nest `fields`, so
custom types can be built up rather than repeated. Cycles in `extends` are rejected at load.

**Cardinality vocabulary:** `0..1`, `1..1`, `0..n`, `1..n`. Applies to reference rules and
to `list` fields.

---

## Typed `sources[]` shape (for `stale` / `refresh`) — draft

OKF's `sources[]` entries already carry `resource`, `id`, `author`, `usage_count`,
`last_modified`. Extra keys are spec-legal (consumers MUST tolerate unknown keys), so we add
two: `kind` (the source kind, from the extensible registry) and `fingerprint` (what the last
sync recorded, so `stale` has something to compare against). This stays conformant — a
consumer that ignores our keys still reads a valid OKF concept.

```yaml
# frontmatter of e.g. computations/mileage_calc.md
sources:
  - resource: src/billing/mileage.py     # OKF built-in: the artifact
    kind: git-path                        # ours: how to fingerprint + compare
    fingerprint: { blob_sha: 3f9a1c… }    # ours: recorded at last sync

  - resource: src/billing/mileage.py#L40-88
    kind: line-range
    fingerprint: { content_sha256: 9b2e… }

  - resource: docs/policy.md#reimbursement-rates
    kind: markdown-heading
    fingerprint: { section_sha256: 71cc… }

  - resource: https://api.example.com/rates
    kind: url
    fingerprint: { etag: "W/\"abc\"", last_modified: 2026-08-01T00:00:00Z }
```

- `okf stale` recomputes the current fingerprint per `kind` and flags any that differ from
  the recorded one (also flags missing artifacts and `stale_after` expiry).
- `okf refresh <id>` re-reads the artifacts and rewrites only the extension `fingerprint`,
  preserving standard source signals — the "I've reviewed this fingerprint baseline" operation.
- `fingerprint` is a small `kind`-specific map, so new kinds bring their own fields without a
  schema migration.

### Fingerprint canonicalization — proposal

The sleeper risk: if the same artifact hashes differently on macOS vs CI, every concept reads
as perpetually drifted. So each kind pins *exactly* what gets hashed.

- **`git-commit`** — fingerprint is the 40-hex commit SHA. "Current" = last commit touching
  the referenced path (`git log -1 --format=%H -- <path>`); drift when it differs. No hashing.
- **`git-path`** — fingerprint is git's own **blob object id** of the file in the worktree
  (`git hash-object <path>`). Git-native, deterministic, respects `.gitattributes` EOL rules.
  If the path is untracked, degrade to `file`.
- **`file`** (non-git) — `sha256` over the file's **raw bytes**, byte-exact, no normalization.
  For text where you want EOL tolerance, prefer `line-range` / `markdown-heading` instead.
- **`line-range`** / **`markdown-heading`** — extract the region, then apply **text
  canonicalization** before `sha256`:
  1. decode UTF-8; 2. normalize `CRLF`/`CR` → `LF`; 3. strip trailing whitespace per line;
  4. strip leading/trailing blank lines from the region; 5. join with `\n`, no trailing
  newline. `markdown-heading` region = the lines after the heading up to (excluding) the next
  heading of equal-or-shallower level (nested subsections **included**); the heading text line
  itself is excluded, so re-titling the heading alone doesn't churn.
- **`url`** — prefer `etag`; else `last_modified`; else `sha256` of the response body.
- All hashes are lowercase hex. The kind + normalization version is implied by the running
  `okf`; a future change to canonicalization is a major bump (would re-flag everything once).

### Exit codes — proposal

Uniform across commands so humans, CI, and agents can branch on them:

| Code | Meaning |
|------|---------|
| `0` | Success; nothing to report. |
| `1` | Success; reportable results at/above the command's failure threshold (see below). |
| `2` | Usage error — bad flags/args. |
| `3` | Environment/IO error — bundle or `ontology.yaml` missing/unreadable. |
| `4` | Internal error / bug. |

- **CHECK commands fail (exit 1) by default on findings**: `validate` on nonconformance,
  `lint` on findings at/above `--fail-on` severity (default `error`).
- **Discovery commands are informational (exit 0 even with results)**: `stale`, `affected`,
  `diff`, `stats`, `scan`. Opt into failing with `--fail-on` (e.g. `okf stale --fail-on any`
  in CI to fail the build when docs drift).
- **QUERY / MUTATE / RENDER** exit 0 on success regardless of result size (an empty search is
  not a failure); nonzero only for codes 2–4.

---

## `okf schema` NDJSON output — draft

Discovery surface for agents. First line is a header describing the tool/contract; each
subsequent line describes one command. Every line is a standalone JSON object so an agent can
stream and filter without a JSON-array parser.

```jsonl
{"kind":"schema","tool":"okf","tool_version":"0.2.4","okf_spec":["0.2"],"ndjson_schema":"2","global_args":[{"name":"json","type":"bool","default":false}],"bundle_resolution":["explicit","env:OKF_BUNDLE","config:okf.toml","cwd"]}
{"kind":"command","name":"list","group":"query","mutates":false,"mutates_when":null,"summary":"List all concepts (search with no filter)","args":[{"name":"bundle","kind":"positional","type":"path","required":false,"repeatable":false,"default":null,"resolution":["explicit","env:OKF_BUNDLE","config:okf.toml","cwd"]}],"output":{"stream":"concept"}}
```

- Every command declares `group` (`meta`/`query`/`check`/`mutate`/`render`) and a `mutates`
  capability boolean; `mutates_when` identifies conditional writers such as `doctor` and
  `docs --format index`.
- Arguments declare actual scalar/list/path/int types, typed possible values, and bundle
  resolution rather than pretending the effective default is always the current directory.
- `output.stream` names the per-line record type or types a command emits under `--json`
  (`concept`, `finding`, `affected`, `change`, …). **`concept` records are not a fixed schema
  — they mirror the concept's frontmatter verbatim** plus computed `id` / `trust_tier`
  (see the NDJSON principle). Finding/affected/change records *do* have documented shapes.
- Global flags and bundle-resolution precedence live in the header rather than being inferred
  from a fake per-command default.

---

## Open decisions (naming / contract — not yet settled)

- **`--fail-on` severity vocabulary.** Confirm the accepted values (`never`/`info`/`warn`/
  `error`/`any`) and each check command's default threshold.
- **`edit --set` typed values.** How do `--set key=value` args express non-scalars (lists,
  the `verified`/`sources` structured families) on the CLI, or is that add-only via `verify`?
- **`docs` external formats.** Which of html/pdf/graphml/obsidian are v1 vs later, and what
  renders them (pandoc dependency vs. pure-Rust)?
