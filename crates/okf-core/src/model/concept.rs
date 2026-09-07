//! Concept and its identity.
use super::frontmatter::Frontmatter;
use super::trust::{derive_trust_tier, TrustTier};

/// A concept's ID is its bundle-relative path without the `.md` suffix, with a leading
/// slash (e.g. `/tables/customers`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptId(pub String);

impl ConceptId {
    /// Build a canonical id from a bundle-relative path fragment (no `.md`), ensuring a
    /// single leading slash and `/` separators.
    pub fn from_relative(rel: &str) -> Self {
        let rel = rel.replace('\\', "/");
        let trimmed = rel.trim_start_matches('/');
        ConceptId(format!("/{trimmed}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ConceptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A single concept: lossless frontmatter plus the markdown body.
#[derive(Debug, Clone)]
pub struct Concept {
    pub id: ConceptId,
    pub frontmatter: Frontmatter,
    pub body: String,
}

impl Concept {
    pub fn concept_type(&self) -> Option<&str> {
        self.frontmatter.get_str("type")
    }

    pub fn title(&self) -> Option<&str> {
        self.frontmatter.get_str("title")
    }

    pub fn description(&self) -> Option<&str> {
        self.frontmatter.get_str("description")
    }

    pub fn status(&self) -> Option<&str> {
        self.frontmatter.get_str("status")
    }

    pub fn tags(&self) -> Vec<String> {
        self.frontmatter.get_tags()
    }

    pub fn trust_tier(&self) -> TrustTier {
        derive_trust_tier(&self.frontmatter)
    }
}
