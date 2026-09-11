# DomainType

* [Actor](Actor.md) - Who performed a verification, carried as a prefixed string such as human: or process:.
* [Affected record](Affected.md) - The NDJSON record for an impact-query result: a concept reached by the affected reverse walk and why it was reached.
* [Cardinality](Cardinality.md) - How many targets a reference (or list field) may have: 0..1, 1..1, 0..n, or 1..n.
* [Change record](Change.md) - The NDJSON record for a diff result: a concept that was added, removed, or changed versus a git ref.
* [Concept](Concept.md) - A single knowledge unit: its ConceptId, parsed Frontmatter, and markdown body.
* [Concept record (NDJSON)](ConceptRecord.md) - The machine representation of a concept: frontmatter JSON plus collision-safe computed identity, trust, lifecycle, generation, and verification views.
* [ConceptId](ConceptId.md) - A concept's identity, equal to its bundle-relative path without the .md suffix (for example /metrics/revenue).
* [ConceptType](ConceptType.md) - An ontology entry: a named type with its typed fields and reference rules.
* [Config](Config.md) - The okf config model: project-local settings (such as the bundle location) consulted during bundle resolution between the CLI arg and the environment.
* [Field](Field.md) - A typed field on a concept type: a name, a FieldType, and a required flag.
* [FieldType](FieldType.md) - The type of a field: string, text, int, bool, date, datetime, uri, enum, list, object, or a name defined under field_types.
* [Finding](Finding.md) - A single lint result: a severity, a rule name, the concept it applies to, and a message.
* [Fingerprint](Fingerprint.md) - A kind-specific record of a source's state at last sync (a blob id, a content hash, an etag).
* [Frontmatter](Frontmatter.md) - The order-preserving, unknown-key-preserving YAML mapping at the top of a concept.
* [Link](Link.md) - A parsed reference to another concept, classified as bundle-relative, relative, bare, or external.
* [OkfError](OkfError.md) - The crate's error type.
* [ReferenceRule](ReferenceRule.md) - A typed edge rule: a frontmatter key on the source concept, the allowed target type(s), and a cardinality.
* [Severity](Severity.md) - A lint finding's level: error, warn, or info.
* [Source](Source.md) - A provenance entry in sources[]: the OKF fields plus a typed kind and a recorded fingerprint, so stale has something to compare against.
* [SourceKind](SourceKind.md) - The typed kind of a source: git-commit, git-path, markdown-heading, line-range, file, or url.
* [TrustTier](TrustTier.md) - The derived trust level of a concept: unverified, machine-confirmed, or human-reviewed.
* [Verified](Verified.md) - A single verification entry in a concept's verified list: an actor plus a timestamp.
