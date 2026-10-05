//! Missing-description rule: a concept has no non-empty `description`. Advisory (the spec
//! makes `description` optional).
use crate::check::lint::{Finding, RuleContext};

/// Rule identifier.
pub const RULE: &str = "missing-description";

/// One finding per concept lacking a non-empty `description`.
pub fn run(ctx: &RuleContext) -> Vec<Finding> {
    let Some(severity) = ctx.config.missing_description else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for concept in &ctx.bundle.concepts {
        if concept.description().is_none_or(|d| d.trim().is_empty()) {
            out.push(Finding {
                code: None,
                field_path: None,
                rule: RULE.to_string(),
                severity,
                concept: Some(concept.id.0.clone()),
                message: "missing `description`".to_string(),
            });
        }
    }
    out
}
