//! Append a `verified` entry (the write-side of trust).
//!
//! `verify` records that an actor reviewed a concept by appending a `{ by: <actor>, at:
//! <rfc3339> }` entry to the `verified` frontmatter list, losslessly:
//!
//! - no `verified` key → a new one-element list is appended;
//! - a `verified` list → the entry is pushed onto it (position preserved);
//! - a bare `verified` mapping or scalar → it is promoted to a two-element list
//!   `[<existing>, <new>]`, keeping the original entry verbatim.
//!
//! The derived trust tier (`model::trust`) then recomputes automatically from the actor
//! prefixes (`human:` → human-reviewed, else machine-confirmed).
use std::path::PathBuf;
use std::path::Path;

use serde_yaml::{Mapping, Value};

use crate::error::{OkfError, Result};
use crate::model::concept::ConceptId;
use crate::ports::clock::Clock;

use super::edit::{load_concept, save_concept};

/// What `verify` appended.
#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub id: ConceptId,
    pub path: PathBuf,
    pub actor: String,
    pub at: String,
}

/// Append a `verified` entry for `actor` (e.g. `human:andrey`) to the concept at `id`,
/// timestamped by `clock`. Losslessly rewrites the file.
pub fn verify(root: &Path, id: &str, actor: &str, clock: &dyn Clock) -> Result<VerifyResult> {
    if actor.trim().is_empty() {
        return Err(OkfError::Usage("verify: --by actor must not be empty".to_string()));
    }
    let cid = ConceptId::from_relative(id);
    let mut concept = load_concept(root, &cid)?;

    let at = clock.now_rfc3339();
    let mut entry = Mapping::new();
    entry.insert(Value::String("by".to_string()), Value::String(actor.to_string()));
    entry.insert(Value::String("at".to_string()), Value::String(at.clone()));
    let entry = Value::Mapping(entry);

    match concept.frontmatter.map.get_mut("verified") {
        Some(Value::Sequence(seq)) => seq.push(entry),
        Some(Value::Null) => {
            concept
                .frontmatter
                .map
                .insert("verified".to_string(), Value::Sequence(vec![entry]));
        }
        Some(existing) => {
            // A bare mapping/scalar entry becomes a one-then-two-element list.
            let prior = existing.clone();
            *existing = Value::Sequence(vec![prior, entry]);
        }
        None => {
            concept
                .frontmatter
                .map
                .insert("verified".to_string(), Value::Sequence(vec![entry]));
        }
    }

    let path = save_concept(root, &concept)?;
    Ok(VerifyResult {
        id: cid,
        path,
        actor: actor.to_string(),
        at,
    })
}
