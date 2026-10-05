//! Optional tool-local catalog. Identities do not change standard concept IDs.
use crate::error::{OkfError, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogFile {
    pub catalog_version: u32,
    pub bundles: BTreeMap<String, Registration>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Registration {
    pub location: Location,
    #[serde(flatten)]
    pub settings: BTreeMap<String, serde_yaml::Value>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    #[serde(rename = "type")]
    pub type_: String,
    pub path: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct CatalogEntry {
    pub id: String,
    pub configured_root: PathBuf,
    pub root: PathBuf,
    pub overridden: bool,
    pub available: bool,
    #[serde(skip)]
    pub settings: BTreeMap<String, serde_yaml::Value>,
}
#[derive(Debug, Clone)]
pub struct Catalog {
    pub path: PathBuf,
    pub entries: BTreeMap<String, CatalogEntry>,
}
/// Machine-local override file: `bundles: {acme.finance: /local/checkout}`.
/// Paths are relative to the override file. Only registered IDs can be overridden.
pub fn load_catalog(path: &Path, overrides: Option<&Path>) -> Result<Catalog> {
    let text = std::fs::read_to_string(path)?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text)
        .map_err(|e| OkfError::Usage(format!("invalid catalog {}: {e}", path.display())))?;
    let file: CatalogFile = serde_yaml::from_value(value)
        .map_err(|e| OkfError::Usage(format!("invalid catalog {}: {e}", path.display())))?;
    if file.catalog_version != 1 {
        return Err(OkfError::Usage(
            "unsupported catalog_version (expected 1)".into(),
        ));
    }
    let mut entries = BTreeMap::new();
    for (id, registration) in file.bundles {
        if !valid_id(&id) {
            return Err(OkfError::Usage(format!(
                "bundle ID must be namespaced: {id:?}"
            )));
        }
        if registration.location.type_ != "directory" {
            return Err(OkfError::Usage(format!(
                "unsupported location type for {id}: only directory is supported"
            )));
        }
        let root = absolute(
            &path
                .parent()
                .unwrap_or(Path::new("."))
                .join(registration.location.path),
        )?;
        entries.insert(
            id.clone(),
            CatalogEntry {
                id,
                configured_root: root.clone(),
                root,
                overridden: false,
                available: false,
                settings: registration.settings,
            },
        );
    }
    if let Some(overrides) = overrides {
        #[derive(Deserialize)]
        struct Overrides {
            bundles: BTreeMap<String, String>,
        }
        let override_text = std::fs::read_to_string(overrides)?;
        let value: serde_yaml::Value = serde_yaml::from_str(&override_text)
            .map_err(|e| OkfError::Usage(format!("invalid catalog overrides: {e}")))?;
        let file: Overrides = serde_yaml::from_value(value)
            .map_err(|e| OkfError::Usage(format!("invalid catalog overrides: {e}")))?;
        for (id, root) in file.bundles {
            let entry = entries.get_mut(&id).ok_or_else(|| {
                OkfError::Usage(format!("override names unknown bundle ID: {id}"))
            })?;
            entry.root = absolute(&overrides.parent().unwrap_or(Path::new(".")).join(root))?;
            entry.overridden = true;
        }
    }
    for entry in entries.values_mut() {
        entry.available = entry.root.is_dir();
        if entry.available {
            entry.root = std::fs::canonicalize(&entry.root)?;
        }
    }
    let available: Vec<_> = entries.values().filter(|e| e.available).collect();
    for (index, first) in available.iter().enumerate() {
        for second in &available[index + 1..] {
            if first.root.starts_with(&second.root) || second.root.starts_with(&first.root) {
                return Err(OkfError::Usage(format!(
                    "conflicting bundle roots: {} ({}) and {} ({}) are equal or nested",
                    first.id,
                    first.root.display(),
                    second.id,
                    second.root.display()
                )));
            }
        }
    }
    Ok(Catalog {
        path: path.to_path_buf(),
        entries,
    })
}
impl Catalog {
    pub fn entry(&self, id: &str) -> Result<&CatalogEntry> {
        self.entries
            .get(id)
            .ok_or_else(|| OkfError::Usage(format!("unknown bundle ID: {id}")))
    }
    pub fn containing(&self, path: &Path) -> Option<&CatalogEntry> {
        let path = std::fs::canonicalize(path).unwrap_or_else(|_| normalize(path));
        self.entries
            .values()
            .find(|entry| entry.available && path.starts_with(&entry.root))
    }
}
pub fn valid_id(id: &str) -> bool {
    let segments: Vec<_> = id.split('.').collect();
    segments.len() >= 2
        && segments.iter().all(|s| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
}
pub fn absolute(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    Ok(normalize(&path))
}
/// Normalize actual filesystem paths without clamping traversal to a bundle root.
pub fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            part => result.push(part.as_os_str()),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(yaml: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("a")).unwrap();
        std::fs::create_dir(dir.path().join("b")).unwrap();
        std::fs::write(dir.path().join("catalog.yaml"), yaml).unwrap();
        dir
    }
    #[test]
    fn validates_namespaced_ids_and_duplicate_registrations() {
        for bundles in ["plain: {location: {type: directory, path: a}}","acme.a: {location: {type: directory, path: a}}\n  acme.a: {location: {type: directory, path: b}}"] {
            let dir=fixture(&format!("catalog_version: 1\nbundles:\n  {bundles}\n"));
            assert_eq!(load_catalog(&dir.path().join("catalog.yaml"),None).unwrap_err().exit_code(),2);
        }
    }
    #[test]
    fn rejects_equal_and_nested_roots_without_treating_unavailable_as_verified() {
        for path in ["a", "a/child"] {
            let dir=fixture(&format!("catalog_version: 1\nbundles:\n  acme.a: {{location: {{type: directory, path: a}}}}\n  acme.b: {{location: {{type: directory, path: {path}}}}}\n"));
            if path == "a/child" {
                std::fs::create_dir(dir.path().join(path)).unwrap();
            }
            assert!(load_catalog(&dir.path().join("catalog.yaml"), None).is_err());
        }
        let dir=fixture("catalog_version: 1\nbundles:\n  acme.a: {location: {type: directory, path: absent}}\n  acme.b: {location: {type: directory, path: absent/child}}\n");
        let catalog = load_catalog(&dir.path().join("catalog.yaml"), None).unwrap();
        assert!(catalog.entries.values().all(|e| !e.available));
    }
    #[test]
    fn overrides_preserve_configured_root_and_validate_effective_overlap() {
        let dir=fixture("catalog_version: 1\nbundles:\n  acme.a: {location: {type: directory, path: absent}}\n  acme.b: {location: {type: directory, path: b}}\n");
        let overrides = dir.path().join("local.yaml");
        std::fs::write(&overrides, "bundles: {acme.a: a}\n").unwrap();
        let catalog = load_catalog(&dir.path().join("catalog.yaml"), Some(&overrides)).unwrap();
        let entry = &catalog.entries["acme.a"];
        assert!(entry.available && entry.overridden);
        assert!(entry.configured_root.ends_with("absent"));
        assert!(entry.root.ends_with("a"));
        std::fs::write(&overrides, "bundles: {acme.a: b}\n").unwrap();
        assert!(load_catalog(&dir.path().join("catalog.yaml"), Some(&overrides)).is_err());
    }
}
