//! Semantic serialization back to disk (unknown keys/values and body preserved).
use crate::error::Result;
use crate::model::concept::Concept;

/// Assemble a concept file from already-serialized frontmatter text and a body.
///
/// Layout: `---\n<frontmatter>\n---\n<body>`. `frontmatter` is expected to already end
/// with a newline (as `serialize_frontmatter` produces); if it does not, one is added.
pub fn assemble(frontmatter: &str, body: &str) -> String {
    let mut out = String::with_capacity(frontmatter.len() + body.len() + 8);
    out.push_str("---\n");
    out.push_str(frontmatter);
    if !frontmatter.is_empty() && !frontmatter.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("---\n");
    out.push_str(body);
    out
}

/// Re-serialize a concept while preserving frontmatter meaning/order and the body text.
pub fn write_concept(concept: &Concept) -> Result<String> {
    let fm = super::yaml::serialize_frontmatter(&concept.frontmatter.map)?;
    Ok(assemble(&fm, &concept.body))
}
