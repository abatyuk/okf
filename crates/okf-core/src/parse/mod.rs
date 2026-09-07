//! Lossless markdown + YAML parsing and writing.
pub mod markdown;
pub mod writer;
pub mod yaml;

use crate::error::Result;
use crate::model::concept::{Concept, ConceptId};
use crate::model::frontmatter::Frontmatter;

/// Parse raw file content into a [`Concept`] with the given id, preserving frontmatter
/// key order and unknown keys, and the body verbatim.
pub fn parse_concept(id: ConceptId, content: &str) -> Result<Concept> {
    let (fm_text, body) = markdown::split_frontmatter(content);
    let map = match fm_text {
        Some(text) => yaml::parse_frontmatter(&text)?,
        None => Default::default(),
    };
    Ok(Concept {
        id,
        frontmatter: Frontmatter::from_map(map),
        body,
    })
}
