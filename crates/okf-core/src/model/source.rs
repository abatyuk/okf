//! Sources and their typed fingerprints (drives `okf stale` / `okf refresh`).
use indexmap::IndexMap;
use serde_yaml::{Mapping, Value};

/// The kind of a source, which selects how it is fingerprinted and compared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceKind {
    GitCommit,
    GitPath,
    MarkdownHeading,
    LineRange,
    File,
    Url,
    /// Producer-defined kind (the set is extensible).
    Other(String),
}

impl SourceKind {
    /// Parse the on-disk `kind` string. An unrecognized value round-trips as [`SourceKind::Other`].
    pub fn from_kind_str(s: &str) -> Self {
        match s {
            "git-commit" => SourceKind::GitCommit,
            "git-path" => SourceKind::GitPath,
            "markdown-heading" => SourceKind::MarkdownHeading,
            "line-range" => SourceKind::LineRange,
            "file" => SourceKind::File,
            "url" => SourceKind::Url,
            other => SourceKind::Other(other.to_string()),
        }
    }

    /// The canonical `kind` string as written back to frontmatter.
    pub fn as_kind_str(&self) -> &str {
        match self {
            SourceKind::GitCommit => "git-commit",
            SourceKind::GitPath => "git-path",
            SourceKind::MarkdownHeading => "markdown-heading",
            SourceKind::LineRange => "line-range",
            SourceKind::File => "file",
            SourceKind::Url => "url",
            SourceKind::Other(s) => s,
        }
    }
}

/// A recorded fingerprint: a small, kind-specific, order-preserving map captured at last sync.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fingerprint {
    pub fields: Vec<(String, String)>,
}

impl Fingerprint {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pairs(fields: Vec<(String, String)>) -> Self {
        Self { fields }
    }

    /// Value for a fingerprint field (e.g. `blob_sha`, `content_sha256`, `etag`).
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Read a fingerprint sub-map (`fingerprint: { … }`) from YAML. Non-string scalar values
    /// are stringified so heterogeneous recorded values (hex, etags, timestamps) round-trip.
    pub fn from_value(value: &Value) -> Self {
        let mut fields = Vec::new();
        if let Some(map) = value.as_mapping() {
            for (k, v) in map {
                if let Some(key) = k.as_str() {
                    fields.push((key.to_string(), value_to_string(v)));
                }
            }
        }
        Self { fields }
    }

    /// Serialize back into a YAML mapping, preserving field order.
    pub fn to_value(&self) -> Value {
        let mut m = Mapping::new();
        for (k, v) in &self.fields {
            m.insert(Value::String(k.clone()), Value::String(v.clone()));
        }
        Value::Mapping(m)
    }
}

/// An OKF `sources[]` entry plus our `kind` + `fingerprint` extension. Any other keys on the
/// entry (`id`, `author`, `usage_count`, `last_modified`, …) are preserved verbatim in
/// [`Source::extra`] so `refresh` round-trips losslessly.
#[derive(Debug, Clone, PartialEq)]
pub struct Source {
    pub resource: String,
    pub kind: SourceKind,
    pub fingerprint: Fingerprint,
    /// Other spec/unknown keys on the entry, in original order.
    pub extra: IndexMap<String, Value>,
}

impl Source {
    /// Parse one `sources[]` entry (a YAML mapping). Returns `None` if the value is not a mapping.
    pub fn from_value(value: &Value) -> Option<Self> {
        let map = value.as_mapping()?;
        let mut resource = String::new();
        let mut kind: Option<SourceKind> = None;
        let mut fingerprint = Fingerprint::default();
        let mut extra = IndexMap::new();
        for (k, v) in map {
            let Some(key) = k.as_str() else { continue };
            match key {
                "resource" => resource = value_to_string(v),
                "kind" => kind = Some(SourceKind::from_kind_str(v.as_str().unwrap_or_default())),
                "fingerprint" => fingerprint = Fingerprint::from_value(v),
                other => {
                    extra.insert(other.to_string(), v.clone());
                }
            }
        }
        Some(Source {
            resource,
            kind: kind.unwrap_or_else(|| SourceKind::Other(String::new())),
            fingerprint,
            extra,
        })
    }

    /// Serialize back into a YAML mapping: `resource`, `kind`, `fingerprint`, then extras.
    pub fn to_value(&self) -> Value {
        let mut m = Mapping::new();
        m.insert(
            Value::String("resource".to_string()),
            Value::String(self.resource.clone()),
        );
        // Only emit `kind` when meaningfully present (avoid churning kind-less entries).
        if !matches!(&self.kind, SourceKind::Other(s) if s.is_empty()) {
            m.insert(
                Value::String("kind".to_string()),
                Value::String(self.kind.as_kind_str().to_string()),
            );
        }
        if !self.fingerprint.is_empty() {
            m.insert(
                Value::String("fingerprint".to_string()),
                self.fingerprint.to_value(),
            );
        }
        for (k, v) in &self.extra {
            m.insert(Value::String(k.clone()), v.clone());
        }
        Value::Mapping(m)
    }
}

/// Read the `sources` frontmatter field into typed [`Source`] entries (skips non-mappings).
pub fn parse_sources(sources: &Value) -> Vec<Source> {
    match sources {
        Value::Sequence(seq) => seq.iter().filter_map(Source::from_value).collect(),
        _ => Vec::new(),
    }
}

/// Stringify a YAML scalar for a fingerprint/resource field; non-scalars fall back to a
/// compact YAML rendering.
fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => String::new(),
        other => serde_yaml::to_string(other)
            .unwrap_or_default()
            .trim_end()
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> Value {
        serde_yaml::from_str(
            "resource: src/billing/mileage.py\n\
             kind: git-path\n\
             fingerprint:\n  blob_sha: 3f9a1c\n\
             id: src-1\n\
             author: alice\n",
        )
        .unwrap()
    }

    #[test]
    fn parses_kind_fingerprint_and_extra() {
        let s = Source::from_value(&entry()).unwrap();
        assert_eq!(s.resource, "src/billing/mileage.py");
        assert_eq!(s.kind, SourceKind::GitPath);
        assert_eq!(s.fingerprint.get("blob_sha"), Some("3f9a1c"));
        // Non-typed keys survive in `extra`, in order.
        let keys: Vec<&str> = s.extra.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["id", "author"]);
    }

    #[test]
    fn round_trips_losslessly() {
        let original = entry();
        let s = Source::from_value(&original).unwrap();
        let back = s.to_value();
        assert_eq!(back, original, "value round-trip must be stable");

        // And idempotent on a second pass.
        let s2 = Source::from_value(&back).unwrap();
        assert_eq!(s2.to_value(), back);
    }

    #[test]
    fn unknown_kind_round_trips() {
        let v: Value = serde_yaml::from_str("resource: x\nkind: sql-query\n").unwrap();
        let s = Source::from_value(&v).unwrap();
        assert_eq!(s.kind, SourceKind::Other("sql-query".to_string()));
        assert_eq!(s.to_value(), v);
    }
}
