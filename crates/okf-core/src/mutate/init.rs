//! Create a new empty OKF bundle.
//!
//! A bundle is just a directory of markdown concepts, so `init` is deliberately minimal: it
//! creates the target directory and, on request, scaffolds two optional structural files:
//!
//! - a root `index.md` (progressive-disclosure entry point; the spec treats `index.md` as a
//!   reserved, non-concept file), and
//! - an empty tool-local `ontology.yaml` (sidecar, spec-external — see INTENT.md) carrying only
//!   the schema-version key so `okf ontology add` has a valid file to grow.
//!
//! Both are opt-in and neither is required for a conformant bundle. `init` never overwrites an
//! existing `index.md`/`ontology.yaml`; it reports which files it actually created.
use std::path::{Path, PathBuf};

use crate::error::{OkfError, Result};
use crate::ontology::load::ONTOLOGY_FILENAME;

/// Options controlling which optional scaffold files `init` writes.
#[derive(Debug, Clone)]
pub struct InitOptions {
    /// Write a minimal root `index.md`.
    pub index: bool,
    /// Write a minimal `ontology.yaml` (just the schema-version key).
    pub ontology: bool,
    /// Title used in the scaffolded `index.md` heading.
    pub title: Option<String>,
}

impl Default for InitOptions {
    /// Sensible defaults: scaffold both an `index.md` and an `ontology.yaml`.
    fn default() -> Self {
        Self {
            index: true,
            ontology: true,
            title: None,
        }
    }
}

/// What `init` created.
#[derive(Debug, Clone)]
pub struct InitResult {
    pub root: PathBuf,
    /// True if `init` created the bundle directory (false if it already existed).
    pub created_dir: bool,
    /// Path to the `index.md` if one was written this run.
    pub index_path: Option<PathBuf>,
    /// Path to the `ontology.yaml` if one was written this run.
    pub ontology_path: Option<PathBuf>,
}

/// The `okf_ontology` schema version scaffolded into a fresh `ontology.yaml`.
const ONTOLOGY_VERSION: &str = "0.1";

/// Initialize a bundle at `root`. Creates the directory (idempotent) and, per `opts`, writes a
/// root `index.md` and/or an empty `ontology.yaml`, never clobbering existing ones.
pub fn init(root: &Path, opts: &InitOptions) -> Result<InitResult> {
    let created_dir = !root.exists();
    if root.exists() && !root.is_dir() {
        return Err(OkfError::Usage(format!(
            "cannot init bundle: {} exists and is not a directory",
            root.display()
        )));
    }
    std::fs::create_dir_all(root)
        .map_err(|e| OkfError::Io(format!("{}: {e}", root.display())))?;

    let mut index_path = None;
    if opts.index {
        let path = root.join("index.md");
        if !path.exists() {
            let title = opts
                .title
                .clone()
                .or_else(|| {
                    root.file_name()
                        .and_then(|n| n.to_str())
                        .map(str::to_string)
                })
                .unwrap_or_else(|| "Knowledge bundle".to_string());
            let body = format!("# {title}\n\nAn OKF knowledge bundle.\n");
            std::fs::write(&path, body)
                .map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))?;
            index_path = Some(path);
        }
    }

    let mut ontology_path = None;
    if opts.ontology {
        let path = root.join(ONTOLOGY_FILENAME);
        if !path.exists() {
            let body = format!("okf_ontology: \"{ONTOLOGY_VERSION}\"\n");
            std::fs::write(&path, body)
                .map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))?;
            ontology_path = Some(path);
        }
    }

    Ok(InitResult {
        root: root.to_path_buf(),
        created_dir,
        index_path,
        ontology_path,
    })
}
