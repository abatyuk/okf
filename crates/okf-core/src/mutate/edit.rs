//! Losslessly edit a concept's frontmatter and body.
//!
//! `edit` applies a batch of operations while preserving unknown frontmatter keys and their
//! on-disk order: an existing key is updated *in place* (its position is retained), a new key
//! is appended, and everything else round-trips byte-stably through
//! [`parse_concept`]/[`write_concept`].
//!
//! **Frontmatter ops.** `--set key=value` sets a scalar (`true`/`false` → bool, canonical
//! ints/floats → number, `null` → null, else string; values that would lose information when
//! coerced, like `007` or `1.0`, stay strings). `--unset key` removes a field. `--add
//! key=value` appends an item to a list field (creating the list, idempotent), and `--remove
//! key=value` drops matching item(s). `add_sources` appends typed source mappings; structured
//! `verified` entries still belong to the dedicated `verify` writer.
//!
//! **Body ops.** `--set-body`/`--append-body`/`--clear-body` rewrite the whole body; the
//! section-aware `--set-section`/`--append-section`/`--remove-section` splice a single heading
//! (see [`crate::mutate::body`]).
//!
//! Operations apply in a fixed order (unset, set, add, remove, then whole-body, then section
//! edits), so a single invocation is deterministic regardless of flag order.
use std::path::{Path, PathBuf};

use serde_yaml::Value;

use crate::error::{OkfError, Result};
use crate::model::concept::{Concept, ConceptId};
use crate::model::frontmatter::Frontmatter;
use crate::model::source::Source;
use crate::mutate::body;
use crate::parse::{parse_concept, writer::write_concept};

/// The on-disk path a concept id maps to (`/policies/travel` → `<root>/policies/travel.md`).
pub(crate) fn id_to_path(root: &Path, id: &ConceptId) -> PathBuf {
    let rel = id.0.trim_start_matches('/');
    root.join(format!("{rel}.md"))
}

/// Read + parse the concept at `id` from disk. A missing file is an environment error.
pub(crate) fn load_concept(root: &Path, id: &ConceptId) -> Result<Concept> {
    let path = id_to_path(root, id);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| OkfError::Environment(format!("cannot read {}: {e}", path.display())))?;
    parse_concept(id.clone(), &content)
}

/// Serialize `concept` losslessly and write it to its id-derived path (creating parent
/// directories as needed). Returns the path written.
pub(crate) fn save_concept(root: &Path, concept: &Concept) -> Result<PathBuf> {
    let path = id_to_path(root, &concept.id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| OkfError::Io(format!("{}: {e}", parent.display())))?;
    }
    let text = write_concept(concept)?;
    std::fs::write(&path, text).map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))?;
    Ok(path)
}

/// A batch of edits to apply to one concept in a single, deterministic pass. Empty by default;
/// callers fill the fields for the operations they want.
#[derive(Debug, Default, Clone)]
pub struct EditSpec {
    /// `--set key=value`: set/update a scalar field.
    pub sets: Vec<(String, String)>,
    /// `--unset key`: remove a field.
    pub unsets: Vec<String>,
    /// `--add key=value`: append an item to a list field.
    pub adds: Vec<(String, String)>,
    /// `--remove key=value`: drop matching item(s) from a list field.
    pub removes: Vec<(String, String)>,
    /// Structured entries appended to `sources`.
    pub add_sources: Vec<Source>,
    /// `--clear-body`: empty the body.
    pub clear_body: bool,
    /// `--set-body`: replace the whole body.
    pub set_body: Option<String>,
    /// `--append-body`: append a block to the body.
    pub append_body: Option<String>,
    /// `--set-section (heading, text)`: replace a section's content.
    pub set_sections: Vec<(String, String)>,
    /// `--append-section (heading, text)`: append to a section.
    pub append_sections: Vec<(String, String)>,
    /// `--remove-section heading`: delete a section (heading + content).
    pub remove_sections: Vec<String>,
}

impl EditSpec {
    /// True when no operation is requested.
    pub fn is_empty(&self) -> bool {
        self.sets.is_empty()
            && self.unsets.is_empty()
            && self.adds.is_empty()
            && self.removes.is_empty()
            && self.add_sources.is_empty()
            && !self.clear_body
            && self.set_body.is_none()
            && self.append_body.is_none()
            && self.set_sections.is_empty()
            && self.append_sections.is_empty()
            && self.remove_sections.is_empty()
    }
}

/// One applied change, for reporting (text summary + `--json` detail).
#[derive(Debug, Clone, PartialEq)]
pub enum EditChange {
    /// A scalar field set; `existed` is whether it was updated in place vs. appended.
    Set { key: String, existed: bool },
    /// A field removed; `existed` is whether it was present.
    Unset { key: String, existed: bool },
    /// An item appended to a list field; `added` is false if it was already present.
    Add { key: String, added: bool },
    /// Item(s) removed from a list field; `removed` is how many matched.
    Remove { key: String, removed: usize },
    /// The whole body was replaced.
    SetBody,
    /// A block was appended to the body.
    AppendBody,
    /// The body was cleared.
    ClearBody,
    /// A section's content was replaced.
    SetSection { heading: String },
    /// A block was appended to a section.
    AppendSection { heading: String },
    /// A section was removed.
    RemoveSection { heading: String },
}

/// What `edit` changed.
#[derive(Debug, Clone)]
pub struct EditResult {
    pub id: ConceptId,
    pub path: PathBuf,
    pub changes: Vec<EditChange>,
}

/// Parse a scalar `key=value` string into a YAML value, conservatively: bool, canonical
/// integer/float, `null`, else the raw string (see the module note on lossy coercions).
pub fn parse_scalar(raw: &str) -> Value {
    match raw {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        "null" | "~" => return Value::Null,
        _ => {}
    }
    if let Ok(i) = raw.parse::<i64>() {
        if i.to_string() == raw {
            return Value::Number(i.into());
        }
    }
    if raw.contains('.') || raw.contains('e') || raw.contains('E') {
        if let Ok(f) = raw.parse::<f64>() {
            // Only accept as a number when it round-trips exactly (rejects "1.0", "1e",…).
            if f.to_string() == raw {
                return Value::Number(serde_yaml::Number::from(f));
            }
        }
    }
    Value::String(raw.to_string())
}

/// Split a `key=value` argument for `--set`/`--add`/`--remove`. The first `=` separates; the
/// value may itself contain `=`. `flag` names the flag for the error message.
pub fn parse_kv(flag: &str, arg: &str) -> Result<(String, String)> {
    match arg.split_once('=') {
        Some((k, v)) if !k.trim().is_empty() => Ok((k.trim().to_string(), v.to_string())),
        _ => Err(OkfError::Usage(format!(
            "invalid {flag} argument {arg:?}: expected key=value"
        ))),
    }
}

/// Back-compat alias for `--set`. Prefer [`parse_kv`].
pub fn parse_set_arg(arg: &str) -> Result<(String, String)> {
    parse_kv("--set", arg)
}

/// Append `value` to the list field `key`, creating it if absent. Idempotent: an item already
/// present is not duplicated. Errors if the field exists as a non-list scalar.
fn apply_add(fm: &mut Frontmatter, key: &str, value: Value) -> Result<bool> {
    match fm.map.get_mut(key) {
        None => {
            fm.map.insert(key.to_string(), Value::Sequence(vec![value]));
            Ok(true)
        }
        Some(Value::Sequence(seq)) => {
            if seq.contains(&value) {
                Ok(false)
            } else {
                seq.push(value);
                Ok(true)
            }
        }
        Some(_) => Err(OkfError::Usage(format!(
            "--add: field {key:?} is not a list; use --set to replace it"
        ))),
    }
}

/// Remove every item equal to `value` from the list field `key`. A missing field is a no-op
/// (returns 0). Errors if the field exists as a non-list scalar.
fn apply_remove(fm: &mut Frontmatter, key: &str, value: Value) -> Result<usize> {
    match fm.map.get_mut(key) {
        None => Ok(0),
        Some(Value::Sequence(seq)) => {
            let before = seq.len();
            seq.retain(|v| v != &value);
            Ok(before - seq.len())
        }
        Some(_) => Err(OkfError::Usage(format!(
            "--remove: field {key:?} is not a list"
        ))),
    }
}

/// Apply a batch of frontmatter and body edits to the concept at `id`, losslessly.
///
/// Operations run in a fixed order (unset, set, add, remove, whole-body, section edits) so the
/// result is deterministic regardless of flag order. Existing keys are updated in place; new
/// keys are appended; untouched keys and body regions are preserved byte-stably.
pub fn edit(root: &Path, id: &str, spec: &EditSpec) -> Result<EditResult> {
    if spec.is_empty() {
        return Err(OkfError::Usage("edit: no operations specified".to_string()));
    }
    let cid = ConceptId::from_relative(id);
    let mut concept = load_concept(root, &cid)?;
    let mut changes = Vec::new();

    for key in &spec.unsets {
        ensure_key(key)?;
        // shift_remove preserves the order of the remaining keys.
        let existed = concept.frontmatter.map.shift_remove(key).is_some();
        changes.push(EditChange::Unset {
            key: key.clone(),
            existed,
        });
    }
    for (key, raw) in &spec.sets {
        ensure_key(key)?;
        let existed = concept.frontmatter.map.contains_key(key);
        // IndexMap::insert updates in place (keeping position) or appends a new key.
        concept
            .frontmatter
            .map
            .insert(key.clone(), parse_scalar(raw));
        changes.push(EditChange::Set {
            key: key.clone(),
            existed,
        });
    }
    for (key, raw) in &spec.adds {
        ensure_key(key)?;
        let added = apply_add(&mut concept.frontmatter, key, parse_scalar(raw))?;
        changes.push(EditChange::Add {
            key: key.clone(),
            added,
        });
    }
    for (key, raw) in &spec.removes {
        ensure_key(key)?;
        let removed = apply_remove(&mut concept.frontmatter, key, parse_scalar(raw))?;
        changes.push(EditChange::Remove {
            key: key.clone(),
            removed,
        });
    }
    for source in &spec.add_sources {
        let added = apply_add(&mut concept.frontmatter, "sources", source.to_value())?;
        changes.push(EditChange::Add {
            key: "sources".to_string(),
            added,
        });
    }

    if spec.clear_body {
        concept.body = String::new();
        changes.push(EditChange::ClearBody);
    }
    if let Some(text) = &spec.set_body {
        concept.body = body::set_body(text);
        changes.push(EditChange::SetBody);
    }
    if let Some(text) = &spec.append_body {
        concept.body = body::append_body(&concept.body, text);
        changes.push(EditChange::AppendBody);
    }

    for heading in &spec.remove_sections {
        concept.body = body::remove_section(&concept.body, heading)?;
        changes.push(EditChange::RemoveSection {
            heading: heading.clone(),
        });
    }
    for (heading, text) in &spec.set_sections {
        concept.body = body::set_section(&concept.body, heading, text)?;
        changes.push(EditChange::SetSection {
            heading: heading.clone(),
        });
    }
    for (heading, text) in &spec.append_sections {
        concept.body = body::append_section(&concept.body, heading, text)?;
        changes.push(EditChange::AppendSection {
            heading: heading.clone(),
        });
    }

    let path = save_concept(root, &concept)?;
    Ok(EditResult {
        id: cid,
        path,
        changes,
    })
}

/// Reject an empty/whitespace field key.
fn ensure_key(key: &str) -> Result<()> {
    if key.trim().is_empty() {
        return Err(OkfError::Usage("edit: empty field key".to_string()));
    }
    Ok(())
}
