//! ontology.yaml model, composition, load and edit.
//!
//! - [`schema`] — the typed model ([`Ontology`], [`ConceptType`], [`Field`],
//!   [`FieldType`], [`ReferenceRule`], [`Cardinality`]).
//! - [`field_types`] — `extends`/`object` composition, cycle detection, and the
//!   ontology-conformance check consumed by `lint`.
//! - [`load`] — locate and parse `ontology.yaml` (`try_load` is non-fatal on absence).
//! - [`edit`] — self-validating add/update/remove of concept types.
pub mod edit;
pub mod field_types;
pub mod load;
pub mod schema;

pub use schema::{
    Cardinality, ConceptType, Field, FieldType, FieldTypeDef, Ontology, ReferenceRule, Target,
    TrustExpectation,
};
