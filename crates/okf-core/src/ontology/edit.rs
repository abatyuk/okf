//! Programmatic add / update / remove of concept types (and their fields and reference
//! rules) — the operations `okf ontology add/update/remove` call.
//!
//! Writes are **self-validating**: [`save_ontology`] serializes, re-parses and validates
//! the result *before* touching disk, so `ontology.yaml` never lands in a broken state.
//!
//! Unknown keys and ordering are preserved via order-preserving maps and flattened `extra`
//! fields. [`save_ontology`] also reattaches leading and inline comments to surviving YAML key
//! paths after typed serialization. Blank-line layout may still normalize.
use std::path::Path;

use crate::error::{OkfError, Result};

use super::load::{parse_ontology, validate_ontology};
use super::schema::{ConceptType, Field, Ontology, ReferenceRule};

/// Add a new concept type. Errors if a type with `name` already exists (use
/// [`update_concept_type`] / [`upsert_concept_type`] to modify).
pub fn add_concept_type(ontology: &mut Ontology, name: &str, ct: ConceptType) -> Result<()> {
    if ontology.concepts.contains_key(name) {
        return Err(OkfError::Usage(format!(
            "concept type {name:?} already exists"
        )));
    }
    ontology.concepts.insert(name.to_string(), ct);
    Ok(())
}

/// Replace an existing concept type. Errors if it does not exist.
pub fn update_concept_type(ontology: &mut Ontology, name: &str, ct: ConceptType) -> Result<()> {
    if !ontology.concepts.contains_key(name) {
        return Err(OkfError::Usage(format!(
            "concept type {name:?} does not exist"
        )));
    }
    ontology.concepts.insert(name.to_string(), ct);
    Ok(())
}

/// Insert or replace a concept type, preserving its position if it already exists.
pub fn upsert_concept_type(ontology: &mut Ontology, name: &str, ct: ConceptType) {
    ontology.concepts.insert(name.to_string(), ct);
}

/// Remove a concept type. Errors if it does not exist.
pub fn remove_concept_type(ontology: &mut Ontology, name: &str) -> Result<ConceptType> {
    // `shift_remove` keeps the order of the remaining entries.
    ontology
        .concepts
        .shift_remove(name)
        .ok_or_else(|| OkfError::Usage(format!("concept type {name:?} does not exist")))
}

/// Set (add or replace) a typed field on a concept type. Errors if the type is unknown.
pub fn set_field(
    ontology: &mut Ontology,
    concept: &str,
    field_key: &str,
    field: Field,
) -> Result<()> {
    let ct = concept_mut(ontology, concept)?;
    ct.fields.insert(field_key.to_string(), field);
    Ok(())
}

/// Remove a field from a concept type. Errors if the type or field is unknown.
pub fn remove_field(ontology: &mut Ontology, concept: &str, field_key: &str) -> Result<Field> {
    let ct = concept_mut(ontology, concept)?;
    ct.fields
        .shift_remove(field_key)
        .ok_or_else(|| OkfError::Usage(format!("concept {concept:?} has no field {field_key:?}")))
}

/// Set (add or replace) a typed reference rule on a concept type.
pub fn set_reference(
    ontology: &mut Ontology,
    concept: &str,
    rule_key: &str,
    rule: ReferenceRule,
) -> Result<()> {
    let ct = concept_mut(ontology, concept)?;
    ct.references.insert(rule_key.to_string(), rule);
    Ok(())
}

/// Remove a reference rule from a concept type.
pub fn remove_reference(
    ontology: &mut Ontology,
    concept: &str,
    rule_key: &str,
) -> Result<ReferenceRule> {
    let ct = concept_mut(ontology, concept)?;
    ct.references.shift_remove(rule_key).ok_or_else(|| {
        OkfError::Usage(format!("concept {concept:?} has no reference {rule_key:?}"))
    })
}

fn concept_mut<'a>(ontology: &'a mut Ontology, name: &str) -> Result<&'a mut ConceptType> {
    ontology
        .concepts
        .get_mut(name)
        .ok_or_else(|| OkfError::Usage(format!("concept type {name:?} does not exist")))
}

/// Serialize an ontology to YAML text, validating it first. Returns `Usage`/`Yaml` if the
/// (edited) ontology is not structurally valid, so callers never persist a broken file.
pub fn to_yaml(ontology: &Ontology) -> Result<String> {
    // Validate the in-memory model, then round-trip through the parser as a belt-and-braces
    // check that what we are about to write parses back cleanly.
    validate_ontology(ontology)?;
    let text = serde_yaml::to_string(ontology).map_err(|e| OkfError::Internal(e.to_string()))?;
    parse_ontology(&text)?;
    Ok(text)
}

/// Validate then write an ontology back to `path`. The file is only touched once the
/// serialized result validates, so a bad edit cannot corrupt an existing file.
pub fn save_ontology(path: &Path, ontology: &Ontology) -> Result<()> {
    let mut text = to_yaml(ontology)?;
    if let Ok(original) = std::fs::read_to_string(path) {
        text = preserve_comments(&original, &text);
        parse_ontology(&text)?;
    }
    std::fs::write(path, text)
        .map_err(|e| OkfError::Environment(format!("cannot write {}: {e}", path.display())))?;
    Ok(())
}

/// Reattach comments to the same YAML mapping key after serde's structural rewrite. This keeps
/// rationale comments stable while still validating the typed document before it is written.
fn preserve_comments(original: &str, generated: &str) -> String {
    use std::collections::HashMap;

    fn key_path(line: &str, stack: &mut Vec<(usize, String)>) -> Option<String> {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
            return None;
        }
        let indent = line.len() - trimmed.len();
        let key = trimmed.split_once(':')?.0.trim().trim_matches(['\'', '"']);
        if key.is_empty() {
            return None;
        }
        while stack.last().is_some_and(|(level, _)| *level >= indent) {
            stack.pop();
        }
        let mut parts: Vec<&str> = stack.iter().map(|(_, k)| k.as_str()).collect();
        parts.push(key);
        let path = parts.join("\u{1f}");
        stack.push((indent, key.to_string()));
        Some(path)
    }

    let mut comments: HashMap<String, Vec<String>> = HashMap::new();
    let mut inline: HashMap<String, String> = HashMap::new();
    let mut pending = Vec::new();
    let mut stack = Vec::new();
    for line in original.lines() {
        if line.trim_start().starts_with('#') {
            pending.push(line.to_string());
            continue;
        }
        if line.trim().is_empty() {
            if !pending.is_empty() {
                pending.push(String::new());
            }
            continue;
        }
        if let Some(path) = key_path(line, &mut stack) {
            if !pending.is_empty() {
                comments.insert(path.clone(), std::mem::take(&mut pending));
            }
            if let Some(pos) = line.find(" #") {
                inline.insert(path, line[pos..].to_string());
            }
        } else {
            pending.clear();
        }
    }

    let mut out = Vec::new();
    let mut stack = Vec::new();
    for line in generated.lines() {
        if let Some(path) = key_path(line, &mut stack) {
            if let Some(block) = comments.remove(&path) {
                out.extend(block);
            }
            if let Some(comment) = inline.get(&path) {
                out.push(format!("{line}{comment}"));
                continue;
            }
        }
        out.push(line.to_string());
    }
    format!("{}\n", out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::load::parse_ontology;

    #[test]
    fn save_preserves_comments_attached_to_existing_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ontology.yaml");
        let original = "# bundle rationale\nokf_ontology: '0.1'\nconcepts:\n  # why policies exist\n  Policy:\n    fields:\n      status: # lifecycle rationale\n        type: enum\n        values: [draft, active]\n";
        std::fs::write(&path, original).unwrap();
        let mut ontology = parse_ontology(original).unwrap();
        ontology.concepts.get_mut("Policy").unwrap().description = Some("Rules".into());
        save_ontology(&path, &ontology).unwrap();
        let after = std::fs::read_to_string(path).unwrap();
        assert!(after.contains("# bundle rationale"), "{after}");
        assert!(after.contains("# why policies exist"), "{after}");
        assert!(after.contains("status: # lifecycle rationale"), "{after}");
    }
}
