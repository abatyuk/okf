//! Orphan rule: a concept with neither inbound nor outbound links — disconnected from the
//! rest of the bundle graph. Advisory.
use crate::check::lint::{Finding, RuleContext};

/// Rule identifier.
pub const RULE: &str = "orphan";

/// One finding per concept that has no inbound and no outbound edges.
pub fn run(ctx: &RuleContext) -> Vec<Finding> {
    let mut out = Vec::new();
    for concept in &ctx.bundle.concepts {
        let id = &concept.id.0;
        if ctx.graph.inbound(id).is_empty() && ctx.graph.outbound(id).is_empty() {
            out.push(Finding {
                rule: RULE.to_string(),
                severity: ctx.config.orphan,
                concept: Some(id.clone()),
                message: "orphan: no inbound or outbound links".to_string(),
            });
        }
    }
    out
}
