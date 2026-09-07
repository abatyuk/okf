---
name: okf-retrieval
description: PRIMARY CONSUMER SKILL. Use this whenever the user asks a question that an OKF bundle can answer — "what does our bundle say about X?", "find the concept for Y", "how are these concepts related?", "what depends on Z?", "explain the policy/metric/computation for …", or any lookup/Q&A over a knowledge bundle. Answers via progressive disclosure — okf search → okf show → okf graph/okf backlinks — so you never load the whole bundle into context. Use this before reading bundle files directly.
---

# Answer questions over a bundle (retrieval)

This is the agent-facing payoff of OKF: answer a question by pulling in only the concepts you
need, via progressive disclosure. **Do not read the whole bundle or cat every file** — that
defeats the format. Let the CLI's query commands narrow first.

## Tool discipline
**CLI argument reference (read first).** This skill bundles the full argument list for every `okf` command as `okf-cli-reference.md` in **this skill's own directory** — read it there (the skill's absolute directory is provided to you when the skill loads; equivalently `${CLAUDE_SKILL_DIR}/okf-cli-reference.md`). Consult it to learn a command's flags; do **not** run `okf <cmd> --help` or `okf schema` just to discover arguments. Every command also takes global `--json` and an optional trailing `bundle` positional.

Discover and inspect everything in the bundle **only through the `okf` CLI** — `okf search`,
`okf list`, `okf show`, `okf graph`, `okf backlinks`, `okf resolve`, `okf stats` (add `--json`
when parsing). Do **not** use Glob, Grep, `find`, or generic file-content search over the
bundle: `okf` already indexes it and supports progressive disclosure, so grepping it is
wasteful and defeats the design. Read a bundle markdown file directly **only when you already
know its exact path** (from `okf resolve` or an `okf show --json` record), and prefer
`okf show` over a raw read. When a raw read is unavoidable, read the **narrowest slice** needed
— a known line range, a section/heading, or a named symbol — never the whole file speculatively.

## Progressive disclosure loop

1. **Locate candidates — `okf search`.** Start narrow. Search by the dimension that matches the
   question (use `--json` / NDJSON to parse):
   - `okf search <bundle> --text "<terms>"` for content,
   - `--tag <tag>`, `--type <Type>`, or `--field <key>=<value>` for structured filters.
   - `okf list <bundle>` (search with no filter) only when you genuinely need the whole
     inventory (id, type, title, status, trust tier).
   Skim the returned ids/titles/types and pick the few most relevant concepts.

2. **Read the relevant concepts — `okf show`.** For each promising candidate,
   `okf show <bundle> <concept-id>` to get its full frontmatter + body. Read only what you need;
   stop once you can answer. Note each concept's `trust_tier` — it signals how much to rely on
   the content (unverified / machine-confirmed / human-reviewed).

3. **Follow relationships — `okf graph` / `okf backlinks`.** When the question is about how
   things connect or what's impacted:
   - `okf backlinks <bundle> <concept-id>` — what references this concept (e.g. which policies
     use a computation).
   - `okf graph <bundle> [subtree] [--format mermaid]` — the local relationship structure; scope
     to a subtree (`okf graph <bundle> api`) rather than the whole bundle.
   - `okf resolve <bundle> <link>` — turn a link/id into a concrete path when you need to chase
     a reference precisely.

4. **Synthesize and answer.** Compose the answer from the concepts you actually opened. **Cite
   the concept ids** you used and surface their trust tier where it matters (e.g. "per the
   human-reviewed Policy `policies/travel_expenses` …"). If the bundle doesn't contain the
   answer, say so — don't fill gaps with outside assumptions.

## Guardrails
- Prefer many small, targeted queries over one broad dump. Widen (more search terms, `okf list`,
  deeper `okf graph`) only if the narrow pass misses.
- This skill is **read-only**: it never mutates the bundle. If the answer reveals the docs are
  wrong or stale, hand off to `okf-update`.
- Be honest about trust: flag when an answer rests on `unverified` concepts.
- Use `--json` output when you need to filter/aggregate results programmatically; text output
  when you're just reading.
