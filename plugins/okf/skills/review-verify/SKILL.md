---
name: review-verify
description: Review OKF concepts against their declared sources and append document-level verified events with the real reviewer actor. Use for sign-off or re-verification; do not use for executing or attesting a computation run.
---

# Review and verify concepts

`okf verify` records that a document was checked against its sources or resource. It does not
execute an Attested Computation and is never a substitute for a deterministic run attestation.

## CLI and review scope

Read `okf-cli-reference.md` in this skill directory. Prefer `OKF_BUNDLE`; the fully qualified
write is `okf verify <concept-id> <bundle> --by <actor>`.

1. Select concepts using `okf list <bundle> --json`, `okf stats <bundle>`, and change context.
   Check `okf stale <bundle>` but keep lifecycle expiry, source credibility/modification, and
   fingerprint drift as separate review facts.
2. Read the complete relevant document, using `okf show --outline` and targeted lines when large.
   Check its claims against declared sources, applicable `resource`, and linked concepts.
3. Resolve internal sources with document context. Use `okf artifact resolve <resource> <bundle>
   --from <concept-id>` and bounded `okf artifact show` for opaque files; use `okf show` for
   concepts. A `scope`, `missing`, or `blocked` result needs an explicit review decision and may
   prevent verification. Remote access needs explicit policy. Inspection never authorizes code
   execution.
4. Confirm lifecycle and generation state: absent status is stable; expired `stale_after` is
   independent from fingerprint drift; generation after the latest check requires a new review.
5. Only after the review passes, run `okf verify <concept-id> <bundle> --by <actor>`.
   Valid actors are `human:<id>`, `process:<id>`, or `<agent>/<version>`. Use `human:` only after
   that identified person explicitly confirms. The CLI appends a valid explicit-offset timestamp.
6. Re-read the concept and report the exact appended event and derived tier. Multiple `verified`
   entries are independent checks. Meaningful content changes invalidate current verification;
   history elsewhere does not make changed content reviewed.

## Computation boundary

For a request to attest a numeric or other computation result, do not call `okf verify` as a
proxy. The current CLI can run `okf computation check <concept-id> <bundle>` to inspect the exact
`Attested Computation` contract, but it cannot execute or attest it. Report this limitation and
the missing run verdict. A future execution skill may proceed only after runtime support and
separate execution authorization exist.

`references/` is only an optional convention: Markdown files are concepts and non-Markdown files
are opaque artifacts. Cite both the declaring concept and inspected source/artifact. Report
verified concepts and events, declined reviews with reasons, lifecycle and source limitations,
and any computation request routed to inspect-only handling.
