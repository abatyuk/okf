//! Individual lint rules. Each submodule exposes `run(&RuleContext) -> Vec<Finding>` and
//! carries its own default severity (overridable via [`super::LintConfig`]).
pub mod broken_link;
pub mod missing_description;
pub mod missing_title;
pub mod ontology_violation;
pub mod orphan;
