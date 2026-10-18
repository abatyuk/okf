//! Semantic serialization back to disk (unknown keys/values and body preserved).
use crate::error::Result;
use crate::model::concept::Concept;

/// Assemble a concept file from already-serialized frontmatter text and a body.
///
/// Layout: `---\n<frontmatter>\n---\n<body>`. `frontmatter` is expected to already end
/// with a newline (as `serialize_frontmatter` produces); if it does not, one is added.
pub fn assemble(frontmatter: &str, body: &str) -> String {
    let mut out = String::with_capacity(frontmatter.len() + body.len() + 8);
    out.push_str("---\n");
    out.push_str(frontmatter);
    if !frontmatter.is_empty() && !frontmatter.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("---\n");
    out.push_str(body);
    out
}

/// Render authored order for read-only inspection (including physical line windows).
pub fn write_concept(concept: &Concept) -> Result<String> {
    let fm = super::yaml::serialize_frontmatter(&concept.frontmatter.map)?;
    Ok(assemble(&fm, &concept.body))
}

/// Serialize declared fields in ontology order, recursively, followed by undeclared keys.
/// Missing fields are never synthesized; unknown values and list order are preserved.
pub fn write_concept_with_ontology(
    concept: &Concept,
    ontology: Option<&crate::ontology::schema::Ontology>,
) -> Result<String> {
    use crate::ontology::field_types::resolve_field;
    use indexmap::IndexMap;
    let mut map = concept.frontmatter.map.clone();
    let mut ordered = IndexMap::new();
    if let Some(value) = map.shift_remove("type") {
        ordered.insert("type".to_string(), value);
    }
    if let Some(ontology) = ontology {
        if let Some(ct) = concept
            .frontmatter
            .get("type")
            .and_then(serde_yaml::Value::as_str)
            .and_then(|name| ontology.concepts.get(name))
        {
            for (name, field) in &ct.fields {
                if let Some(mut value) = map.shift_remove(name) {
                    order_value(&mut value, &resolve_field(ontology, field)?.ty);
                    ordered.insert(name.clone(), value);
                }
            }
            // Reference-only declarations follow typed fields in reference declaration order.
            for name in ct.references.keys() {
                if let Some(value) = map.shift_remove(name) {
                    ordered.insert(name.clone(), value);
                }
            }
        }
    }
    ordered.extend(map);
    let fm = super::yaml::serialize_frontmatter(&ordered)?;
    Ok(assemble(&fm, &concept.body))
}

fn order_value(value: &mut serde_yaml::Value, ty: &crate::ontology::field_types::ResolvedType) {
    use serde_yaml::{Mapping, Value};
    match value {
        Value::Mapping(map) => {
            let mut remaining = std::mem::take(map);
            let mut ordered = Mapping::new();
            for (name, field) in &ty.fields {
                let key = Value::String(name.clone());
                if let Some(mut value) = remaining.shift_remove(&key) {
                    order_value(&mut value, &field.ty);
                    ordered.insert(key, value);
                }
            }
            ordered.extend(remaining);
            *map = ordered;
        }
        Value::Sequence(items) => {
            if let Some(item_type) = &ty.item {
                for item in items {
                    order_value(item, item_type);
                }
            }
        }
        _ => {}
    }
}

/// Whether serialization would change mapping order (ignores YAML presentation).
pub fn needs_order_normalization(
    concept: &Concept,
    ontology: Option<&crate::ontology::schema::Ontology>,
) -> Result<bool> {
    let authored = assemble(
        &super::yaml::serialize_frontmatter(&concept.frontmatter.map)?,
        &concept.body,
    );
    Ok(authored != write_concept_with_ontology(concept, ontology)?)
}
