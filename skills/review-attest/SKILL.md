---
name: review-attest
description: Use this when the user wants to review and attest OKF concepts to raise their trust — "verify these concepts", "mark this as human-reviewed", "attest the bundle", "raise the trust tier", "sign off on these docs", "record that a human/process checked this". Walks a human or agent through verifying concepts and appends verified entries via okf verify, driving concepts up the trust tiers (unverified → machine-confirmed → human-reviewed). Read-only Q&A is okf:retrieval; syncing drift is okf:update.
---

# Review & attest — raise concepts up the trust tiers

Trust is **computed, not executed** (v1). The derived trust tier is a deterministic function
of the `verified` field's actor prefixes:
- `human:<name>` → **human-reviewed** (highest),
- `process:<name>` → **machine-confirmed**,
- `<agent>/<ver>` → agent attestation,
- no relevant `verified` entry → **unverified**.

`okf verify` is the write-side of trust: it appends a `verified` entry (actor + timestamp)
losslessly. Your job is to make sure a real review actually happened before recording it.

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

## Steps

1. **Find what needs review.** Prioritize with `okf list <bundle> --json` (each record carries
   the derived `trust_tier`) and `okf stats <bundle>` (trust-tier distribution). Target
   `unverified` concepts, and re-review anything recently changed. Cross-check `okf stale
   <bundle>` — a drifted concept should be reconciled (via `okf:update` + `okf refresh`) before
   it's attested, not signed off while wrong.

2. **Review each concept (the judgment part).** `okf show <bundle> <concept-id>` and actually
   verify the content:
   - Is the prose accurate and current? Do its `sources[]` still support the claims (spot-check
     with `okf resolve` / read the artifact)?
   - Are references valid (`okf backlinks`, `okf lint`)?
   For a **human** attestation, present the concept to the human and get an explicit yes. Do not
   record `human:` on your own judgment — that prefix means a person reviewed it.

3. **Record the attestation.** Only for concepts that genuinely passed review:
   `okf verify <concept-id> --by <actor>`, choosing the actor prefix that reflects who reviewed:
   - `--by human:<name>` when a person signed off,
   - `--by process:<name>` for a deterministic automated check,
   - `--by <agent>/<ver>` for an agent's own attestation.
   The command appends the `verified` entry with a timestamp; the tier recomputes automatically.

4. **Confirm the tier moved.** Re-run `okf show <concept-id>` or `okf list --json` and check the
   derived `trust_tier` rose as expected. If it didn't, the actor prefix was likely wrong.

5. **Report.** List concepts attested with the actor used and the resulting tier, concepts you
   declined to attest (and why — e.g. stale, unsupported claim, missing human sign-off), and any
   left for follow-up.

## Guardrails
- Never fabricate a `human:` attestation. The whole value of the tier is that `human:` means a
  human really looked. If you (the agent) reviewed it, use the agent actor form, not `human:`.
- Attest content, not vibes: tie each sign-off to accurate prose and supporting sources.
- Don't attest drifted concepts — reconcile via `okf:update`/`okf refresh` first.
- `okf verify` is lossless and additive; it appends to `verified`, preserving history.
