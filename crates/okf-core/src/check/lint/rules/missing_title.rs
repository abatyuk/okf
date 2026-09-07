//! Missing-title rule: a concept has no non-empty `title`. Advisory (the spec makes `title`
//! optional).
use crate::check::lint::{Finding, RuleContext};

/// Rule identifier.
pub const RULE: &str = "missing-title";

/// One finding per concept lacking a non-empty `title`.
pub fn run(ctx: &RuleContext) -> Vec<Finding> {
    let mut out = Vec::new();
    for concept in &ctx.bundle.concepts {
        if concept.title().map_or(true, |t| t.trim().is_empty()) {
            out.push(Finding {
                rule: RULE.to_string(),
                severity: ctx.config.missing_title,
                concept: Some(concept.id.0.clone()),
                message: "missing `title`".to_string(),
            });
        }
    }
    out
}
