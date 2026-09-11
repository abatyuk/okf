//! Ontology model types: [`Ontology`], [`ConceptType`], [`Field`], [`FieldTypeDef`],
//! [`FieldType`], [`ReferenceRule`], [`Cardinality`].
//!
//! These mirror the `ontology.yaml` shape drafted in INTENT.md. All types are serde
//! (de)serializable and preserve unknown keys via a flattened `extra` map. The ontology writer
//! restores comments after serialization.
use std::fmt;
use std::str::FromStr;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::error::OkfError;

/// The whole `ontology.yaml`, order-preserving. Unknown top-level keys survive in `extra`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Ontology {
    /// Version of *this* ontology file's schema (our tool), e.g. `"0.1"`. Not the OKF version.
    pub okf_ontology: String,

    /// Reusable, composable typed field definitions (DRY). Composable via `extends` and
    /// via `object` bases with nested `fields`.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub field_types: IndexMap<String, FieldTypeDef>,

    /// Source kinds `okf stale` knows how to fingerprint. Advisory list; unmodelled here
    /// beyond preserving order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_kinds: Vec<String>,

    /// Concept types. The KEY is the OKF `type` string verbatim (may contain spaces).
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub concepts: IndexMap<String, ConceptType>,

    /// Any unknown top-level keys, preserved verbatim for lossless round-trips.
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// A single concept type: its typed fields, advisory trust expectations and typed
/// reference rules.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ConceptType {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// OKF built-in fields that MUST be present on a concept of this type (advisory).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,

    /// Typed custom fields keyed by frontmatter key.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub fields: IndexMap<String, Field>,

    /// If true, `okf add --attested` scaffolds this as an OKF Attested Computation.
    #[serde(default, skip_serializing_if = "is_false")]
    pub attested: bool,

    /// Advisory trust expectation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust: Option<TrustExpectation>,

    /// Typed reference rules, keyed by the frontmatter key that holds the links
    /// (e.g. `computations`).
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub references: IndexMap<String, ReferenceRule>,

    /// Unknown per-concept keys preserved verbatim.
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// Advisory trust expectation for a concept type.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TrustExpectation {
    /// `lint` warns if the concept's derived tier is lower than this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_tier: Option<String>,

    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// A typed field declaration on a concept type. `type` is either a primitive
/// ([`FieldType`]) name or a `field_types` name.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Field {
    /// The declared type name: a primitive keyword or a `field_types` entry name.
    #[serde(rename = "type")]
    pub type_name: String,

    /// Whether the field must be present. Defaults to false.
    #[serde(default, skip_serializing_if = "is_false")]
    pub required: bool,

    /// For `enum` fields: the allowed values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,

    /// For `list` fields: the element type name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,

    /// For `object` fields: nested sub-fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<IndexMap<String, Field>>,

    /// Any further constraints (`pattern`, `min`, `max`, …) preserved verbatim.
    #[serde(flatten)]
    pub constraints: IndexMap<String, Value>,
}

/// A reusable, composable field-type definition (an entry under `field_types:`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FieldTypeDef {
    /// The primitive base kind name (`string`, `object`, …). Mutually informative with
    /// `extends` — a definition may inherit its base from a parent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,

    /// Inherit base + constraints from another `field_types` entry (most-derived wins).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// For an `enum` base: the allowed values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,

    /// For a `list` base: the element type name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,

    /// For an `object` base: nested sub-fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<IndexMap<String, Field>>,

    /// Constraints (`pattern`, `min`, `max`, …), preserved verbatim and merged on `extends`.
    #[serde(flatten)]
    pub constraints: IndexMap<String, Value>,
}

/// The primitive field kinds understood by the tool. Any other name must resolve through
/// `field_types`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Text,
    Int,
    Bool,
    Date,
    Datetime,
    Uri,
    Enum,
    List,
    Object,
}

impl FieldType {
    /// Parse a primitive keyword. Returns `None` for names that must resolve via
    /// `field_types`.
    pub fn from_keyword(s: &str) -> Option<FieldType> {
        Some(match s {
            "string" => FieldType::String,
            "text" => FieldType::Text,
            "int" => FieldType::Int,
            "bool" => FieldType::Bool,
            "date" => FieldType::Date,
            "datetime" => FieldType::Datetime,
            "uri" => FieldType::Uri,
            "enum" => FieldType::Enum,
            "list" => FieldType::List,
            "object" => FieldType::Object,
            _ => return None,
        })
    }

    pub fn as_keyword(self) -> &'static str {
        match self {
            FieldType::String => "string",
            FieldType::Text => "text",
            FieldType::Int => "int",
            FieldType::Bool => "bool",
            FieldType::Date => "date",
            FieldType::Datetime => "datetime",
            FieldType::Uri => "uri",
            FieldType::Enum => "enum",
            FieldType::List => "list",
            FieldType::Object => "object",
        }
    }
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_keyword())
    }
}

/// A typed reference rule: a frontmatter key on the source concept constrained to link at
/// certain target type(s) with a cardinality.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferenceRule {
    /// The allowed target type(s). A single string or a list (union of targets).
    pub target: Target,

    /// How many links this rule allows.
    pub cardinality: Cardinality,

    /// Unknown reference-rule keys preserved verbatim.
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

/// A reference rule's target: one type or a union of types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Target {
    One(String),
    Many(Vec<String>),
}

impl Target {
    /// The target type names as a flat slice-like vector.
    pub fn types(&self) -> Vec<&str> {
        match self {
            Target::One(s) => vec![s.as_str()],
            Target::Many(v) => v.iter().map(String::as_str).collect(),
        }
    }

    /// True if `ty` is one of the allowed targets.
    pub fn allows(&self, ty: &str) -> bool {
        self.types().contains(&ty)
    }
}

/// Reference / list cardinality vocabulary: `0..1`, `1..1`, `0..n`, `1..n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// `0..1` — optional single.
    ZeroOne,
    /// `1..1` — required single.
    OneOne,
    /// `0..n` — optional many.
    ZeroN,
    /// `1..n` — at least one.
    OneN,
}

impl Cardinality {
    /// Minimum number of links required.
    pub fn min(self) -> usize {
        match self {
            Cardinality::ZeroOne | Cardinality::ZeroN => 0,
            Cardinality::OneOne | Cardinality::OneN => 1,
        }
    }

    /// Maximum number of links allowed (`None` == unbounded `n`).
    pub fn max(self) -> Option<usize> {
        match self {
            Cardinality::ZeroOne | Cardinality::OneOne => Some(1),
            Cardinality::ZeroN | Cardinality::OneN => None,
        }
    }

    /// True if a count of `n` links satisfies this cardinality.
    pub fn permits(self, n: usize) -> bool {
        n >= self.min() && self.max().is_none_or(|m| n <= m)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Cardinality::ZeroOne => "0..1",
            Cardinality::OneOne => "1..1",
            Cardinality::ZeroN => "0..n",
            Cardinality::OneN => "1..n",
        }
    }
}

impl FromStr for Cardinality {
    type Err = OkfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "0..1" => Ok(Cardinality::ZeroOne),
            "1..1" => Ok(Cardinality::OneOne),
            "0..n" => Ok(Cardinality::ZeroN),
            "1..n" => Ok(Cardinality::OneN),
            other => Err(OkfError::Usage(format!(
                "invalid cardinality {other:?}: expected one of 0..1, 1..1, 0..n, 1..n"
            ))),
        }
    }
}

impl fmt::Display for Cardinality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for Cardinality {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Cardinality {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

fn is_false(b: &bool) -> bool {
    !*b
}
