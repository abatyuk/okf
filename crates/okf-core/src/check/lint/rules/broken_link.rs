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
            let explicit_nested = ctx
                .ontology
                .and_then(|o| concept.concept_type().and_then(|t| o.concepts.get(t)))
                .is_some_and(|ct| {
                    ct.references
                        .iter()
                        .filter_map(|(_, r)| r.selector.as_ref())
                        .any(|raw| {
                            let root =
                                serde_yaml::to_value(&concept.frontmatter.map).unwrap_or_default();
                            crate::query::selector::Selector::parse(raw)
                                .ok()
                                .is_some_and(|s| {
                                    s.select(&root)
                                        .leaves
                                        .iter()
                                        .filter_map(|o| o.value.as_str())
                                        .any(|raw| {
                                            crate::model::link::resolve_link(&concept.id, raw)
                                                == *target
                                        })
                                })
                        })
                });
            if !ctx.graph.exists(&target.0) && !explicit_nested {
                out.push(Finding {
                    code: None,
                    field_path: None,
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
