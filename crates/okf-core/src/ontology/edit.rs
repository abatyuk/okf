//! Programmatic add / update / remove of concept types (and their fields and reference
//! rules) — the operations `okf ontology add/update/remove` call.
//!
//! Writes are **self-validating**: [`save_ontology`] serializes, re-parses and validates
//! the result *before* touching disk, so `ontology.yaml` never lands in a broken state.
//!
//! **Losslessness limits (v1):** unknown top-level keys, unknown per-concept/field/rule
//! keys, and concept/field ordering are preserved via order-preserving maps and flattened
//! `extra` fields. **Comments and blank-line layout are NOT preserved** — `serde_yaml`
//! drops them on re-serialize. This is the accepted ARCHITECTURE.md decision (a) for v1;
//! a CST-based surgical editor is a possible later upgrade.
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
    ct.references
        .shift_remove(rule_key)
        .ok_or_else(|| OkfError::Usage(format!("concept {concept:?} has no reference {rule_key:?}")))
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
    let text = to_yaml(ontology)?;
    std::fs::write(path, text)
        .map_err(|e| OkfError::Environment(format!("cannot write {}: {e}", path.display())))?;
    Ok(())
}
