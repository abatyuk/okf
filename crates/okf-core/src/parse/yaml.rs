//! Lossless YAML load (order-preserving mapping) and serialize.
use crate::error::{OkfError, Result};
use indexmap::IndexMap;
use serde_yaml::Value;

/// Parse a frontmatter block into an order-preserving map. An empty/blank block is an
/// empty map (not an error).
pub fn parse_frontmatter(yaml: &str) -> Result<IndexMap<String, Value>> {
    if yaml.trim().is_empty() {
        return Ok(IndexMap::new());
    }
    serde_yaml::from_str::<IndexMap<String, Value>>(yaml).map_err(|e| OkfError::Yaml(e.to_string()))
}

/// Serialize a frontmatter map back to YAML text, preserving key order. Returns an empty
/// string for an empty map (so no `{}` noise is emitted).
pub fn serialize_frontmatter(map: &IndexMap<String, Value>) -> Result<String> {
    if map.is_empty() {
        return Ok(String::new());
    }
    serde_yaml::to_string(map).map_err(|e| OkfError::Internal(e.to_string()))
}
