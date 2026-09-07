//! search + list (list = search with no filter).
use crate::bundle::loader::Bundle;
use crate::model::concept::Concept;

/// Filters for [`search`]. All are optional and combine with AND. `list` is `search` with
/// an all-`None` filter.
#[derive(Debug, Clone, Default)]
pub struct SearchFilter {
    /// Exact match on the concept `type`.
    pub type_: Option<String>,
    /// Membership in the concept's `tags`.
    pub tag: Option<String>,
    /// Case-insensitive substring across id, title, description, and body.
    pub text: Option<String>,
}

impl SearchFilter {
    pub fn is_empty(&self) -> bool {
        self.type_.is_none() && self.tag.is_none() && self.text.is_none()
    }
}

fn matches(concept: &Concept, filter: &SearchFilter) -> bool {
    if let Some(ty) = &filter.type_ {
        if concept.concept_type() != Some(ty.as_str()) {
            return false;
        }
    }
    if let Some(tag) = &filter.tag {
        if !concept.tags().iter().any(|t| t == tag) {
            return false;
        }
    }
    if let Some(text) = &filter.text {
        let needle = text.to_lowercase();
        let hay = format!(
            "{}\n{}\n{}\n{}",
            concept.id.0,
            concept.title().unwrap_or(""),
            concept.description().unwrap_or(""),
            concept.body,
        )
        .to_lowercase();
        if !hay.contains(&needle) {
            return false;
        }
    }
    true
}

/// Return the concepts matching `filter`, in bundle order.
pub fn search<'a>(bundle: &'a Bundle, filter: &SearchFilter) -> Vec<&'a Concept> {
    bundle.concepts.iter().filter(|c| matches(c, filter)).collect()
}

/// List all concepts (search with no filter).
pub fn list(bundle: &Bundle) -> Vec<&Concept> {
    bundle.concepts.iter().collect()
}
