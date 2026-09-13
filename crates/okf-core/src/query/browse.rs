//! Browse a bundle through its structural `index.md` files.
//!
//! An existing index is returned without loading the bundle's concept documents. When the
//! requested directory has no index, the consumer fallback allowed by the OKF specification is
//! used: load the bundle and synthesize the directory listing in memory.
use crate::bundle::loader::load_bundle_metadata;
use crate::error::{OkfError, Result};
use crate::render::index::generate_index;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexSource {
    File,
    Synthesized,
}

impl IndexSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Synthesized => "synthesized",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowseResult {
    /// Canonical bundle-relative directory (`/` for the root).
    pub directory: String,
    /// Bundle-relative path of the structural index.
    pub path: PathBuf,
    pub source: IndexSource,
    pub content: String,
}

/// Return a directory's checked-in `index.md`, or synthesize it when absent.
pub fn browse(root: &Path, directory: Option<&str>) -> Result<BrowseResult> {
    let relative = normalize_directory(directory.unwrap_or("/"))?;
    let absolute_dir = root.join(&relative);
    if !absolute_dir.is_dir() {
        return Err(OkfError::Usage(format!(
            "bundle directory not found: {}",
            display_directory(&relative)
        )));
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|e| OkfError::Io(format!("{}: {e}", root.display())))?;
    let canonical_dir = absolute_dir
        .canonicalize()
        .map_err(|e| OkfError::Io(format!("{}: {e}", absolute_dir.display())))?;
    if !canonical_dir.starts_with(&canonical_root) {
        return Err(OkfError::Usage(format!(
            "directory must stay inside the bundle: {:?}",
            directory.unwrap_or("/")
        )));
    }

    let path = relative.join("index.md");
    let absolute_index = root.join(&path);
    if absolute_index.is_file() {
        let content = std::fs::read_to_string(&absolute_index)
            .map_err(|e| OkfError::Io(format!("{}: {e}", absolute_index.display())))?;
        return Ok(BrowseResult {
            directory: display_directory(&relative),
            path,
            source: IndexSource::File,
            content,
        });
    }

    let bundle = load_bundle_metadata(root)?;
    let content = generate_index(&bundle, &relative).content;
    Ok(BrowseResult {
        directory: display_directory(&relative),
        path,
        source: IndexSource::Synthesized,
        content,
    })
}

fn normalize_directory(raw: &str) -> Result<PathBuf> {
    let normalized = raw.trim().replace('\\', "/");
    let mut out = PathBuf::new();
    for component in Path::new(&normalized).components() {
        match component {
            Component::RootDir | Component::CurDir => {}
            Component::Normal(segment) => out.push(segment),
            Component::ParentDir | Component::Prefix(_) => {
                return Err(OkfError::Usage(format!(
                    "directory must stay inside the bundle: {raw:?}"
                )))
            }
        }
    }
    Ok(out)
}

fn display_directory(relative: &Path) -> String {
    if relative.as_os_str().is_empty() {
        "/".to_string()
    } else {
        format!("/{}", relative.to_string_lossy().replace('\\', "/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample-bundle")
    }

    #[test]
    fn reads_existing_root_index_without_synthesizing() {
        let result = browse(&fixture(), None).unwrap();
        assert_eq!(result.directory, "/");
        assert_eq!(result.path, PathBuf::from("index.md"));
        assert_eq!(result.source, IndexSource::File);
        assert!(result.content.contains("reserved `index.md`"));
    }

    #[test]
    fn synthesizes_a_missing_directory_index() {
        let result = browse(&fixture(), Some("tables")).unwrap();
        assert_eq!(result.directory, "/tables");
        assert_eq!(result.source, IndexSource::Synthesized);
        assert!(result.content.contains("[Customers](customers.md)"));
    }

    #[test]
    fn rejects_parent_traversal() {
        assert!(browse(&fixture(), Some("../outside")).is_err());
    }

    #[test]
    fn existing_index_does_not_require_loading_concepts() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("index.md"), "# Available\n").unwrap();
        std::fs::write(tmp.path().join("broken.md"), "no frontmatter\n").unwrap();
        let result = browse(tmp.path(), None).unwrap();
        assert_eq!(result.source, IndexSource::File);
        assert_eq!(result.content, "# Available\n");
    }
}
