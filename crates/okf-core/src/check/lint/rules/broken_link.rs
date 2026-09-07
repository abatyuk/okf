//! Broken-link rule: a concept links (via frontmatter refs or body markdown) at an internal
//! target that is not a loaded concept. Broken links are spec-legal, so this is advisory only.
use crate::check::lint::{Finding, RuleContext};

/// Rule identifier.
pub const RULE: &str = "broken-link";

/// One finding per outbound edge whose target does not exist in the bundle.
pub fn run(ctx: &RuleContext) -> Vec<Finding> {
    let mut out = Vec::new();
    for concept in &ctx.bundle.concepts {
        for target in ctx.graph.outbound(&concept.id.0) {
            if !ctx.graph.exists(&target.0) {
                out.push(Finding {
                    rule: RULE.to_string(),
                    severity: ctx.config.broken_link,
                    concept: Some(concept.id.0.clone()),
                    message: format!("links to missing concept {}", target.0),
                });
            }
        }
    }
    out
}
