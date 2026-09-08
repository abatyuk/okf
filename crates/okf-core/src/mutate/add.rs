//! Add a new concept, scaffolded from the ontology.
//!
//! `add` builds a fresh concept's frontmatter from an ontology concept type: it always writes
//! `type`, `title` and `description`, then fills the type's required OKF built-ins (`requires`),
//! its required custom `fields`, and its required reference keys with empty placeholders. With
//! `--attested` it additionally scaffolds an OKF Attested Computation block (`computation`,
//! `executor`, `attester`). The concept is written via the lossless [`write_concept`] path, so
//! even a scaffold round-trips cleanly. `add` refuses to overwrite an existing file.
//!
//! Placeholders are intentionally empty/typed-neutral (empty string, `false`, `0`, empty list,
//! or empty map). Required enums are omitted so validation can flag them instead of guessing.
use std::path::{Path, PathBuf};

use serde_yaml::{Mapping, Value};

use crate::error::{OkfError, Result};
use crate::model::concept::{Concept, ConceptId};
use crate::model::frontmatter::Frontmatter;
use crate::model::source::Source;
use crate::ontology::field_types::resolve_field;
use crate::ontology::schema::{ConceptType, FieldType, Ontology};

use super::edit::{id_to_path, save_concept};

/// Inputs to `add` beyond the target path.
#[derive(Debug, Clone, Default)]
pub struct AddOptions {
    /// The concept `type` string (an ontology concept-type key). Optional when `attested`.
    pub concept_type: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    /// Scaffold an OKF Attested Computation (`computation`/`executor`/`attester`).
    pub attested: bool,
    /// Custom scalar values supplied at creation.
    pub sets: Vec<(String, Value)>,
    /// Declared references supplied at creation.
    pub references: Vec<(String, String)>,
    /// Structured source entries supplied at creation.
    pub sources: Vec<Source>,
}

/// What `add` created.
#[derive(Debug, Clone)]
pub struct AddResult {
    pub id: ConceptId,
    pub path: PathBuf,
    pub concept_type: String,
    pub attested: bool,
}

/// Add a concept at bundle-relative `rel_path` (with or without a trailing `.md`), scaffolded
/// from `ontology` (may be `None`). Refuses if the target file already exists.
pub fn add(
    root: &Path,
    rel_path: &str,
    ontology: Option<&Ontology>,
    opts: &AddOptions,
) -> Result<AddResult> {
    let stem = rel_path.strip_suffix(".md").unwrap_or(rel_path);
    let id = ConceptId::from_relative(stem);
    let path = id_to_path(root, &id);
    if path.exists() {
        return Err(OkfError::Usage(format!(
            "refusing to add: {} already exists",
            path.display()
        )));
    }

    let concept_type = resolve_type_name(ontology, opts)?;
    let ct: Option<&ConceptType> = ontology.and_then(|o| o.concepts.get(&concept_type));
    let attested = opts.attested || ct.map(|c| c.attested).unwrap_or(false);

    let title = opts.title.clone().unwrap_or_default();
    let description = opts.description.clone().unwrap_or_default();

    let mut map: indexmap::IndexMap<String, Value> = indexmap::IndexMap::new();
    map.insert("type".to_string(), Value::String(concept_type.clone()));
    map.insert("title".to_string(), Value::String(title.clone()));
    map.insert("description".to_string(), Value::String(description));

    if let (Some(ont), Some(ct)) = (ontology, ct) {
        // Required OKF built-ins (`requires:`), skipping ones we already wrote.
        for req in &ct.requires {
            if !map.contains_key(req) {
                map.insert(req.clone(), Value::String(String::new()));
            }
        }
        // Required custom fields, with a type-appropriate placeholder.
        for (key, field) in &ct.fields {
            if field.required && !map.contains_key(key) {
                if let Some(value) = field_placeholder(ont, field) {
                    map.insert(key.clone(), value);
                }
            }
        }
        // Required reference keys (cardinality lower bound ≥ 1).
        for (key, rule) in &ct.references {
            if rule.cardinality.min() >= 1 && !map.contains_key(key) {
                let placeholder = if rule.cardinality.max() == Some(1) {
                    Value::String(String::new())
                } else {
                    Value::Sequence(Vec::new())
                };
                map.insert(key.clone(), placeholder);
            }
        }
    }

    if attested {
        for key in ["computation", "executor", "attester"] {
            map.entry(key.to_string())
                .or_insert_with(|| Value::String(String::new()));
        }
    }

    // Explicit creation-time values replace placeholders without requiring a second command.
    for (key, value) in &opts.sets {
        map.insert(key.clone(), value.clone());
    }
    for (key, link) in &opts.references {
        let rule = ct.and_then(|c| c.references.get(key)).ok_or_else(|| {
            OkfError::Usage(format!(
                "add: --ref key {key:?} is not declared on concept type {concept_type:?}"
            ))
        })?;
        if rule.cardinality.max() == Some(1) {
            if map
                .get(key)
                .and_then(Value::as_str)
                .is_some_and(|s| !s.is_empty())
            {
                return Err(OkfError::Usage(format!(
                    "add: reference {key:?} accepts only one value"
                )));
            }
            map.insert(key.clone(), Value::String(link.clone()));
        } else {
            match map.get_mut(key) {
                Some(Value::Sequence(values)) => values.push(Value::String(link.clone())),
                Some(_) => {
                    map.insert(
                        key.clone(),
                        Value::Sequence(vec![Value::String(link.clone())]),
                    );
                }
                None => {
                    map.insert(
                        key.clone(),
                        Value::Sequence(vec![Value::String(link.clone())]),
                    );
                }
            }
        }
    }
    if !opts.sources.is_empty() {
        map.insert(
            "sources".to_string(),
            Value::Sequence(opts.sources.iter().map(Source::to_value).collect()),
        );
    }

    let heading = if title.is_empty() {
        stem.rsplit('/').next().unwrap_or(stem)
    } else {
        title.as_str()
    };
    let body = format!("# {heading}\n");

    let concept = Concept {
        id: id.clone(),
        frontmatter: Frontmatter::from_map(map),
        body,
    };
    let path = save_concept(root, &concept)?;

    Ok(AddResult {
        id,
        path,
        concept_type,
        attested,
    })
}

/// Decide the concept `type` string. Uses `--type` if given; otherwise, with `--attested`,
/// defaults to the ontology's attested concept type (or `Computation`). Errors if neither a
/// type nor `--attested` is supplied.
fn resolve_type_name(ontology: Option<&Ontology>, opts: &AddOptions) -> Result<String> {
    if let Some(t) = &opts.concept_type {
        if t.trim().is_empty() {
            return Err(OkfError::Usage("add: --type must not be empty".to_string()));
        }
        return Ok(t.clone());
    }
    if opts.attested {
        let name = ontology
            .and_then(|o| {
                o.concepts
                    .iter()
                    .find(|(_, ct)| ct.attested)
                    .map(|(name, _)| name.clone())
            })
            .unwrap_or_else(|| "Computation".to_string());
        return Ok(name);
    }
    Err(OkfError::Usage(
        "add: specify --type <ConceptType> (or --attested)".to_string(),
    ))
}

/// A type-appropriate empty placeholder for a required custom field.
fn field_placeholder(ontology: &Ontology, field: &crate::ontology::schema::Field) -> Option<Value> {
    match resolve_field(ontology, field) {
        Ok(resolved) => match resolved.ty.base {
            FieldType::Bool => Some(Value::Bool(false)),
            FieldType::Int => Some(Value::Number(0.into())),
            FieldType::List => Some(Value::Sequence(Vec::new())),
            FieldType::Object => Some(Value::Mapping(Mapping::new())),
            // An enum has no honest neutral value. Omitting it lets lint identify the missing
            // required field instead of silently claiming the first member.
            FieldType::Enum => None,
            _ => Some(Value::String(String::new())),
        },
        // Unresolvable field type — fall back to an empty string placeholder.
        Err(_) => Some(Value::String(String::new())),
    }
}
