//! Safe, read-only resolution and bounded retrieval of OKF path-valued artifacts.

use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::bundle::walk::walk_files;
use crate::error::{OkfError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Concept,
    Reserved,
    Artifact,
    External,
    Scope,
    Missing,
    Blocked,
}

impl ArtifactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Concept => "concept",
            Self::Reserved => "reserved",
            Self::Artifact => "artifact",
            Self::External => "external",
            Self::Scope => "scope",
            Self::Missing => "missing",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedArtifact {
    pub resource: String,
    pub kind: ArtifactKind,
    pub path: Option<PathBuf>,
    pub exists: bool,
    pub size: Option<u64>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArtifactEntry {
    pub path: PathBuf,
    pub kind: ArtifactKind,
    pub size: u64,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArtifactContent {
    pub resolved: ResolvedArtifact,
    pub text: Option<String>,
    pub sha256: String,
    pub truncated: bool,
    pub binary: bool,
}

pub fn list_artifacts(
    root: &Path,
    directory: Option<&str>,
    digest: bool,
) -> Result<Vec<ArtifactEntry>> {
    let prefix = normalize_local(directory.unwrap_or(""), None)?;
    let mut out = Vec::new();
    for rel in walk_files(root)? {
        if !prefix.as_os_str().is_empty() && !rel.starts_with(&prefix) {
            continue;
        }
        let abs = root.join(&rel);
        let metadata =
            std::fs::metadata(&abs).map_err(|e| OkfError::Io(format!("{}: {e}", abs.display())))?;
        let kind = classify_existing(&rel);
        let sha256 = digest
            .then(|| std::fs::read(&abs).map(|b| hex_sha256(&b)))
            .transpose()?;
        out.push(ArtifactEntry {
            path: rel,
            kind,
            size: metadata.len(),
            sha256,
        });
    }
    Ok(out)
}

pub fn resolve_artifact(root: &Path, from: Option<&str>, resource: &str) -> ResolvedArtifact {
    let trimmed = resource.trim();
    if trimmed.len() >= 3
        && trimmed.as_bytes()[0].is_ascii_alphabetic()
        && trimmed.as_bytes()[1] == b':'
        && matches!(trimmed.as_bytes()[2], b'/' | b'\\')
    {
        return ResolvedArtifact {
            resource: resource.to_string(),
            kind: ArtifactKind::Blocked,
            path: None,
            exists: false,
            size: None,
            message: Some("filesystem-prefixed artifact paths are not bundle-relative".to_string()),
        };
    }
    if is_uri(trimmed) {
        return ResolvedArtifact {
            resource: resource.to_string(),
            kind: ArtifactKind::External,
            path: None,
            exists: false,
            size: None,
            message: None,
        };
    }
    if looks_like_scope(trimmed) {
        return ResolvedArtifact {
            resource: resource.to_string(),
            kind: ArtifactKind::Scope,
            path: None,
            exists: false,
            size: None,
            message: Some("resource is a scope descriptor, not a retrievable path".to_string()),
        };
    }
    let rel = match normalize_local(trimmed, from) {
        Ok(path) => path,
        Err(e) => {
            return ResolvedArtifact {
                resource: resource.to_string(),
                kind: ArtifactKind::Blocked,
                path: None,
                exists: false,
                size: None,
                message: Some(e.to_string()),
            }
        }
    };
    let abs = root.join(&rel);
    if !abs.exists() {
        return ResolvedArtifact {
            resource: resource.to_string(),
            kind: ArtifactKind::Missing,
            path: Some(rel),
            exists: false,
            size: None,
            message: None,
        };
    }
    let canonical_root = match root.canonicalize() {
        Ok(path) => path,
        Err(e) => return blocked(resource, rel, e.to_string()),
    };
    let canonical = match abs.canonicalize() {
        Ok(path) => path,
        Err(e) => return blocked(resource, rel, e.to_string()),
    };
    if !canonical.starts_with(&canonical_root) {
        return blocked(resource, rel, "resolved path escapes bundle".to_string());
    }
    let metadata = match std::fs::metadata(&canonical) {
        Ok(value) if value.is_file() => value,
        Ok(_) => return blocked(resource, rel, "resolved resource is not a file".to_string()),
        Err(e) => return blocked(resource, rel, e.to_string()),
    };
    ResolvedArtifact {
        resource: resource.to_string(),
        kind: classify_existing(&rel),
        path: Some(rel),
        exists: true,
        size: Some(metadata.len()),
        message: None,
    }
}

pub fn show_artifact(
    root: &Path,
    from: Option<&str>,
    resource: &str,
    max_bytes: usize,
    lines: Option<(usize, usize)>,
) -> Result<ArtifactContent> {
    let resolved = resolve_artifact(root, from, resource);
    if !matches!(
        resolved.kind,
        ArtifactKind::Artifact | ArtifactKind::Reserved
    ) {
        return Err(OkfError::Usage(format!(
            "artifact show requires a local opaque/reserved artifact; resolved as {}",
            resolved.kind.as_str()
        )));
    }
    let rel = resolved.path.as_ref().unwrap();
    let bytes = std::fs::read(root.join(rel))
        .map_err(|e| OkfError::Io(format!("{}: {e}", rel.display())))?;
    let sha256 = hex_sha256(&bytes);
    let truncated = bytes.len() > max_bytes;
    let slice = &bytes[..bytes.len().min(max_bytes)];
    let (text, binary) = match std::str::from_utf8(slice) {
        Ok(text) => {
            let selected = if let Some((start, end)) = lines {
                if start == 0 || end < start {
                    return Err(OkfError::Usage(
                        "lines must be a positive START:END range".to_string(),
                    ));
                }
                text.lines()
                    .enumerate()
                    .filter_map(|(i, line)| {
                        let n = i + 1;
                        (n >= start && n <= end).then_some(line)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                text.to_string()
            };
            (Some(selected), false)
        }
        Err(_) => (None, true),
    };
    Ok(ArtifactContent {
        resolved,
        text,
        sha256,
        truncated,
        binary,
    })
}

/// Fetch a bounded HTTP(S) artifact only when the network feature is compiled in. The caller
/// must still enforce its explicit authorization/policy boundary.
pub fn fetch_artifact(resource: &str, max_bytes: usize) -> Result<ArtifactContent> {
    if !(resource.starts_with("https://") || resource.starts_with("http://")) {
        return Err(OkfError::Usage(
            "remote artifact scheme must be http or https".to_string(),
        ));
    }
    fetch_impl(resource, max_bytes)
}

#[cfg(feature = "url-sources")]
fn fetch_impl(resource: &str, max_bytes: usize) -> Result<ArtifactContent> {
    use std::io::Read;
    use std::time::Duration;

    let response = ureq::get(resource)
        .timeout(Duration::from_secs(15))
        .call()
        .map_err(|e| OkfError::Environment(format!("remote artifact fetch failed: {e}")))?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| OkfError::Io(format!("remote artifact read failed: {e}")))?;
    let truncated = bytes.len() > max_bytes;
    bytes.truncate(max_bytes);
    let sha256 = hex_sha256(&bytes);
    let (text, binary) = match String::from_utf8(bytes) {
        Ok(text) => (Some(text), false),
        Err(_) => (None, true),
    };
    Ok(ArtifactContent {
        resolved: ResolvedArtifact {
            resource: resource.to_string(),
            kind: ArtifactKind::External,
            path: None,
            exists: true,
            size: None,
            message: None,
        },
        text,
        sha256,
        truncated,
        binary,
    })
}

#[cfg(not(feature = "url-sources"))]
fn fetch_impl(_resource: &str, _max_bytes: usize) -> Result<ArtifactContent> {
    Err(OkfError::Environment(
        "remote artifact retrieval requires the `url-sources` build feature".to_string(),
    ))
}

fn normalize_local(resource: &str, from: Option<&str>) -> Result<PathBuf> {
    let core = resource
        .split(['#', '?'])
        .next()
        .unwrap_or(resource)
        .replace('\\', "/");
    let mut joined = PathBuf::new();
    if !core.starts_with('/') {
        if let Some(from) = from {
            let from = from
                .trim_start_matches('/')
                .strip_suffix(".md")
                .unwrap_or(from.trim_start_matches('/'));
            if let Some(parent) = Path::new(from).parent() {
                joined.push(parent);
            }
        }
    }
    joined.push(core.trim_start_matches('/'));
    let mut out = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(segment) => out.push(segment),
            Component::ParentDir => {
                if !out.pop() {
                    return Err(OkfError::Usage("artifact path escapes bundle".to_string()));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(OkfError::Usage(
                    "artifact path must be bundle-relative".to_string(),
                ))
            }
        }
    }
    Ok(out)
}

fn classify_existing(path: &Path) -> ArtifactKind {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if matches!(name, "index.md" | "log.md") {
        ArtifactKind::Reserved
    } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
        ArtifactKind::Concept
    } else {
        ArtifactKind::Artifact
    }
}

fn blocked(resource: &str, path: PathBuf, message: String) -> ResolvedArtifact {
    ResolvedArtifact {
        resource: resource.to_string(),
        kind: ArtifactKind::Blocked,
        path: Some(path),
        exists: false,
        size: None,
        message: Some(message),
    }
}

fn is_uri(s: &str) -> bool {
    s.split_once(':').is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    })
}

fn looks_like_scope(s: &str) -> bool {
    s.is_empty() || s.chars().any(char::is_whitespace)
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_reference_artifacts_and_blocks_escape() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("references")).unwrap();
        std::fs::write(root.path().join("references/a.sql"), "select 1\n").unwrap();
        let r = resolve_artifact(root.path(), Some("metrics/revenue"), "../references/a.sql");
        assert_eq!(r.kind, ArtifactKind::Artifact);
        assert_eq!(r.path.as_deref(), Some(Path::new("references/a.sql")));
        assert_eq!(
            resolve_artifact(root.path(), None, "../x").kind,
            ArtifactKind::Blocked
        );
    }

    #[test]
    fn retrieves_bounded_text_without_executing() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("a.sql"), "one\ntwo\nthree\n").unwrap();
        let got = show_artifact(root.path(), None, "a.sql", 1024, Some((2, 2))).unwrap();
        assert_eq!(got.text.as_deref(), Some("two"));
        assert!(!got.binary);
    }

    #[test]
    fn binary_and_scope_results_are_distinct() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("a.bin"), [0, 159, 255]).unwrap();
        let got = show_artifact(root.path(), None, "a.bin", 1024, None).unwrap();
        assert!(got.binary);
        assert!(got.text.is_none());
        assert_eq!(
            resolve_artifact(root.path(), None, "all queries in project X").kind,
            ArtifactKind::Scope
        );
        assert_eq!(
            resolve_artifact(root.path(), None, "https://example.com/a").kind,
            ArtifactKind::External
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_blocked() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        symlink(outside.path(), root.path().join("outside.txt")).unwrap();
        assert_eq!(
            resolve_artifact(root.path(), None, "outside.txt").kind,
            ArtifactKind::Blocked
        );
    }
}
