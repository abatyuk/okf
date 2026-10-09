//! Inspect an Attested Computation contract without executing it.

use serde_yaml::Value;

use crate::bundle::loader::Bundle;
use crate::error::{OkfError, Result};
use crate::model::concept::Concept;

#[derive(Debug, Clone)]
pub struct ComputationContract<'a> {
    pub concept: &'a Concept,
    pub runtime: Option<String>,
    pub parameters: Vec<(String, String, bool)>,
    pub computation: Option<String>,
    pub inline: bool,
    pub executor_resource: Option<String>,
    pub receipt: Vec<String>,
    pub attester_resource: Option<String>,
    pub issues: Vec<String>,
}

pub fn inspect<'a>(bundle: &'a Bundle, id: &str) -> Result<ComputationContract<'a>> {
    let concept = bundle
        .get(id)
        .ok_or_else(|| OkfError::Usage(format!("concept not found: {id}")))?;
    Ok(inspect_concept(concept))
}

/// Inspect an already-loaded computation concept.
pub fn inspect_concept(concept: &Concept) -> ComputationContract<'_> {
    let mut issues = crate::check::lint::rules::spec_v02::computation_issues(concept);
    if concept.concept_type() != Some("Attested Computation") {
        issues.push("concept type is not exact `Attested Computation`".to_string());
    }
    let runtime = concept
        .frontmatter
        .get_str("runtime")
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string);
    let mut parameters = Vec::new();
    if let Some(value) = concept.frontmatter.get("parameters") {
        if let Some(seq) = value.as_sequence() {
            for parameter in seq {
                let name = parameter.get("name").and_then(Value::as_str);
                let ty = parameter.get("type").and_then(Value::as_str);
                let required = parameter.get("required").and_then(Value::as_bool);
                match (name, ty, required) {
                    (Some(name), Some(ty), Some(required))
                        if !name.is_empty() && !ty.is_empty() =>
                    {
                        parameters.push((name.to_string(), ty.to_string(), required));
                    }
                    _ => {}
                }
            }
        }
    }
    let computation = concept
        .frontmatter
        .get_str("computation")
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string);
    let inline = crate::check::lint::rules::spec_v02::computation_fences(&concept.body).0 > 0;
    let executor_resource = nested_string(concept, "executor", "resource");
    let receipt = concept
        .frontmatter
        .get("executor")
        .and_then(|v| v.get("receipt"))
        .and_then(Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let attester_resource = nested_string(concept, "attester", "resource");
    ComputationContract {
        concept,
        runtime,
        parameters,
        computation,
        inline,
        executor_resource,
        receipt,
        attester_resource,
        issues,
    }
}

fn nested_string(concept: &Concept, family: &str, key: &str) -> Option<String> {
    concept
        .frontmatter
        .get(family)
        .and_then(|v| v.get(key))
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract(yaml: &str, body: &str) -> Concept {
        crate::parse::parse_concept(
            crate::model::concept::ConceptId::from_relative("job"),
            &format!("---\ntype: Attested Computation\nruntime: sql\n{yaml}---\n{body}"),
        )
        .unwrap()
    }

    #[test]
    fn inspection_reuses_lint_contract_checks() {
        let concept = contract("executor: nope\nattester: {}\nparameters:\n- {name: x, type: string, required: true}\n- {name: x, type: string, required: false}\n", "# Computation\n```sql\n```\n```sql\n```\n");
        let issues = inspect_concept(&concept).issues;
        for expected in [
            "executor",
            "attester.resource",
            "duplicate computation parameter",
            "single fenced code block",
            "fence is empty",
        ] {
            assert!(
                issues.iter().any(|issue| issue.contains(expected)),
                "{expected}: {issues:?}"
            );
        }
        let valid = contract("", "# Computation\n```sql\nselect 1;\n```\n");
        assert!(inspect_concept(&valid).issues.is_empty());
    }
}
