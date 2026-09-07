//! Show one concept's full content.
use crate::bundle::loader::Bundle;
use crate::model::concept::Concept;

/// Look up a single concept by id (leading slash optional).
pub fn show<'a>(bundle: &'a Bundle, id: &str) -> Option<&'a Concept> {
    bundle.get(id)
}
