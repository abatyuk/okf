//! Order-preserving, unknown-key-preserving frontmatter — the round-trip linchpin.
//!
//! Backed by `indexmap::IndexMap<String, serde_yaml::Value>` so key order and unknown
//! keys survive read → write byte-stable, and so the NDJSON `concept` record can mirror
//! the frontmatter verbatim.
use indexmap::IndexMap;
use serde_yaml::Value;

/// Lossless frontmatter: an ordered map of YAML values. Never discards keys.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Frontmatter {
    /// Insertion-ordered key/value pairs, preserving the on-disk order and any
    /// unknown keys verbatim.
    pub map: IndexMap<String, Value>,
}

impl Frontmatter {
    pub fn new() -> Self {
        Self { map: IndexMap::new() }
    }

    pub fn from_map(map: IndexMap<String, Value>) -> Self {
        Self { map }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Raw value for a key, preserving the underlying YAML type.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(key)
    }

    /// Typed string accessor (e.g. `get_str("type")`) that does not discard the map.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.map.get(key).and_then(Value::as_str)
    }

    /// The `tags` field flattened to a list of strings (non-string items are skipped).
    pub fn get_tags(&self) -> Vec<String> {
        match self.map.get("tags") {
            Some(Value::Sequence(seq)) => seq
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect(),
            Some(Value::String(s)) => vec![s.clone()],
            _ => Vec::new(),
        }
    }
}
