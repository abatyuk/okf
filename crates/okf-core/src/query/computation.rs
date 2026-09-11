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
    let mut issues = Vec::new();
    if concept.concept_type() != Some("Attested Computation") {
        issues.push("concept type is not exact `Attested Computation`".to_string());
    }
    let runtime = concept
        .frontmatter
        .get_str("runtime")
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string);
    if runtime.is_none() {
        issues.push("missing required runtime".to_string());
    }
    let mut parameters = Vec::new();
    if let Some(value) = concept.frontmatter.get("parameters") {
        if let Some(seq) = value.as_sequence() {
            for (i, parameter) in seq.iter().enumerate() {
                let name = parameter.get("name").and_then(Value::as_str);
                let ty = parameter.get("type").and_then(Value::as_str);
                let required = parameter.get("required").and_then(Value::as_bool);
                match (name, ty, required) {
                    (Some(name), Some(ty), Some(required))
                        if !name.is_empty() && !ty.is_empty() =>
                    {
                        parameters.push((name.to_string(), ty.to_string(), required));
                    }
                    _ => issues.push(format!("parameters[{i}] must contain name, type, required")),
                }
            }
        } else {
            issues.push("parameters must be a list".to_string());
        }
    }
    let computation = concept
        .frontmatter
        .get_str("computation")
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string);
    let inline = has_inline(&concept.body);
    if inline == computation.is_some() {
        issues.push("provide exactly one inline computation fence or computation path".to_string());
    }
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
    Ok(ComputationContract {
        concept,
        runtime,
        parameters,
        computation,
        inline,
        executor_resource,
        receipt,
        attester_resource,
        issues,
    })
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

fn has_inline(body: &str) -> bool {
    let mut under = false;
    for line in body.lines() {
        let t = line.trim();
        if t.starts_with("# ") {
            under = t.trim_start_matches("# ").trim() == "Computation";
        } else if under && (t.starts_with("```") || t.starts_with("~~~")) {
            return true;
        }
    }
    false
}
