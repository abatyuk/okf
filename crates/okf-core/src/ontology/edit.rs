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
use super::schema::{ConceptType, Field, FieldType, Ontology, ReferenceRule, RelationshipRule};
use serde_yaml::{Mapping, Value};

/// Add a new concept type. Errors if a type with `name` already exists (use
/// [`update_concept_type`] / [`upsert_concept_type`] to modify).
pub fn add_concept_type(ontology: &mut Ontology, name: &str, ct: ConceptType) -> Result<()> {
    declaration_name(&Value::String(name.into()))?;
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

/// Set a complete semantic relationship declaration.
pub fn set_relationship(
    ontology: &mut Ontology,
    concept: &str,
    key: &str,
    rule: RelationshipRule,
) -> Result<()> {
    concept_mut(ontology, concept)?
        .relationships
        .insert(key.to_owned(), rule);
    Ok(())
}

/// Remove an existing semantic relationship declaration.
pub fn remove_relationship(
    ontology: &mut Ontology,
    concept: &str,
    key: &str,
) -> Result<RelationshipRule> {
    concept_mut(ontology, concept)?
        .relationships
        .shift_remove(key)
        .ok_or_else(|| OkfError::Usage(format!("concept {concept:?} has no relationship {key:?}")))
}

/// Overlay a partial concept definition. Named declarations replace completely, omitted
/// declarations remain, and other supplied properties replace their complete value.
pub fn merge_concept_type(existing: &ConceptType, definition: &Value) -> Result<ConceptType> {
    let supplied = mapping(definition, "concept definition")?;
    let mut result = serde_yaml::to_value(existing).map_err(|e| OkfError::Yaml(e.to_string()))?;
    let out = result.as_mapping_mut().unwrap();
    for (key, value) in supplied {
        let name = key
            .as_str()
            .ok_or_else(|| OkfError::Usage("concept keys must be strings".into()))?;
        if matches!(name, "fields" | "references" | "relationships") {
            let declarations = mapping(value, name)?;
            let destination = out
                .entry(key.clone())
                .or_insert_with(|| Value::Mapping(Mapping::new()));
            let destination = destination
                .as_mapping_mut()
                .ok_or_else(|| OkfError::Usage(format!("{name} must be a mapping")))?;
            for (name, declaration) in declarations {
                declaration_name(name)?;
                destination.insert(name.clone(), declaration.clone());
            }
        } else {
            out.insert(key.clone(), value.clone());
        }
    }
    serde_yaml::from_value(result)
        .map_err(|e| OkfError::Usage(format!("invalid concept definition: {e}")))
}

/// Apply a coordinated change document transactionally in memory. No intermediate
/// ontology needs to validate: dependent definitions are checked together at the end.
pub fn apply_changes(ontology: &mut Ontology, changes: &Value) -> Result<()> {
    let plan = mapping(changes, "ontology changes")?;
    check_keys(
        plan,
        &["field_types", "concepts", "remove"],
        "ontology changes",
    )?;
    let mut candidate = ontology.clone();
    let empty = Mapping::new();
    let types = optional_mapping(plan, "field_types")?.unwrap_or(&empty);
    let concepts = optional_mapping(plan, "concepts")?.unwrap_or(&empty);
    if let Some(removals) = optional_mapping(plan, "remove")? {
        check_keys(removals, &["field_types", "concepts"], "remove")?;
        for name in removal_names(removals, "field_types")? {
            reject_conflict(types, &name, "field type")?;
            candidate
                .field_types
                .shift_remove(&name)
                .ok_or_else(|| OkfError::Usage(format!("field type {name:?} does not exist")))?;
        }
        for name in removal_names(removals, "concepts")? {
            reject_conflict(concepts, &name, "concept")?;
            remove_concept_type(&mut candidate, &name)?;
        }
    }
    for (name, definition) in types {
        let name = declaration_name(name)?;
        validate_field_type_name(&name)?;
        mapping(definition, "field-type definition")?;
        let parsed = serde_yaml::from_value(definition.clone())
            .map_err(|e| OkfError::Usage(format!("invalid field-type definition: {e}")))?;
        candidate.field_types.insert(name, parsed);
    }
    for (name, definition) in concepts {
        let name = declaration_name(name)?;
        let mut supplied = mapping(definition, "concept definition")?.clone();
        let removals = supplied.remove(Value::String("remove".into()));
        let existing = candidate.concepts.get(&name).cloned().unwrap_or_default();
        let merged = merge_concept_type(&existing, &Value::Mapping(supplied.clone()))?;
        if merged.attested && name != "Attested Computation" {
            return Err(OkfError::Usage(
                "attested is reserved for the exact OKF type `Attested Computation`".into(),
            ));
        }
        candidate.concepts.insert(name.clone(), merged);
        if let Some(removals) = removals {
            let removals = mapping(&removals, "concept remove")?;
            check_keys(
                removals,
                &["fields", "references", "relationships"],
                "concept remove",
            )?;
            for section in ["fields", "references", "relationships"] {
                for key in removal_names(removals, section)? {
                    if let Some(declarations) = optional_mapping(&supplied, section)? {
                        reject_conflict(declarations, &key, section)?;
                    }
                    match section {
                        "fields" => {
                            remove_field(&mut candidate, &name, &key)?;
                        }
                        "references" => {
                            remove_reference(&mut candidate, &name, &key)?;
                        }
                        _ => {
                            remove_relationship(&mut candidate, &name, &key)?;
                        }
                    }
                }
            }
        }
    }
    validate_ontology(&candidate)?;
    *ontology = candidate;
    Ok(())
}

/// Reusable definitions cannot shadow primitive names, which resolve before definitions.
pub fn validate_field_type_name(name: &str) -> Result<()> {
    declaration_name(&Value::String(name.into()))?;
    if FieldType::from_keyword(name).is_some() {
        return Err(OkfError::Usage(format!(
            "field type {name:?} is a reserved primitive name"
        )));
    }
    Ok(())
}

fn mapping<'a>(value: &'a Value, context: &str) -> Result<&'a Mapping> {
    value
        .as_mapping()
        .ok_or_else(|| OkfError::Usage(format!("{context} must be a mapping")))
}
fn optional_mapping<'a>(map: &'a Mapping, key: &str) -> Result<Option<&'a Mapping>> {
    map.get(Value::String(key.into()))
        .map(|value| mapping(value, key))
        .transpose()
}
fn declaration_name(value: &Value) -> Result<String> {
    value
        .as_str()
        .filter(|name| !name.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| OkfError::Usage("declaration names must be nonempty strings".into()))
}
fn check_keys(map: &Mapping, allowed: &[&str], context: &str) -> Result<()> {
    for key in map.keys() {
        let name = declaration_name(key)?;
        if !allowed.contains(&name.as_str()) {
            return Err(OkfError::Usage(format!("unknown {context} key {name:?}")));
        }
    }
    Ok(())
}
fn reject_conflict(map: &Mapping, name: &str, section: &str) -> Result<()> {
    if map.contains_key(Value::String(name.into())) {
        return Err(OkfError::Usage(format!(
            "conflicting set and remove for {section} {name:?}"
        )));
    }
    Ok(())
}
fn removal_names(map: &Mapping, section: &str) -> Result<Vec<String>> {
    let Some(value) = map.get(Value::String(section.into())) else {
        return Ok(Vec::new());
    };
    let values = value
        .as_sequence()
        .ok_or_else(|| OkfError::Usage(format!("remove.{section} must be a sequence of names")))?;
    let mut names = Vec::new();
    for value in values {
        let name = declaration_name(value)?;
        if names.contains(&name) {
            return Err(OkfError::Usage(format!(
                "duplicate remove.{section} name {name:?}"
            )));
        }
        names.push(name);
    }
    Ok(names)
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
pub fn render_ontology(path: &Path, ontology: &Ontology) -> Result<String> {
    let mut text = to_yaml(ontology)?;
    match std::fs::read_to_string(path) {
        Ok(original) => {
            let restored = preserve_comments(&original, &text);
            // Comment restoration must never alter authored values. YAML permits hashes and
            // apparent mapping keys inside strings, so retain the safe serialization whenever
            // the best-effort layout restoration changes the document's meaning.
            if parse_ontology(&restored).is_ok_and(|parsed| parsed == *ontology) {
                text = restored;
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(OkfError::Environment(format!(
                "cannot read {}: {error}",
                path.display()
            )))
        }
    }
    Ok(text)
}

/// Render, validate, then publish an ontology with one atomic rename.
pub fn save_ontology(path: &Path, ontology: &Ontology) -> Result<()> {
    let text = render_ontology(path, ontology)?;
    use std::io::Write;
    // Stage alongside the destination so publishing uses one atomic rename.
    static NEXT_TEMP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT_TEMP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let destination = if path.is_symlink() {
        std::fs::canonicalize(path)
            .map_err(|e| OkfError::Environment(format!("cannot resolve {}: {e}", path.display())))?
    } else {
        path.to_path_buf()
    };
    let temp = destination.with_file_name(format!(
        ".okf-ontology-{}-{sequence}.tmp",
        std::process::id()
    ));
    let mut created = false;
    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        created = true;
        if let Ok(metadata) = std::fs::metadata(&destination) {
            file.set_permissions(metadata.permissions())?;
        }
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&temp, &destination)
    })();
    if created && result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result.map_err(|e| OkfError::Environment(format!("cannot write {}: {e}", path.display())))
}

/// Reattach comments to the same YAML mapping key after serde's structural rewrite. This keeps
/// rationale comments stable while still validating the typed document before it is written.
fn preserve_comments(original: &str, generated: &str) -> String {
    use std::collections::HashMap;

    // Content below a literal/folded scalar header is data, including lines beginning with
    // '#'. Both extraction and insertion need this guard because serde also emits blocks.
    fn in_block(line: &str, block_indent: &mut Option<usize>) -> bool {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if let Some(level) = *block_indent {
            if trimmed.is_empty() || indent > level {
                return true;
            }
            *block_indent = None;
        }
        let content = comment_start(line).map_or(line, |pos| &line[..pos]);
        if let Some((_, value)) = content.split_once(':') {
            let value = value.trim();
            if value.starts_with(['|', '>'])
                && value[1..]
                    .chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-'))
            {
                *block_indent = Some(indent);
            }
        }
        false
    }

    // A hash in a quoted value is part of that value, not an inline YAML comment.
    fn comment_start(line: &str) -> Option<usize> {
        let mut quote = None;
        let mut escaped = false;
        let mut previous = None;
        for (pos, c) in line.char_indices() {
            if escaped {
                escaped = false;
            } else if quote == Some('"') && c == '\\' {
                escaped = true;
            } else if matches!(c, '\'' | '"') {
                if quote == Some(c) {
                    quote = None;
                } else if quote.is_none() {
                    quote = Some(c);
                }
            } else if c == '#' && quote.is_none() && previous.is_none_or(char::is_whitespace) {
                return Some(pos);
            }
            previous = Some(c);
        }
        None
    }

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
    let mut block_indent = None;
    for line in original.lines() {
        if in_block(line, &mut block_indent) {
            continue;
        }
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
            if let Some(pos) = comment_start(line) {
                inline.insert(path, format!(" {}", &line[pos..]));
            }
        } else {
            pending.clear();
        }
    }

    let mut out = Vec::new();
    let mut stack = Vec::new();
    let mut block_indent = None;
    for line in generated.lines() {
        if in_block(line, &mut block_indent) {
            out.push(line.to_string());
            continue;
        }
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

    #[test]
    fn render_and_save_preserve_hashes_and_keys_inside_scalars() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ontology.yaml");
        let original = "# rationale\nokf_ontology: '0.1'\nconcepts:\n  Policy:\n    description: |\n      # authored description\n      status: retained prose # data\n    local: 'text # still data' # real comment\n    fields:\n      status: {type: string}\n";
        std::fs::write(&path, original).unwrap();
        let mut ontology = parse_ontology(original).unwrap();
        ontology.concepts.get_mut("Policy").unwrap().requires = vec!["title".into()];
        let preview = render_ontology(&path, &ontology).unwrap();
        assert_eq!(parse_ontology(&preview).unwrap(), ontology);
        assert_eq!(preview.matches("# authored description").count(), 1);
        assert!(preview.contains("# rationale"));
        assert!(preview.contains("# real comment"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        save_ontology(&path, &ontology).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), preview);
        save_ontology(&path, &ontology).unwrap();
        assert_eq!(
            parse_ontology(&std::fs::read_to_string(&path).unwrap()).unwrap(),
            ontology
        );
    }

    #[test]
    fn quoted_multiline_scalars_cannot_be_changed_by_comment_restoration() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ontology.yaml");
        let original = "okf_ontology: '0.1'\nconcepts:\n  Policy:\n    description: 'first\n\n      # authored description\n\n      status: text'\n    fields:\n      status: {type: string}\n";
        std::fs::write(&path, original).unwrap();
        let mut ontology = parse_ontology(original).unwrap();
        ontology.concepts.get_mut("Policy").unwrap().requires = vec!["title".into()];
        let preview = render_ontology(&path, &ontology).unwrap();
        assert_eq!(parse_ontology(&preview).unwrap(), ontology);
    }
}
