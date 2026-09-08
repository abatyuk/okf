//! Link parsing and classification (bundle-relative `/…`, relative `./…`, bare ids).
//!
//! OKF links point at other concepts. We recognize three internal shapes plus external
//! URIs (which are never concept edges):
//!
//! - **bundle-relative** — a leading slash, resolved from the bundle root:
//!   `/tables/customers.md` → id `/tables/customers`.
//! - **relative** — `./other.md`, `../metrics/x.md`, resolved against the *directory* of the
//!   concept that contains the link.
//! - **bare** — no leading slash and no `./`/`../`, treated as a bundle-root id
//!   (`tables/customers` → `/tables/customers`).
//! - **external** — anything with a URI scheme (`https://…`, `bigquery://…`, `mailto:…`) or a
//!   pure `#fragment`; never a concept edge.
//!
//! Frontmatter strings are considered only when their key is a reference rule declared by the
//! concept's ontology type. Markdown body links are always considered.
//!
//! Broken links are **not** errors (spec: consumers MUST tolerate them). Extraction records
//! edges to ids that may not exist; the graph carries them so callers can detect breakage.
use crate::model::concept::{Concept, ConceptId};
use crate::ontology::schema::Ontology;
use pulldown_cmark::{Event, Parser, Tag};
use serde_yaml::Value;
use std::collections::HashSet;

/// The syntactic class of a link target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    /// Leading slash: resolved from the bundle root.
    BundleRelative,
    /// `./` or `../`: resolved against the containing concept's directory.
    Relative,
    /// No leading slash and not dot-relative: a bundle-root id.
    Bare,
    /// A URI (scheme) or a pure `#fragment`: never a concept edge.
    External,
}

/// Classify a raw link string into a [`LinkKind`].
pub fn classify(link: &str) -> LinkKind {
    let s = link.trim();
    if is_external(s) {
        LinkKind::External
    } else if s.starts_with('/') {
        LinkKind::BundleRelative
    } else if s.starts_with("./") || s.starts_with("../") {
        LinkKind::Relative
    } else {
        LinkKind::Bare
    }
}

/// True for URIs (`scheme://…`, `mailto:…`) and pure in-page `#fragment` links — things that
/// are never concept edges.
fn is_external(s: &str) -> bool {
    if s.is_empty() || s.starts_with('#') {
        return true;
    }
    if s.contains("://") {
        return true;
    }
    // scheme: prefix (mailto:, tel:, urn:, bigquery:, …) — alnum/+/-/. up to the first colon,
    // with no slash before it (so `./a:b` is not treated as a scheme).
    if let Some(colon) = s.find(':') {
        let scheme = &s[..colon];
        if !scheme.is_empty()
            && !scheme.contains('/')
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        {
            return true;
        }
    }
    false
}

/// Resolve a raw link (relative to `from`) into a concrete [`ConceptId`].
///
/// Strips any `#fragment`/`?query` and a trailing `.md`, then normalizes `.`/`..` segments.
/// Bundle-relative and bare links resolve from the bundle root; relative links resolve against
/// the directory containing `from`. External links are still normalized as paths (callers
/// filter them out via [`classify`] first).
pub fn resolve_link(from: &ConceptId, link: &str) -> ConceptId {
    let mut s = link.trim();
    if let Some(hash) = s.find('#') {
        s = &s[..hash];
    }
    if let Some(q) = s.find('?') {
        s = &s[..q];
    }
    let s = s.strip_suffix(".md").unwrap_or(s);

    let joined = match classify(link) {
        LinkKind::Relative => {
            let base = parent_dir(from);
            format!("{base}/{s}")
        }
        // Bundle-relative, bare, and (defensively) external all normalize from the root.
        _ => s.to_string(),
    };
    normalize_id(&joined)
}

/// Directory portion of a concept id (`/tables/customers` → `/tables`, `/customers` → ``).
fn parent_dir(id: &ConceptId) -> String {
    match id.0.rfind('/') {
        Some(i) => id.0[..i].to_string(),
        None => String::new(),
    }
}

/// Collapse `.`/`..`/empty segments into a canonical leading-slash id.
fn normalize_id(path: &str) -> ConceptId {
    let path = path.replace('\\', "/");
    let mut out: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    ConceptId(format!("/{}", out.join("/")))
}

/// Pull strings out of a declared frontmatter reference value.
fn collect_from_value(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => {
            if !s.trim().is_empty() {
                out.push(s.clone());
            }
        }
        Value::Sequence(seq) => {
            for item in seq {
                collect_from_value(item, out);
            }
        }
        _ => {}
    }
}

/// Extract every markdown link target (`[text](target)` and autolinks) from a body.
fn collect_from_body(body: &str, out: &mut Vec<String>) {
    let parser = Parser::new(body);
    for event in parser {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            out.push(dest_url.to_string());
        }
    }
}

/// Raw (unresolved) link strings a concept references, from BOTH frontmatter reference fields
/// and markdown links in the body, in a stable order (frontmatter first, then body).
pub fn raw_links(concept: &Concept, ontology: Option<&Ontology>) -> Vec<String> {
    let mut raw = Vec::new();
    if let Some(ct) = concept
        .concept_type()
        .and_then(|name| ontology.and_then(|o| o.concepts.get(name)))
    {
        for key in ct.references.keys() {
            if let Some(value) = concept.frontmatter.get(key) {
                collect_from_value(value, &mut raw);
            }
        }
    }
    collect_from_body(&concept.body, &mut raw);
    raw
}

/// The concept ids a concept links to (outbound edges): resolved, external links dropped,
/// self-links dropped, deduplicated, order-preserving. Targets may not exist in the bundle
/// (broken links are recorded, not rejected).
pub fn outbound_links(concept: &Concept, ontology: Option<&Ontology>) -> Vec<ConceptId> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<ConceptId> = Vec::new();
    for raw in raw_links(concept, ontology) {
        if classify(&raw) == LinkKind::External {
            continue;
        }
        let id = resolve_link(&concept.id, &raw);
        if id == concept.id {
            continue; // drop self-links
        }
        if seen.insert(id.0.clone()) {
            out.push(id);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::frontmatter::Frontmatter;
    use indexmap::IndexMap;

    fn id(s: &str) -> ConceptId {
        ConceptId::from_relative(s)
    }

    #[test]
    fn classifies_kinds() {
        assert_eq!(classify("/tables/customers.md"), LinkKind::BundleRelative);
        assert_eq!(classify("./other.md"), LinkKind::Relative);
        assert_eq!(classify("../metrics/x"), LinkKind::Relative);
        assert_eq!(classify("tables/customers"), LinkKind::Bare);
        assert_eq!(classify("https://example.com"), LinkKind::External);
        assert_eq!(classify("bigquery://p/d/t"), LinkKind::External);
        assert_eq!(classify("mailto:a@b.com"), LinkKind::External);
        assert_eq!(classify("#section"), LinkKind::External);
    }

    #[test]
    fn resolves_bundle_relative() {
        assert_eq!(
            resolve_link(&id("policies/travel"), "/tables/customers.md"),
            id("tables/customers")
        );
    }

    #[test]
    fn resolves_relative_against_containing_dir() {
        assert_eq!(
            resolve_link(&id("tables/customers"), "../metrics/revenue.md"),
            id("metrics/revenue")
        );
        assert_eq!(
            resolve_link(&id("tables/customers"), "./orders"),
            id("tables/orders")
        );
    }

    #[test]
    fn resolves_bare_from_root() {
        assert_eq!(
            resolve_link(&id("policies/travel"), "tables/customers"),
            id("tables/customers")
        );
    }

    #[test]
    fn strips_fragment_and_query() {
        assert_eq!(
            resolve_link(&id("a/b"), "/tables/customers.md#rates"),
            id("tables/customers")
        );
    }

    fn concept(id_str: &str, yaml: &str, body: &str) -> Concept {
        let map: IndexMap<String, Value> = serde_yaml::from_str(yaml).unwrap();
        Concept {
            id: id(id_str),
            frontmatter: Frontmatter::from_map(map),
            body: body.to_string(),
        }
    }

    #[test]
    fn extracts_from_frontmatter_and_body() {
        let ontology = crate::ontology::load::parse_ontology(
            "okf_ontology: '0.1'\nconcepts:\n  Policy:\n    references:\n      computations: {target: X, cardinality: 0..n}\n      inputs: {target: X, cardinality: 0..n}\n"
        ).unwrap();
        let c = concept(
            "policies/travel",
            "type: Policy\ntitle: Travel policy\ncomputations:\n- /computations/mileage\ninputs:\n- ../tables/customers\nresource: bigquery://p/d/t",
            "See [customers](/tables/customers.md) and [ext](https://x.com).",
        );
        let outs: Vec<String> = outbound_links(&c, Some(&ontology))
            .iter()
            .map(|c| c.0.clone())
            .collect();
        assert_eq!(
            outs,
            vec![
                "/computations/mileage".to_string(),
                "/tables/customers".to_string(),
            ],
            "frontmatter refs first, body links next, external+dupes dropped"
        );
    }

    #[test]
    fn ignores_title_tags_and_external_scalars() {
        let c = concept(
            "notes/n",
            "type: Note\ntitle: A B C\ntags:\n- finance\n- core\nstatus: draft",
            "no links here",
        );
        assert!(outbound_links(&c, None).is_empty());
    }

    #[test]
    fn only_declared_reference_fields_become_edges() {
        let ontology = crate::ontology::load::parse_ontology(
            "okf_ontology: '0.1'\nconcepts:\n  Contract:\n    fields:\n      schema: {type: string}\n    references:\n      depends_on: {target: Contract, cardinality: 0..1}\n"
        ).unwrap();
        let c = concept(
            "contracts/a",
            "type: Contract\nschema: contracts/permissions/tables.schema.json\ndepends_on: /contracts/base",
            "",
        );
        let outs = outbound_links(&c, Some(&ontology));
        assert_eq!(outs, vec![id("contracts/base")]);
    }
}
