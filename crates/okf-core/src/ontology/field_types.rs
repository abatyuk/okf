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
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
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
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
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
            let base = resolve_type_name(ontology, base_name, chain)?.base;
            if resolved.base != base {
                return Err(OkfError::Usage(format!(
                    "field type {name:?} changes inherited base"
                )));
            }
        }
    }

    check_declared_shape(
        resolved.base,
        def.values.is_some(),
        def.item.is_some(),
        def.fields.is_some(),
    )?;
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
    check_declared_shape(
        ty.base,
        field.values.is_some(),
        field.item.is_some(),
        field.fields.is_some(),
    )?;
    for (k, v) in &field.constraints {
        ty.constraints.insert(k.clone(), v.clone());
    }
    if let Some(values) = &field.values {
        ty.values = values.clone();
    }
    if let Some(item_name) = &field.item {
        let mut chain = Vec::new();
        ty.item = Some(Box::new(resolve_type_name(
            ontology, item_name, &mut chain,
        )?));
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
        check_declared_shape(
            ty.base,
            f.values.is_some(),
            f.item.is_some(),
            f.fields.is_some(),
        )?;
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
        validate_effective(&resolve_type_name(ontology, name, &mut chain)?)?;
    }
    for (ct_name, ct) in &ontology.concepts {
        for (fname, field) in &ct.fields {
            let resolved = resolve_field(ontology, field).map_err(|e| {
                OkfError::Usage(format!("concept {ct_name:?} field {fname:?}: {e}"))
            })?;
            validate_effective(&resolved.ty)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Ontology-conformance check (consumed later by `check::lint`).
// ---------------------------------------------------------------------------

/// A single advisory ontology violation found on a concept.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct OntologyViolation {
    pub kind: ViolationKind,
    pub message: String,
}

/// The category of an [`OntologyViolation`].
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
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
    Metadata {
        code: String,
        path: String,
    },
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
    check_concept_with_budget(ontology, frontmatter, resolve_link_type, FINDING_BUDGET)
}
pub fn check_concept_with_budget<F>(
    ontology: &Ontology,
    frontmatter: &Frontmatter,
    resolve_link_type: F,
    budget: usize,
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
            push_violation(
                &mut out,
                budget,
                OntologyViolation {
                    kind: ViolationKind::UnknownType,
                    message: format!("type {ty:?} is not defined in the ontology"),
                },
            );
            return out;
        }
    };

    // requires: built-in fields must be present and non-empty.
    for req in &ct.requires {
        if !has_value(frontmatter, req) {
            push_violation(
                &mut out,
                budget,
                OntologyViolation {
                    kind: ViolationKind::MissingRequiredBuiltin(req.clone()),
                    message: format!("{ty}: required built-in field {req:?} is missing"),
                },
            );
        }
    }

    for (fname, field) in &ct.fields {
        if let Ok(resolved) = resolve_field(ontology, field) {
            check_value(
                &resolved,
                frontmatter.get(fname),
                fname,
                0,
                &mut out,
                budget,
            );
        }
    }

    // reference rules: cardinality + target-type checks.
    for (key, rule) in &ct.references {
        check_reference(
            ontology,
            key,
            rule,
            frontmatter,
            &resolve_link_type,
            &mut out,
            budget,
        );
    }

    let concept = crate::model::concept::Concept {
        id: crate::model::concept::ConceptId("/context".into()),
        frontmatter: frontmatter.clone(),
        body: String::new(),
    };
    for edge in crate::graph::relationships::relationships(&concept, ontology) {
        if edge.status == "unknown-kind" {
            metadata(
                &mut out,
                budget,
                "metadata-relationship-kind",
                &edge.field_path,
                format!(
                    "unknown or absent relationship kind {:?}",
                    edge.authored_kind
                ),
            );
        }
        if let Some(targets) = ct
            .relationships
            .get(&edge.rule)
            .and_then(|r| edge.authored_kind.as_ref().and_then(|k| r.kinds.get(k)))
            .and_then(|k| k.target.as_ref())
        {
            if let Some(actual) = resolve_link_type(&edge.raw_reference) {
                if !targets
                    .types()
                    .iter()
                    .any(|expected| concept_is_a(ontology, &actual, expected))
                {
                    metadata(
                        &mut out,
                        budget,
                        "metadata-reference-target",
                        &edge.field_path,
                        format!(
                            "{}: target type {actual:?} not allowed by relationship kind",
                            edge.field_path
                        ),
                    );
                }
            }
        }
    }
    out
}

fn check_reference<F>(
    ontology: &Ontology,
    key: &str,
    rule: &ReferenceRule,
    frontmatter: &Frontmatter,
    resolve_link_type: &F,
    out: &mut Vec<OntologyViolation>,
    budget: usize,
) where
    F: Fn(&str) -> Option<String>,
{
    let ty = frontmatter.get_str("type").unwrap_or("");
    let links = if let Some(selector) = &rule.selector {
        let root = serde_yaml::to_value(&frontmatter.map).unwrap_or_default();
        crate::query::selector::Selector::parse(selector)
            .map(|s| {
                s.select(&root)
                    .leaves
                    .into_iter()
                    .filter_map(|o| {
                        o.value
                            .as_str()
                            .filter(|s| !s.trim().is_empty())
                            .map(str::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        links_of(frontmatter, key)
    };
    let card: Cardinality = rule.cardinality;
    if !card.permits(links.len()) {
        push_violation(
            out,
            budget,
            OntologyViolation {
                kind: ViolationKind::CardinalityViolation(key.to_string()),
                message: format!(
                    "{ty}: reference {key:?} has {} link(s) but cardinality is {}",
                    links.len(),
                    card
                ),
            },
        );
    }
    if let Some(selector) = &rule.selector {
        let root = serde_yaml::to_value(&frontmatter.map).unwrap_or_default();
        if let Ok(selector) = crate::query::selector::Selector::parse(selector) {
            for o in selector.select(&root).leaves {
                if !o.value.as_str().is_some_and(|s| !s.trim().is_empty()) {
                    metadata(
                        out,
                        budget,
                        "metadata-reference-type",
                        &o.path,
                        format!("{}: reference target must be a nonempty string", o.path),
                    );
                } else if crate::model::link::classify(o.value.as_str().unwrap())
                    != crate::model::link::LinkKind::External
                    && resolve_link_type(o.value.as_str().unwrap()).is_none()
                {
                    metadata(
                        out,
                        budget,
                        "metadata-reference-missing",
                        &o.path,
                        format!(
                            "{}: missing reference target {:?}",
                            o.path,
                            o.value.as_str().unwrap()
                        ),
                    );
                }
            }
        }
    }
    for link in &links {
        if let Some(target_ty) = resolve_link_type(link) {
            if !rule
                .target
                .types()
                .iter()
                .any(|expected| concept_is_a(ontology, &target_ty, expected))
            {
                push_violation(
                    out,
                    budget,
                    OntologyViolation {
                        kind: ViolationKind::WrongReferenceTarget(key.to_string()),
                        message: format!(
                            "{ty}: reference {key:?} -> {link:?} targets type {target_ty:?}, \
                         but the rule allows {:?}",
                            rule.target.types()
                        ),
                    },
                );
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

/// Reject malformed recognized constraints after applying all inheritance and overlays.
pub fn validate_effective(ty: &ResolvedType) -> Result<()> {
    let bad = |s: &str| OkfError::Usage(format!("invalid {} schema: {s}", ty.base));
    if ty.base == FieldType::Enum && ty.values.is_empty() {
        return Err(bad("enum requires nonempty values"));
    }
    if ty.base != FieldType::Enum && !ty.values.is_empty() {
        return Err(bad("values requires enum"));
    }
    if ty.base == FieldType::List && ty.item.is_none() {
        return Err(bad("list requires item type"));
    }
    if ty.base != FieldType::List && ty.item.is_some() {
        return Err(bad("item requires list"));
    }
    if ty.base != FieldType::Object && !ty.fields.is_empty() {
        return Err(bad("fields requires object"));
    }
    for (key, value) in &ty.constraints {
        match key.as_str() {
            "min" | "max" => {
                if !matches!(
                    ty.base,
                    FieldType::Int | FieldType::String | FieldType::Text | FieldType::List
                ) {
                    return Err(bad("bounds require int, string, text, or list"));
                }
                let n = value
                    .as_i64()
                    .ok_or_else(|| bad("bounds must be integers"))?;
                if ty.base != FieldType::Int && n < 0 {
                    return Err(bad("length bounds must be nonnegative"));
                }
            }
            "pattern" => {
                if !matches!(
                    ty.base,
                    FieldType::String
                        | FieldType::Text
                        | FieldType::Uri
                        | FieldType::Date
                        | FieldType::Datetime
                        | FieldType::Enum
                ) {
                    return Err(bad("pattern requires string-valued type"));
                }
                let pattern = value
                    .as_str()
                    .ok_or_else(|| bad("pattern must be a string"))?;
                regex::Regex::new(pattern).map_err(|e| bad(&format!("invalid pattern: {e}")))?;
            }
            "unique_items" => {
                if ty.base != FieldType::List || value.as_bool().is_none() {
                    return Err(bad("unique_items requires list and boolean"));
                }
            }
            "additional_properties"
                if ty.base != FieldType::Object || value.as_bool().is_none() =>
            {
                return Err(bad("additional_properties requires object and boolean"));
            }
            _ => {}
        }
    }
    if let (Some(min), Some(max)) = (
        ty.constraints.get("min").and_then(Value::as_i64),
        ty.constraints.get("max").and_then(Value::as_i64),
    ) {
        if min > max {
            return Err(bad("min exceeds max"));
        }
    }
    if let Some(item) = &ty.item {
        validate_effective(item)?;
    }
    for field in ty.fields.values() {
        validate_effective(&field.ty)?;
    }
    Ok(())
}

pub const FINDING_BUDGET: usize = 1000;
fn metadata(
    out: &mut Vec<OntologyViolation>,
    budget: usize,
    code: &str,
    path: &str,
    message: String,
) {
    if out.len() < budget {
        out.push(OntologyViolation {
            kind: ViolationKind::Metadata {
                code: code.into(),
                path: path.into(),
            },
            message,
        });
    } else if !out
        .iter()
        .any(|v| matches!(&v.kind,ViolationKind::Metadata{code,..} if code=="incomplete-check"))
    {
        out.push(OntologyViolation {
            kind: ViolationKind::Metadata {
                code: "incomplete-check".into(),
                path: path.into(),
            },
            message: "ontology finding budget exhausted; check incomplete".into(),
        });
    }
}
fn nonempty(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::String(s)) => !s.trim().is_empty(),
        Some(Value::Sequence(s)) => !s.is_empty(),
        _ => true,
    }
}
fn check_value(
    field: &ResolvedField,
    value: Option<&Value>,
    path: &str,
    depth: usize,
    out: &mut Vec<OntologyViolation>,
    budget: usize,
) {
    if depth > 64 {
        metadata(
            out,
            budget,
            "incomplete-check",
            path,
            "ontology depth limit 64 exhausted; check incomplete".into(),
        );
        return;
    }
    if out.len() > budget {
        return;
    }
    if field.required && !nonempty(value) {
        metadata(
            out,
            budget,
            "metadata-required",
            path,
            format!("{path}: required value is absent or empty"),
        );
        return;
    }
    let Some(value) = value else {
        return;
    };
    let ty = &field.ty;
    let valid = match ty.base {
        FieldType::String | FieldType::Text | FieldType::Enum => value.is_string(),
        FieldType::Int => value.as_i64().is_some(),
        FieldType::Bool => value.is_bool(),
        FieldType::Date => value.as_str().is_some_and(valid_date),
        FieldType::Datetime => value.as_str().is_some_and(valid_datetime),
        FieldType::Uri => value.as_str().is_some_and(valid_uri),
        FieldType::List => value.is_sequence(),
        FieldType::Object => value
            .as_mapping()
            .is_some_and(|m| m.keys().all(Value::is_string)),
    };
    if !valid {
        metadata(
            out,
            budget,
            "metadata-type",
            path,
            format!("{path}: expected {}; found {}", ty.base, bounded(value)),
        );
        return;
    }
    for key in ty.constraints.keys().filter(|k| {
        !matches!(
            k.as_str(),
            "min" | "max" | "pattern" | "unique_items" | "additional_properties"
        )
    }) {
        metadata(
            out,
            budget,
            "metadata-unenforced",
            path,
            format!("{path}: unsupported constraint {key:?} is unenforced"),
        );
    }
    if ty.base == FieldType::Enum && !ty.values.iter().any(|s| Some(s.as_str()) == value.as_str()) {
        metadata(
            out,
            budget,
            "metadata-enum",
            path,
            format!(
                "{path}: expected one of {:?}; found {}",
                ty.values,
                bounded(value)
            ),
        );
    }
    if let Some(pattern) = ty.constraints.get("pattern").and_then(Value::as_str) {
        if let Ok(regex) = regex::Regex::new(pattern) {
            if !regex.is_match(value.as_str().unwrap_or_default()) {
                metadata(
                    out,
                    budget,
                    "metadata-pattern",
                    path,
                    format!(
                        "{path}: expected pattern {pattern:?}; found {}",
                        bounded(value)
                    ),
                );
            }
        }
    }
    let measure = match ty.base {
        FieldType::Int => value.as_i64(),
        FieldType::String | FieldType::Text => value.as_str().map(|s| s.chars().count() as i64),
        FieldType::List => value.as_sequence().map(|s| s.len() as i64),
        _ => None,
    };
    if let Some(actual) = measure {
        for key in ["min", "max"] {
            if let Some(bound) = ty.constraints.get(key).and_then(Value::as_i64) {
                if (key == "min" && actual < bound) || (key == "max" && actual > bound) {
                    metadata(
                        out,
                        budget,
                        "metadata-range",
                        path,
                        format!("{path}: expected {key} {bound}; found {actual}"),
                    );
                }
            }
        }
    }
    if let Some(items) = value.as_sequence() {
        if ty.constraints.get("unique_items").and_then(Value::as_bool) == Some(true) {
            for (i, item) in items.iter().enumerate() {
                if items[..i].iter().any(|prior| yaml_equal(prior, item)) {
                    metadata(
                        out,
                        budget,
                        "metadata-duplicate",
                        &format!("{path}[{i}]"),
                        format!("{path}[{i}]: duplicate list item"),
                    );
                }
            }
        }
        if let Some(item_ty) = &ty.item {
            for (i, item) in items.iter().enumerate() {
                check_value(
                    &ResolvedField {
                        required: false,
                        ty: (**item_ty).clone(),
                    },
                    Some(item),
                    &format!("{path}[{i}]"),
                    depth + 1,
                    out,
                    budget,
                );
            }
        }
    }
    if let Some(map) = value.as_mapping() {
        for (name, child) in &ty.fields {
            check_value(
                child,
                map.get(Value::String(name.clone())),
                &format!("{path}.{name}"),
                depth + 1,
                out,
                budget,
            );
        }
        if ty
            .constraints
            .get("additional_properties")
            .and_then(Value::as_bool)
            == Some(false)
        {
            for key in map.keys().filter_map(Value::as_str) {
                if !ty.fields.contains_key(key) {
                    metadata(
                        out,
                        budget,
                        "metadata-unknown-property",
                        &format!("{path}.{key}"),
                        format!("{path}.{key}: undeclared property in closed object"),
                    );
                }
            }
        }
    }
}
fn bounded(value: &Value) -> String {
    serde_yaml::to_string(value)
        .unwrap_or_else(|_| "<value>".into())
        .trim()
        .chars()
        .take(160)
        .collect()
}
pub fn yaml_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Mapping(a), Value::Mapping(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|w| yaml_equal(v, w)))
        }
        (Value::Sequence(a), Value::Sequence(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(v, w)| yaml_equal(v, w))
        }
        _ => a == b,
    }
}
fn valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10
        || b[4] != b'-'
        || b[7] != b'-'
        || !b
            .iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
    {
        return false;
    }
    let year = s[..4].parse::<u32>().unwrap_or(0);
    let month = s[5..7].parse::<u32>().unwrap_or(0);
    let day = s[8..].parse::<u32>().unwrap_or(0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 0,
    };
    year > 0 && day > 0 && day <= days
}
fn valid_datetime(s: &str) -> bool {
    let regex=regex::Regex::new(r"^([0-9]{4}-[0-9]{2}-[0-9]{2})[Tt]([0-9]{2}):([0-9]{2}):([0-9]{2})(\.[0-9]+)?([Zz]|[+-][0-9]{2}:[0-9]{2})$").unwrap();
    let Some(c) = regex.captures(s) else {
        return false;
    };
    if !valid_date(&c[1]) {
        return false;
    }
    let n = |i: usize| c[i].parse::<u32>().unwrap_or(99);
    if n(2) > 23 || n(3) > 59 || n(4) > 60 {
        return false;
    }
    let zone = c.get(6).unwrap().as_str();
    zone.len() == 1
        || (zone[1..3].parse::<u32>().unwrap_or(99) <= 23
            && zone[4..].parse::<u32>().unwrap_or(99) <= 59)
}
fn valid_uri(s: &str) -> bool {
    fn unreserved(c: u8) -> bool {
        c.is_ascii_alphanumeric() || matches!(c, b'-' | b'.' | b'_' | b'~')
    }
    fn subdelim(c: u8) -> bool {
        matches!(
            c,
            b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'='
        )
    }
    fn component(s: &str, extra: &[u8]) -> bool {
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i];
            if c == b'%' {
                if i + 2 >= bytes.len()
                    || !bytes[i + 1].is_ascii_hexdigit()
                    || !bytes[i + 2].is_ascii_hexdigit()
                {
                    return false;
                }
                i += 3;
            } else {
                if !unreserved(c) && !subdelim(c) && !extra.contains(&c) {
                    return false;
                }
                i += 1;
            }
        }
        true
    }
    let Some((scheme, mut rest)) = s.split_once(':') else {
        return false;
    };
    if scheme.is_empty()
        || !scheme.as_bytes()[0].is_ascii_alphabetic()
        || !scheme
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'-' | b'.'))
    {
        return false;
    }
    if let Some(authority_path) = rest.strip_prefix("//") {
        let end = authority_path
            .find(['/', '?', '#'])
            .unwrap_or(authority_path.len());
        let authority = &authority_path[..end];
        rest = &authority_path[end..];
        let hostport = if let Some((userinfo, hostport)) = authority.rsplit_once('@') {
            if !component(userinfo, b":") {
                return false;
            }
            hostport
        } else {
            authority
        };
        if let Some(ip) = hostport.strip_prefix('[') {
            let Some((address, port)) = ip.split_once(']') else {
                return false;
            };
            let valid_ip = address.parse::<std::net::Ipv6Addr>().is_ok()
                || address.strip_prefix(['v', 'V']).is_some_and(|future| {
                    future.split_once('.').is_some_and(|(version, body)| {
                        !version.is_empty()
                            && version.bytes().all(|c| c.is_ascii_hexdigit())
                            && !body.is_empty()
                            && body
                                .bytes()
                                .all(|c| unreserved(c) || subdelim(c) || c == b':')
                    })
                });
            if !valid_ip
                || (!port.is_empty()
                    && !port
                        .strip_prefix(':')
                        .is_some_and(|p| p.bytes().all(|c| c.is_ascii_digit())))
            {
                return false;
            }
        } else {
            let (host, port) = hostport
                .split_once(':')
                .map(|(h, p)| (h, Some(p)))
                .unwrap_or((hostport, None));
            if !component(host, b"") || port.is_some_and(|p| !p.bytes().all(|c| c.is_ascii_digit()))
            {
                return false;
            }
        }
    }
    let (without_fragment, fragment) = rest
        .split_once('#')
        .map(|(p, f)| (p, Some(f)))
        .unwrap_or((rest, None));
    if fragment.is_some_and(|f| !component(f, b":@/?")) {
        return false;
    }
    let (path, query) = without_fragment
        .split_once('?')
        .map(|(p, q)| (p, Some(q)))
        .unwrap_or((without_fragment, None));
    component(path, b":@/") && query.is_none_or(|q| component(q, b":@/?"))
}

/// Local concept ancestry is independent of reusable field-type inheritance.
pub fn concept_parents(ct: &ConceptType) -> Result<Vec<String>> {
    match ct.extra.get("extends") {
        None => Ok(vec![]),
        Some(Value::String(s)) => Ok(vec![s.clone()]),
        Some(Value::Sequence(v)) => v
            .iter()
            .map(|v| {
                v.as_str().map(str::to_owned).ok_or_else(|| {
                    OkfError::Usage("concept extends entries must be type strings".into())
                })
            })
            .collect(),
        _ => Err(OkfError::Usage(
            "concept extends must be type string or list".into(),
        )),
    }
}
pub fn validate_concept_ancestry(ontology: &Ontology) -> Result<()> {
    fn visit(ontology: &Ontology, name: &str, chain: &mut Vec<String>) -> Result<()> {
        if chain.iter().any(|n| n == name) {
            return Err(OkfError::Usage(format!(
                "concept type inheritance cycle: {} -> {name}",
                chain.join(" -> ")
            )));
        }
        let ct = ontology
            .concepts
            .get(name)
            .ok_or_else(|| OkfError::Usage(format!("unknown concept parent type {name:?}")))?;
        chain.push(name.into());
        for parent in concept_parents(ct)? {
            visit(ontology, &parent, chain)?;
        }
        chain.pop();
        Ok(())
    }
    for name in ontology.concepts.keys() {
        visit(ontology, name, &mut Vec::new())?;
    }
    Ok(())
}
pub fn concept_is_a(ontology: &Ontology, actual: &str, expected: &str) -> bool {
    if actual == expected {
        return true;
    }
    let mut pending = vec![actual.to_owned()];
    let mut seen = std::collections::HashSet::new();
    while let Some(name) = pending.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        if let Some(ct) = ontology.concepts.get(&name) {
            for parent in concept_parents(ct).unwrap_or_default() {
                if parent == expected {
                    return true;
                }
                pending.push(parent);
            }
        }
    }
    false
}

fn check_declared_shape(base: FieldType, values: bool, item: bool, fields: bool) -> Result<()> {
    if (values && base != FieldType::Enum)
        || (item && base != FieldType::List)
        || (fields && base != FieldType::Object)
    {
        return Err(OkfError::Usage(format!(
            "invalid {base} schema: values/item/fields incompatible with type"
        )));
    }
    Ok(())
}

fn push_violation(out: &mut Vec<OntologyViolation>, budget: usize, violation: OntologyViolation) {
    if out.len() < budget {
        out.push(violation);
    } else {
        metadata(out, budget, "incomplete-check", "", String::new());
    }
}
