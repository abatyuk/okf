//! Walk a directory into concepts; honor reserved filenames (index.md/log.md).
use crate::error::{OkfError, Result};
use crate::model::concept::{Concept, ConceptId};
use crate::parse::parse_concept;
use std::path::{Path, PathBuf};

use super::walk::walk_markdown;

/// Reserved filenames that are structural, not concepts.
const RESERVED: [&str; 2] = ["index.md", "log.md"];

/// A loaded bundle: its root and the concepts it contains (sorted by id).
#[derive(Debug, Clone)]
pub struct Bundle {
    pub root: PathBuf,
    pub concepts: Vec<Concept>,
}

impl Bundle {
    /// Find a concept by id. Accepts ids with or without a leading slash.
    pub fn get(&self, id: &str) -> Option<&Concept> {
        let target = ConceptId::from_relative(id);
        self.concepts.iter().find(|c| c.id == target)
    }
}

/// Walk `root` (respecting `.gitignore`) and load every `*.md` file except reserved
/// names into a [`Bundle`].
pub fn load_bundle(root: &Path) -> Result<Bundle> {
    if !root.is_dir() {
        return Err(OkfError::Environment(format!(
            "bundle path is not a directory: {}",
            root.display()
        )));
    }

    let mut concepts = Vec::new();
    for rel in walk_markdown(root)? {
        let path = root.join(&rel);
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if RESERVED.contains(&file_name) {
            continue;
        }

        let rel_str = rel.to_string_lossy();
        let stem = rel_str
            .strip_suffix(".md")
            .ok_or_else(|| OkfError::Internal(format!("expected .md suffix: {rel_str}")))?;
        let id = ConceptId::from_relative(stem);

        let content = std::fs::read_to_string(&path)
            .map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))?;
        concepts.push(parse_concept(id, &content)?);
    }

    concepts.sort_by(|a, b| a.id.0.cmp(&b.id.0));

    Ok(Bundle {
        root: root.to_path_buf(),
        concepts,
    })
}
