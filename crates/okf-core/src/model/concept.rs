//! Concept and its identity.
use super::frontmatter::Frontmatter;
use super::standard::{parse_timestamp, valid_actor};
use super::trust::{derive_trust_tier, TrustTier};
use serde_yaml::Value;

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

    /// Parse a user-supplied concept ID without allowing path traversal or reserved filenames.
    pub fn parse(raw: &str) -> crate::error::Result<Self> {
        use crate::error::OkfError;
        let normalized = raw.trim().replace('\\', "/");
        let normalized = normalized.strip_suffix(".md").unwrap_or(&normalized);
        let rel = normalized.trim_start_matches('/');
        if rel.is_empty() || rel.contains('\0') {
            return Err(OkfError::Usage("concept id must not be empty".to_string()));
        }
        let segments: Vec<&str> = rel.split('/').collect();
        if segments
            .iter()
            .any(|segment| segment.is_empty() || matches!(*segment, "." | ".."))
        {
            return Err(OkfError::Usage(format!(
                "invalid concept id {raw:?}: path traversal and empty segments are not allowed"
            )));
        }
        if segments.first().is_some_and(|segment| {
            segment.len() == 2
                && segment.ends_with(':')
                && segment.as_bytes()[0].is_ascii_alphabetic()
        }) {
            return Err(OkfError::Usage(format!(
                "invalid concept id {raw:?}: filesystem prefixes are not allowed"
            )));
        }
        if matches!(segments.last(), Some(&"index" | &"log")) {
            return Err(OkfError::Usage(format!(
                "invalid concept id {raw:?}: index.md and log.md are reserved"
            )));
        }
        Ok(Self(format!("/{rel}")))
    }
}

impl std::fmt::Display for ConceptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A single concept: semantically preserved frontmatter plus the Markdown body.
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

    /// OKF defines an absent lifecycle status as `stable`.
    pub fn effective_status(&self) -> &str {
        self.status()
            .filter(|status| !status.trim().is_empty())
            .unwrap_or("stable")
    }

    pub fn tags(&self) -> Vec<String> {
        self.frontmatter.get_tags()
    }

    pub fn trust_tier(&self) -> TrustTier {
        derive_trust_tier(&self.frontmatter)
    }

    /// Current content timestamp, with the v0.1 `timestamp` fallback when `generated` is absent.
    pub fn generated_at(&self) -> Option<&str> {
        self.frontmatter
            .get("generated")
            .and_then(|v| v.get("at"))
            .and_then(Value::as_str)
            .or_else(|| {
                self.frontmatter
                    .get("generated")
                    .is_none()
                    .then(|| self.frontmatter.get_str("timestamp"))
                    .flatten()
            })
    }

    pub fn latest_verified_at(&self) -> Option<&str> {
        let verified = self.frontmatter.get("verified")?;
        let entries: Vec<&Value> = match verified {
            Value::Sequence(seq) => seq.iter().collect(),
            Value::Mapping(_) => vec![verified],
            _ => Vec::new(),
        };
        entries
            .into_iter()
            .filter(|entry| {
                entry
                    .get("by")
                    .and_then(Value::as_str)
                    .is_some_and(valid_actor)
            })
            .filter_map(|entry| entry.get("at").and_then(Value::as_str))
            .filter(|at| parse_timestamp(at).is_some())
            .max_by_key(|at| parse_timestamp(at))
    }

    /// Whether the latest valid verification is at least as recent as generated content.
    pub fn verification_current(&self) -> Option<bool> {
        let generated = parse_timestamp(self.generated_at()?)?;
        let verified = parse_timestamp(self.latest_verified_at()?)?;
        Some(verified >= generated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_ids_reject_traversal_and_reserved_names() {
        assert_eq!(
            ConceptId::parse("tables/customers.md").unwrap().0,
            "/tables/customers"
        );
        assert!(ConceptId::parse("../outside").is_err());
        assert!(ConceptId::parse("a//b").is_err());
        assert!(ConceptId::parse("nested/index").is_err());
    }
}
