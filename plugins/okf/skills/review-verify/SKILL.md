---
name: review-verify
description: Review OKF concepts against their declared sources and append document-level verified events with the real reviewer actor. Use for sign-off or re-verification; do not use for executing or attesting a computation run.
---

# Review and verify concepts

`okf verify` records that a document was checked against its sources or resource. It does not
execute an Attested Computation and is never a substitute for a deterministic run attestation.

## CLI and review scope

Prefer `OKF_BUNDLE`; the fully qualified write is `okf verify <concept-id> <bundle> --by <actor>`.
Before using an unfamiliar command or flag, read its entry in `references/cli.md`. If the installed
version differs or rejects documented syntax, use that command's `--help` output as the runtime
authority.

1. Open a named concept directly. When the review scope is not already specified, select concepts
   using `okf list <bundle> --json`, `okf stats <bundle>`, and change context. Establish the requested
   bundle scope first. Check scan completeness before claiming the selection covers every concept;
   use a sufficient `--scan-limit` or deliberate `--full-scan` for an exhaustive inventory, otherwise
   report the observed selection. Page limits do not reduce scan work. Check `okf stale
   <bundle>` but keep lifecycle expiry, source credibility/modification, and fingerprint drift as
   separate review facts. `unrecorded` means no fingerprint baseline, `missing` means fingerprinting
   failed, and `drifted` means the baseline changed. Stale findings exit successfully by default;
   use `--fail-on any` for a health gate and inspect the findings either way. Kind-less sources are
   not fingerprint-checked. Review evidence directly; do not refresh merely to make checks green.
2. Read the complete relevant document, using `okf show <concept-id> <bundle> --outline`, then `okf
   show <concept-id> <bundle> --lines <START:END>` when large. The concept argument is required even
   when `OKF_BUNDLE` is set. Plain `show --json` is metadata only. Review all material claims,
   including tables, footnotes, and qualifications, against declared sources, applicable `resource`,
   and linked concepts. For sliced reads, track sections/claims covered and continue through the
   remaining document. Do not append document-wide verification after reviewing only selected
   sections.
3. Resolve internal sources with document context. Use `okf artifact resolve <resource> <bundle>
   --from <concept-id>` and bounded `okf artifact show` for opaque files; use `okf show` for
   concepts. A `scope`, `missing`, or `blocked` result needs an explicit review decision and may
   prevent verification. Consult the artifact reference for external source access and truncation. A
   request to check named external sources supplies read authorization within that scope;
   inaccessible evidence remains a review gap. Inspection never authorizes code execution.
4. Confirm lifecycle and generation state: absent status is stable; expired `stale_after` is
   independent from fingerprint drift; generation after the latest check requires a new review.
5. Only after every material claim is checked and the document-wide review passes, run `okf verify
   <concept-id> <bundle> --by <actor>`. Valid actors are `human:<id>`, `process:<id>`, or
   `<agent>/<version>`. Use `human:` only after that identified person explicitly confirms the
   completed review; prior confirmation of this same review counts. Never invent an agent version if
   the actual identity is unavailable. The CLI appends a valid explicit-offset timestamp.
6. Re-read the concept and report the exact appended event and derived tier. Multiple `verified`
   entries are independent checks. Meaningful content changes invalidate current verification;
   history elsewhere does not make changed content reviewed.

## Computation boundary

For a request to attest a numeric or other computation result, do not call `okf verify` as a proxy.
The current CLI can run `okf computation check <concept-id> <bundle>` to inspect the exact `Attested
Computation` contract, but it cannot execute or attest it. Report this limitation and the missing
run verdict.

`references/` is only an optional convention: Markdown files are concepts and non-Markdown files are
opaque artifacts. Cite both the declaring concept and inspected source/artifact. Report verified
concepts and events, or the incomplete review scope and outstanding claims/sources. Include
lifecycle limitations and computation boundaries only when relevant.

## Bundle identity and examination scope

Keep bundle paths and catalog IDs distinct. `OKF_BUNDLE` always selects a path; use the
`--bundle-id <id>` for a registered identity. When a catalog is configured, use `okf catalog` to inspect effective local
registrations, overrides, availability, and per-bundle settings. Catalog paths are relative to
`okf.toml`, locations are relative to the catalog, and machine-local overrides change effective
locations rather than accepting different snapshots. Inspect effective settings before relying on
a per-bundle ontology or view. Equal or nested registered roots are configuration errors.

Registration alone does not authorize catalog-wide traversal. Start with the selected bundle;
request additional bundles explicitly with repeatable `--scope-bundle <id>` or deliberate
`--catalog-scope` when the task requires them. Report requested and examined
scope and unavailable members. Out-of-scope references remain unknown. Ordinary relative paths
retain their meaning across registered boundaries; distributing one bundle alone can break them.
Never infer another bundle from a matching filename or title.

## Resolution is separate from review

Catalog identity, relationship resolution, file digests, and requested-snapshot matches establish
reference evidence only. Inspect the source actually used and label unresolved expectations or
current candidates. Never manufacture a verification event from a successful resolver or lint
result. Review the relevant nested relationship conditions and each bundle's interpretation
settings before making a source-backed sign-off.
