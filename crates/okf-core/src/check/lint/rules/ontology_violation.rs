//! Ontology-violation rule: check each concept against its ontology-declared type (unknown
//! type, missing required built-in/custom fields, reference cardinality, wrong reference
//! target). Skipped entirely when the bundle has no `ontology.yaml`.
//!
//! Reference-target checks need the *type* of each linked concept, which lives in the graph /
//! bundle. We supply that to [`ontology::check_concept`] as a `resolve_link_type` closure:
//! resolve the raw link against the source concept, look the target up in the bundle, return
//! its `type`. Unresolvable links yield `None` and are skipped (keeping the check permissive).
//!
//! [`ontology::check_concept`]: crate::ontology::field_types::check_concept
use crate::check::lint::{Finding, RuleContext};
use crate::model::link::resolve_link;
use crate::ontology::field_types::check_concept_with_budget;

/// Rule identifier.
pub const RULE: &str = "ontology-violation";

/// One finding per ontology violation across all concepts. Empty when no ontology is present.
pub fn run(ctx: &RuleContext) -> Vec<Finding> {
    let Some(severity) = ctx.config.ontology_violation else {
        return Vec::new();
    };
    let Some(ontology) = ctx.ontology else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for concept in &ctx.bundle.concepts {
        // Resolve a raw link (relative to this concept) to its target's `type`.
        let resolve_link_type = |link: &str| -> Option<String> {
            let target = resolve_link(&concept.id, link);
            ctx.bundle
                .get(&target.0)
                .and_then(|c| c.concept_type().map(str::to_string))
        };

        for violation in check_concept_with_budget(
            ontology,
            &concept.frontmatter,
            resolve_link_type,
            ctx.config.finding_budget,
        ) {
            out.push(Finding {
                code: match &violation.kind {
                    crate::ontology::field_types::ViolationKind::Metadata { code, .. } => {
                        Some(code.clone())
                    }
                    _ => None,
                },
                field_path: match &violation.kind {
                    crate::ontology::field_types::ViolationKind::Metadata { path, .. } => {
                        Some(path.clone())
                    }
                    _ => None,
                },
                rule: RULE.to_string(),
                severity,
                concept: Some(concept.id.0.clone()),
                message: violation.message,
            });
        }
    }
    out
}
