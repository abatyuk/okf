//! Deterministic traversal of the physical OKF bundle tree.
//!
//! Git ignore rules do not redefine a bundle. We exclude VCS and transaction control metadata and do not follow
//! directory symlinks.

use std::path::{Path, PathBuf};

use crate::error::{OkfError, Result};

const EXCLUDED_DIRS: &[&str] = &[".git", ".okf-transaction"];

/// Walk every regular file below `root`, in bundle-relative order.
pub fn walk_files(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.is_dir() {
        return Err(OkfError::Environment(format!(
            "bundle path is not a directory: {}",
            root.display()
        )));
    }
    let mut out = Vec::new();
    walk_dir(root, root, &mut out, &|_| true)?;
    out.sort();
    Ok(out)
}

/// Walk every `.md` file below `root`, regardless of Git ignore rules.
pub fn walk_markdown(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.is_dir() {
        return Err(OkfError::Environment(format!(
            "bundle path is not a directory: {}",
            root.display()
        )));
    }
    let mut out = Vec::new();
    walk_dir(root, root, &mut out, &|path| {
        path.extension().and_then(|e| e.to_str()) == Some("md")
    })?;
    out.sort();
    Ok(out)
}

fn walk_dir(
    root: &Path,
    dir: &Path,
    out: &mut Vec<PathBuf>,
    include: &dyn Fn(&Path) -> bool,
) -> Result<()> {
    let mut entries = std::fs::read_dir(dir)
        .map_err(|e| OkfError::Io(format!("{}: {e}", dir.display())))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| OkfError::Io(format!("{}: {e}", dir.display())))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let ty = entry
            .file_type()
            .map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))?;
        if ty.is_dir() {
            if !EXCLUDED_DIRS.contains(&name.as_ref()) {
                walk_dir(root, &path, out, include)?;
            }
        } else if ty.is_file() && name != ".okf-transaction.lock" && include(&path) {
            out.push(
                path.strip_prefix(root)
                    .map_err(|e| OkfError::Internal(e.to_string()))?
                    .to_path_buf(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_transaction_control_files_from_bundle_discovery() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join(".okf-transaction.lock"), "").unwrap();
        std::fs::create_dir(root.path().join(".okf-transaction")).unwrap();
        std::fs::write(
            root.path().join(".okf-transaction/manifest.json"),
            "private before images",
        )
        .unwrap();
        std::fs::write(
            root.path().join(".okf-transaction/backup.md"),
            "private document",
        )
        .unwrap();
        std::fs::write(root.path().join(".authored.md"), "authored").unwrap();
        std::fs::write(root.path().join("note.md"), "note").unwrap();
        assert_eq!(
            walk_files(root.path()).unwrap(),
            vec![PathBuf::from(".authored.md"), PathBuf::from("note.md")]
        );
        assert_eq!(
            walk_markdown(root.path()).unwrap(),
            vec![PathBuf::from(".authored.md"), PathBuf::from("note.md")]
        );
    }

    #[test]
    fn includes_gitignored_files_but_not_git_metadata() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join(".gitignore"), "ignored.md\n").unwrap();
        std::fs::write(root.path().join("ignored.md"), "x").unwrap();
        std::fs::create_dir(root.path().join(".git")).unwrap();
        std::fs::write(root.path().join(".git/internal.md"), "x").unwrap();
        assert_eq!(
            walk_markdown(root.path()).unwrap(),
            vec![PathBuf::from("ignored.md")]
        );
    }
}
