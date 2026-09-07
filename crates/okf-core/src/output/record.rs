//! NDJSON record types.
//!
//! A `concept` record mirrors the concept's frontmatter **verbatim** (same keys, same
//! order — `serde_json` is built with `preserve_order`), plus computed `id` and
//! `trust_tier`. There is no fixed concept schema: the record *is* the frontmatter.
use crate::model::concept::Concept;
use serde_json::{Map, Value as Json};
use serde_yaml::Value as Yaml;

/// Convert a YAML value into the equivalent JSON value. Non-string mapping keys are
/// coerced to their string form (spec frontmatter keys are strings in practice).
pub fn yaml_to_json(value: &Yaml) -> Json {
    match value {
        Yaml::Null => Json::Null,
        Yaml::Bool(b) => Json::Bool(*b),
        Yaml::Number(n) => {
            if let Some(i) = n.as_i64() {
                Json::from(i)
            } else if let Some(u) = n.as_u64() {
                Json::from(u)
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(Json::Number)
                    .unwrap_or(Json::Null)
            } else {
                Json::Null
            }
        }
        Yaml::String(s) => Json::String(s.clone()),
        Yaml::Sequence(seq) => Json::Array(seq.iter().map(yaml_to_json).collect()),
        Yaml::Mapping(map) => {
            let mut obj = Map::new();
            for (k, v) in map {
                let key = match k {
                    Yaml::String(s) => s.clone(),
                    other => yaml_scalar_key(other),
                };
                obj.insert(key, yaml_to_json(v));
            }
            Json::Object(obj)
        }
        Yaml::Tagged(t) => yaml_to_json(&t.value),
    }
}

fn yaml_scalar_key(v: &Yaml) -> String {
    match v {
        Yaml::String(s) => s.clone(),
        Yaml::Bool(b) => b.to_string(),
        Yaml::Number(n) => n.to_string(),
        Yaml::Null => "null".to_string(),
        _ => serde_yaml::to_string(v)
            .unwrap_or_default()
            .trim()
            .to_string(),
    }
}

/// Build the NDJSON `concept` record: `id` first, then the frontmatter verbatim, then the
/// computed `trust_tier`.
pub fn concept_record(concept: &Concept) -> Json {
    let mut obj = Map::new();
    obj.insert("id".to_string(), Json::String(concept.id.0.clone()));
    for (k, v) in &concept.frontmatter.map {
        obj.insert(k.clone(), yaml_to_json(v));
    }
    obj.insert(
        "trust_tier".to_string(),
        Json::String(concept.trust_tier().as_str().to_string()),
    );
    Json::Object(obj)
}
