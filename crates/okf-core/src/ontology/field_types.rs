//! `field_types` composition: `extends` merging (most-derived wins), `object` nesting,
//! and `extends`/nesting cycle rejection. Also the ontology-conformance check that
//! `lint` will consume.
//!
//! The resolver expands a field's *declared* type name into a concrete [`ResolvedType`]:
//! a primitive base kind plus merged constraints, enum values, list element and object
//! sub-fields.
use indexmap::IndexMap;
use serde_yaml::Value;

use crate::error::{OkfError, Result};
use crate::model::frontmatter::Frontmatter;

use super::schema::{Cardinality, ConceptType, Field, FieldType, Ontology, ReferenceRule};

/// A field type expanded to a concrete shape. Constraints from an `extends` chain are
/// merged with the most-derived definition winning on conflict.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedType {
    /// The primitive base kind.
    pub base: FieldType,
    /// Enum values (empty unless `base == Enum`).
    pub values: Vec<String>,
    /// List element type (present only when `base == List`).
    pub item: Option<Box<ResolvedType>>,
    /// Object sub-fields (present only when `base == Object`).
    pub fields: IndexMap<String, ResolvedField>,
    /// Merged constraints (`pattern`, `min`, `max`, …).
    pub constraints: IndexMap<String, Value>,
}

/// A resolved field: its `required` flag plus its concrete type.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedField {
    pub required: bool,
    pub ty: ResolvedType,
}

/// Resolve a single concept [`Field`] to a [`ResolvedField`], following any `field_types`
/// reference, `extends` chain and `object` nesting. Rejects cycles.
pub fn resolve_field(ontology: &Ontology, field: &Field) -> Result<ResolvedField> {
    let mut chain: Vec<String> = Vec::new();
    let mut ty = resolve_type_name(ontology, &field.type_name, &mut chain)?;
    overlay_field(ontology, field, &mut ty)?;
    Ok(ResolvedField {
        required: field.required,
        ty,
    })
}

/// Resolve a type *name* — a primitive keyword or a `field_types` entry — into a concrete
/// [`ResolvedType`]. `chain` tracks the in-progress `field_types` names to reject cycles.
pub fn resolve_type_name(
    ontology: &Ontology,
    name: &str,
    chain: &mut Vec<String>,
) -> Result<ResolvedType> {
    if let Some(base) = FieldType::from_keyword(name) {
        return Ok(ResolvedType {
            base,
            values: Vec::new(),
            item: None,
            fields: IndexMap::new(),
            constraints: IndexMap::new(),
        });
    }

    // Must be a field_types entry.
    let def = ontology.field_types.get(name).ok_or_else(|| {
        OkfError::Usage(format!(
            "unknown field type {name:?}: not a primitive and not defined under field_types"
        ))
    })?;

    if chain.iter().any(|n| n == name) {
        chain.push(name.to_string());
        return Err(OkfError::Usage(format!(
            "field_types cycle detected: {}",
            chain.join(" -> ")
        )));
    }
    chain.push(name.to_string());

    // Start from the parent (via `extends`) or an empty base.
    let mut resolved = match (&def.extends, &def.base) {
        (Some(parent), _) => resolve_type_name(ontology, parent, chain)?,
        (None, Some(base_name)) => resolve_type_name(ontology, base_name, chain)?,
        (None, None) => {
            chain.pop();
            return Err(OkfError::Usage(format!(
                "field_types entry {name:?} must set `base` or `extends`"
            )));
        }
    };

    // A `base` given alongside `extends` overrides the inherited base kind.
    if def.extends.is_some() {
        if let Some(base_name) = &def.base {
            if let Some(base) = FieldType::from_keyword(base_name) {
                resolved.base = base;
            }
        }
    }

    // Merge constraints: most-derived (this def) wins.
    for (k, v) in &def.constraints {
        resolved.constraints.insert(k.clone(), v.clone());
    }
    if let Some(values) = &def.values {
        resolved.values = values.clone();
    }
    if let Some(item_name) = &def.item {
        let item_ty = resolve_type_name(ontology, item_name, chain)?;
        resolved.item = Some(Box::new(item_ty));
    }
    if let Some(fields) = &def.fields {
        resolved.fields = resolve_subfields(ontology, fields, chain)?;
    }

    chain.pop();
    Ok(resolved)
}

/// Overlay a concept field's inline structure (its own `values` / `item` / `fields` /
/// `constraints`) onto the type resolved from its name. Field-level entries win.
fn overlay_field(ontology: &Ontology, field: &Field, ty: &mut ResolvedType) -> Result<()> {
    for (k, v) in &field.constraints {
        ty.constraints.insert(k.clone(), v.clone());
    }
    if let Some(values) = &field.values {
        ty.values = values.clone();
    }
    if let Some(item_name) = &field.item {
        let mut chain = Vec::new();
        ty.item = Some(Box::new(resolve_type_name(ontology, item_name, &mut chain)?));
    }
    if let Some(fields) = &field.fields {
        let mut chain = Vec::new();
        ty.fields = resolve_subfields(ontology, fields, &mut chain)?;
    }
    Ok(())
}

fn resolve_subfields(
    ontology: &Ontology,
    fields: &IndexMap<String, Field>,
    chain: &mut Vec<String>,
) -> Result<IndexMap<String, ResolvedField>> {
    let mut out = IndexMap::new();
    for (name, f) in fields {
        let mut ty = resolve_type_name(ontology, &f.type_name, chain)?;
        // Overlay inline structure using the shared chain so nesting cycles are caught.
        for (k, v) in &f.constraints {
            ty.constraints.insert(k.clone(), v.clone());
        }
        if let Some(values) = &f.values {
            ty.values = values.clone();
        }
        if let Some(item_name) = &f.item {
            ty.item = Some(Box::new(resolve_type_name(ontology, item_name, chain)?));
        }
        if let Some(nested) = &f.fields {
            ty.fields = resolve_subfields(ontology, nested, chain)?;
        }
        out.insert(
            name.clone(),
            ResolvedField {
                required: f.required,
                ty,
            },
        );
    }
    Ok(out)
}

/// Resolve every field of every concept type, surfacing the first structural error
/// (unknown type name, `extends`/nesting cycle, or a `field_types` entry missing both
/// `base` and `extends`). Used by `load` and `edit` to reject a broken ontology.
pub fn validate_field_types(ontology: &Ontology) -> Result<()> {
    // Resolve standalone field_types entries so unreferenced-but-broken defs are caught too.
    for name in ontology.field_types.keys() {
        let mut chain = Vec::new();
        resolve_type_name(ontology, name, &mut chain)?;
    }
    for (ct_name, ct) in &ontology.concepts {
        for (fname, field) in &ct.fields {
            resolve_field(ontology, field).map_err(|e| {
                OkfError::Usage(format!("concept {ct_name:?} field {fname:?}: {e}"))
            })?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Ontology-conformance check (consumed later by `check::lint`).
// ---------------------------------------------------------------------------

/// A single advisory ontology violation found on a concept.
#[derive(Debug, Clone, PartialEq)]
pub struct OntologyViolation {
    pub kind: ViolationKind,
    pub message: String,
}

/// The category of an [`OntologyViolation`].
#[derive(Debug, Clone, PartialEq)]
pub enum ViolationKind {
    /// The concept's `type` is not defined in the ontology (permissive — advisory only).
    UnknownType,
    /// A `requires:` built-in field is absent.
    MissingRequiredBuiltin(String),
    /// A `required: true` custom field is absent.
    MissingRequiredField(String),
    /// A reference rule's link count violates its cardinality.
    CardinalityViolation(String),
    /// A referenced concept has a type outside the rule's allowed target(s).
    WrongReferenceTarget(String),
}

/// Check a concept's `frontmatter` against its ontology-declared type.
///
/// This is the core logic `check::lint` will drive; it does no I/O and prints nothing.
/// Reference-target checking needs to know the *type* of each linked concept, which lives
/// in the graph/query layer, so the caller supplies `resolve_link_type`: given a link
/// string it returns that concept's `type` (or `None` if it can't be resolved — such
/// links are skipped, not flagged, keeping the check advisory and permissive).
pub fn check_concept<F>(
    ontology: &Ontology,
    frontmatter: &Frontmatter,
    resolve_link_type: F,
) -> Vec<OntologyViolation>
where
    F: Fn(&str) -> Option<String>,
{
    let mut out = Vec::new();

    let ty = match frontmatter.get_str("type") {
        Some(t) => t,
        None => return out, // no type: `validate` covers this, not the ontology check
    };
    let ct: &ConceptType = match ontology.concepts.get(ty) {
        Some(c) => c,
        None => {
            out.push(OntologyViolation {
                kind: ViolationKind::UnknownType,
                message: format!("type {ty:?} is not defined in the ontology"),
            });
            return out;
        }
    };

    // requires: built-in fields must be present and non-empty.
    for req in &ct.requires {
        if !has_value(frontmatter, req) {
            out.push(OntologyViolation {
                kind: ViolationKind::MissingRequiredBuiltin(req.clone()),
                message: format!("{ty}: required built-in field {req:?} is missing"),
            });
        }
    }

    // required custom fields must be present.
    for (fname, field) in &ct.fields {
        if field.required && !has_value(frontmatter, fname) {
            out.push(OntologyViolation {
                kind: ViolationKind::MissingRequiredField(fname.clone()),
                message: format!("{ty}: required field {fname:?} is missing"),
            });
        }
    }

    // reference rules: cardinality + target-type checks.
    for (key, rule) in &ct.references {
        check_reference(ty, key, rule, frontmatter, &resolve_link_type, &mut out);
    }

    out
}

fn check_reference<F>(
    ty: &str,
    key: &str,
    rule: &ReferenceRule,
    frontmatter: &Frontmatter,
    resolve_link_type: &F,
    out: &mut Vec<OntologyViolation>,
) where
    F: Fn(&str) -> Option<String>,
{
    let links = links_of(frontmatter, key);
    let card: Cardinality = rule.cardinality;
    if !card.permits(links.len()) {
        out.push(OntologyViolation {
            kind: ViolationKind::CardinalityViolation(key.to_string()),
            message: format!(
                "{ty}: reference {key:?} has {} link(s) but cardinality is {}",
                links.len(),
                card
            ),
        });
    }
    for link in &links {
        if let Some(target_ty) = resolve_link_type(link) {
            if !rule.target.allows(&target_ty) {
                out.push(OntologyViolation {
                    kind: ViolationKind::WrongReferenceTarget(key.to_string()),
                    message: format!(
                        "{ty}: reference {key:?} -> {link:?} targets type {target_ty:?}, \
                         but the rule allows {:?}",
                        rule.target.types()
                    ),
                });
            }
        }
    }
}

/// True if the frontmatter carries a non-null, non-empty value for `key`.
fn has_value(frontmatter: &Frontmatter, key: &str) -> bool {
    match frontmatter.get(key) {
        None | Some(Value::Null) => false,
        Some(Value::String(s)) => !s.trim().is_empty(),
        Some(Value::Sequence(seq)) => !seq.is_empty(),
        Some(_) => true,
    }
}

/// Extract the links held under a frontmatter `key` as flat strings (a scalar string, or a
/// sequence of strings — non-string items are ignored).
fn links_of(frontmatter: &Frontmatter, key: &str) -> Vec<String> {
    match frontmatter.get(key) {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Sequence(seq)) => seq
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}
