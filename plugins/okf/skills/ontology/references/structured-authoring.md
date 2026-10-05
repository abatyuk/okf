# Author nested metadata and semantic relationships

Use the structured CLI inputs for modeled changes. Preserve existing uncommitted work and record
baseline validation/lint findings. `--set` still names literal scalar keys; dots do not traverse.

## Structured CLI authoring

Set a complete metadata value with YAML (JSON syntax also works), or update one object property:

```sh
okf edit procedures/onboarding knowledge/product --set-yaml 'norms=@norms.yaml' --dry-run
okf edit procedures/onboarding knowledge/product --set-path 'deadline.within=72'
okf edit procedures/onboarding knowledge/product --unset-path 'deadline.unit'
```

`--set-yaml key=value` replaces the entire named value, including lists; `@file` reads the value
from a file and `-` reads stdin. Null stores null; `--unset` deletes a literal key. `--set-path`
parses its value as YAML, creates missing intermediate objects, and refuses scalar/list traversal.
Bare path segments match `[A-Za-z_][A-Za-z0-9_-]*`; other keys need bracket JSON
quoting. Bracket JSON quoting names literal keys, e.g. `--set-path '["deadline.within"]=72'`. These flags
also work on `add` except deletion and patch flags, which belong to `edit`.

For lists or guarded nested changes use an RFC 6902 patch, supplied as one YAML/JSON array:

```sh
okf edit procedures/onboarding knowledge/product --patch @changes.yaml --dry-run
```

```yaml
- op: test
  path: /norms/0/bearer
  value: /teams/platform.md
- op: replace
  path: /norms/0/deadline/within
  value: 48
- op: add
  path: /norms/-
  value: {bearer: /teams/security.md, action: review-security-access}
```

`add`, `remove`, `replace`, `move`, `copy`, and `test` use concrete JSON Pointer addresses, not
query selectors containing `[]`. A failed operation leaves the concept unchanged. Patches operate
on frontmatter; body text uses the existing body/section flags.

Structured ontology flags replace one named declaration completely:

```sh
okf ontology update Specification knowledge/product --ref 'subjects:Component:0..n' --relationship-yaml 'specification={reference: subjects, kind: specifies, inverse: specified-by}'
okf ontology update Procedure knowledge/product --field-yaml 'norms={type: list, item: Norm, min: 1}' --ref-yaml 'bearers={selector: "norms[].bearer", target: Team, cardinality: "1..n"}' --relationship-yaml 'obligations=@obligations.yaml'
okf ontology update Procedure knowledge/product --remove-relationship obligations
okf ontology field-type add Deadline knowledge/product --from deadline.yaml
okf ontology field-type update Deadline knowledge/product --from deadline.yaml
okf ontology field-type remove Deadline knowledge/product --dry-run
okf ontology update Procedure knowledge/product --from procedure-type.yaml --dry-run
```

`ontology add/update --from` accepts a single concept-type definition. Updates preserve omitted
named declarations and replace supplied ones. Field-type update replaces the whole reusable
definition. Repeat inherited fields when an explicit fields map replaces inherited counterparts.
File/flag overlaps and conflicting operations are rejected. Removal remains explicit.

Apply dependent definitions together with `okf ontology apply knowledge/product --from changes.yaml
--dry-run`. The input uses the ontology's `field_types` and `concepts` maps, plus explicit removals:

```yaml
field_types:
  Deadline:
    base: object
    fields:
      within: {type: int, required: true, min: 1}
      unit: {type: enum, values: [hours], required: true}
concepts:
  Procedure:
    fields:
      deadline: {type: Deadline}
    remove:
      relationships: [obsolete-rule]
remove:
  field_types: [OldDeadline]
  concepts: [OldProcedure]
```

Bulk apply validates the combined result before writing once. Omitted declarations remain;
supplied named declarations replace completely. Per-concept `remove` supports `fields`,
`references`, and `relationships`. Top-level `remove` supports `field_types` and `concepts`.

Structured inputs accept exactly one YAML document, reject duplicate/non-string mapping keys,
tags, anchors, aliases, merge keys and nonfinite numbers, and permit only one stdin consumer per invocation. Input fragment comments
need not survive insertion. Use `--dry-run` to review changes, then inspect the actual diff; YAML
presentation can normalize, so do not claim exact formatting preservation. Meaningful concept
edits apply the existing verification invalidation and generation timestamp behavior.

After writes inspect effective definitions with `ontology show`, run `validate` and advisory
`lint`, and inspect semantic edges and inverse backlinks. Successful checks establish neither
policy compliance nor target consent. Preserve unrelated extensions and provenance; malformed
existing YAML still needs a narrow source repair before the mutation command can load it.

The example below is a small uncataloged bundle rooted at `knowledge/product`. File headings give
bundle-relative destinations. Merge the relevant declarations into existing files; do not replace
an existing sidecar with this entire example. For a new empty bundle the complete files can be used.

## `ontology.yaml`

```yaml
okf_ontology: "0.1"
field_types:
  Deadline:
    base: object
    additional_properties: false
    fields:
      within: {type: int, required: true, min: 1}
      unit: {type: enum, values: [hours], required: true}
  Norm:
    base: object
    fields:
      bearer: {type: string, required: true}
      action: {type: string, required: true}
      deadline: {type: Deadline, required: true}
  ClosedNorm:
    extends: Norm
    additional_properties: false
concepts:
  Team: {}
  Procedure:
    fields:
      norms: {type: list, item: ClosedNorm, min: 1}
    references:
      bearers:
        selector: norms[].bearer
        target: Team
        cardinality: 1..n
    relationships:
      obligations:
        reference: bearers
        kind: obliges
        inverse: obligated-by
        attributes:
          action: norms[].action
          deadline: norms[].deadline
```

Explicit `fields`, `values`, and `item` replace inherited counterparts. To add a field in a derived
object with an explicit `fields` map, repeat the inherited fields it must retain. Named field types
do not create graph edges: the `references` selector does. Relationship attribute selectors pair
with the target in the same concrete list occurrence. For cross-bundle targets, qualify target type
restrictions, e.g. `acme.people:Team`; identical type spelling does not establish equivalence.

## `procedures/onboarding.md`

```markdown
---
type: Procedure
title: Account onboarding
description: Assign access reviews to the responsible teams.
norms:
  - bearer: /teams/platform.md
    action: review-platform-access
    deadline: {within: 72, unit: hours}
  - bearer: /teams/security.md
    action: review-security-access
    deadline: {within: 24, unit: hours}
---
Arrange reviews by [Platform](/teams/platform.md) and [Security](/teams/security.md).
```

## `teams/platform.md`

```markdown
---
type: Team
title: Platform
description: Reviews platform access.
---
Platform owns the platform access review.
```

## `teams/security.md`

```markdown
---
type: Team
title: Security
description: Reviews security access.
---
Security owns the security access review.
```

## Check the modeled behavior

```sh
okf ontology show Procedure knowledge/product --json
okf validate knowledge/product --json
okf lint knowledge/product --fail-on never --json
okf search knowledge/product --type Procedure --project norms --expand obligations --target-field title --full-scan --json
okf backlinks teams/platform knowledge/product --details --json
```

Check that Platform's semantic edge carries 72 hours and Security's carries 24; inverse output
must preserve the same conditions. A nested filter matching Platform and 24 hours can still match
this document through different records. The retrieval skill's
[query planning example](../../retrieval/references/query-planning.md) explains that distinction.

For a meaningful raw edit to an existing concept, mirror the CLI lifecycle behavior: update an
existing `generated.at` to the actual edit time and clear active `verified` events. Preserve any
needed historical evidence in the recovery trail, not as current verification. Do not invent a
generation actor, add verification, refresh source fingerprints, or change source timestamps as
part of a YAML edit. Metadata-only model changes do not establish document verification. Route
changed material claims to source review and `review-verify` when requested.

Missing/null, blank required strings, and empty required lists are distinct from optional fields.
Do not infer requiredness from observed presence alone. Unsupported extension constraints stay
preserved and must be reported as unenforced. A depth/finding limit means the check is incomplete;
do not report a clean exhaustive check from an exit code alone.
