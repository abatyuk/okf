//! Locate and parse the tool-local `ontology.yaml`.
//!
//! A missing ontology is **not** fatal for the tool as a whole — [`try_load`] returns
//! `Ok(None)` when there is no file, so callers (`lint`, `add`) can proceed without one.
//! A present-but-broken ontology *is* an error (`Usage`/`Yaml`).
use std::path::{Path, PathBuf};

use crate::error::{OkfError, Result};

use super::field_types::validate_field_types;
use super::schema::Ontology;

/// The tool-local ontology filename, resolved at the bundle root.
pub const ONTOLOGY_FILENAME: &str = "ontology.yaml";

/// The path the ontology would live at for a given bundle root.
pub fn ontology_path(bundle_root: &Path) -> PathBuf {
    bundle_root.join(ONTOLOGY_FILENAME)
}

/// Locate `ontology.yaml` at the bundle root, returning its path if it exists.
pub fn find_ontology(bundle_root: &Path) -> Option<PathBuf> {
    let p = ontology_path(bundle_root);
    p.is_file().then_some(p)
}

/// Parse an ontology from YAML text, then validate its structure (version present, all
/// field types resolvable, no `extends` cycles, cardinalities well-formed).
pub fn parse_ontology(text: &str) -> Result<Ontology> {
    let ontology: Ontology =
        serde_yaml::from_str(text).map_err(|e| OkfError::Yaml(e.to_string()))?;
    validate_ontology(&ontology)?;
    Ok(ontology)
}

/// Validate an already-parsed ontology's structure. Cardinality/`Target` well-formedness is
/// enforced by deserialization; this adds version and field-type-composition checks.
pub fn validate_ontology(ontology: &Ontology) -> Result<()> {
    if ontology.okf_ontology.trim().is_empty() {
        return Err(OkfError::Usage(
            "ontology.yaml: missing or empty `okf_ontology` version".to_string(),
        ));
    }
    validate_field_types(ontology)?;
    Ok(())
}

/// Load and validate the ontology at an explicit path. Errors if the file is
/// unreadable, unparseable or structurally invalid.
pub fn load_ontology(path: &Path) -> Result<Ontology> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| OkfError::Environment(format!("cannot read {}: {e}", path.display())))?;
    parse_ontology(&text)
}

/// Load the ontology for a bundle if one is present. `Ok(None)` when absent (not fatal);
/// `Err` when present but unreadable/invalid.
pub fn try_load(bundle_root: &Path) -> Result<Option<Ontology>> {
    match find_ontology(bundle_root) {
        Some(path) => load_ontology(&path).map(Some),
        None => Ok(None),
    }
}
