//! okf config file model (`okf.toml`).
//!
//! A tool-local, project-scoped config discovered by walking up from the current directory.
//! Its `bundle` key names a default bundle directory (relative to the config file, or absolute).
//! The bundle itself stays spec-pure — this file lives outside it.
use crate::error::{OkfError, Result};
use std::path::{Path, PathBuf};

/// The config filename searched for from the cwd upward.
pub const CONFIG_FILENAME: &str = "okf.toml";

/// Parsed `okf.toml`. Unknown keys are ignored so the format can grow compatibly.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct Config {
    /// Default bundle directory, relative to the config file's directory (or absolute).
    pub bundle: Option<String>,
}

/// Walk up from `start` (inclusive) to the filesystem root, returning the first `okf.toml` found.
pub fn find_config(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(d) = dir {
        let candidate = d.join(CONFIG_FILENAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = d.parent();
    }
    None
}

/// Read and parse a config file. A present-but-malformed file is a usage error.
pub fn load_config(path: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| OkfError::Environment(format!("cannot read {}: {e}", path.display())))?;
    toml::from_str(&text)
        .map_err(|e| OkfError::Usage(format!("invalid config {}: {e}", path.display())))
}

/// Resolve the bundle a config file points at, relative to the config's own directory.
/// Returns `None` when no config is found or it has no `bundle` key.
pub fn config_bundle(start: &Path) -> Result<Option<PathBuf>> {
    let Some(cfg_path) = find_config(start) else {
        return Ok(None);
    };
    let cfg = load_config(&cfg_path)?;
    let Some(bundle) = cfg.bundle else {
        return Ok(None);
    };
    let base = cfg_path.parent().unwrap_or(start);
    // `Path::join` returns `bundle` as-is when it is absolute, else joins onto `base`.
    Ok(Some(base.join(bundle)))
}
