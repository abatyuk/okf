//! Advisory serialization order and bundle-wide normalization.
use crate::check::lint::{Finding, RuleContext};
use crate::parse::writer::needs_order_normalization;

pub const RULE: &str = "frontmatter-order";

pub fn run(ctx: &RuleContext) -> Vec<Finding> {
    let Some(severity) = ctx.config.frontmatter_order else {
        return Vec::new();
    };
    ctx.bundle.concepts.iter().filter(|concept| {
        needs_order_normalization(concept, ctx.ontology).unwrap_or(false)
    }).map(|concept| Finding {
        rule: RULE.into(), code: None, field_path: None, severity,
        concept: Some(concept.id.0.clone()),
        message: "frontmatter should put type first, ontology fields next, and custom fields last (recursively); run okf lint --fix".into(),
    }).collect()
}

/// Normalize only concepts with ordering differences, retaining verification and body text.
pub fn fix(
    bundle: &crate::bundle::loader::Bundle,
    root: &std::path::Path,
    ontology: Option<&crate::ontology::schema::Ontology>,
) -> crate::error::Result<usize> {
    let mut concepts = Vec::new();
    for concept in &bundle.concepts {
        if needs_order_normalization(concept, ontology)? {
            crate::mutate::edit::render_validated(root, concept)?;
            concepts.push(concept);
        }
    }
    for concept in &concepts {
        crate::mutate::edit::save_concept(root, concept)?;
    }
    Ok(concepts.len())
}
