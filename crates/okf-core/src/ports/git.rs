//! Git effects — a thin wrapper over the `git` CLI (subprocess), not a git library.
//! Runtime-optional: only git-based source kinds and `okf diff` need it.
use crate::error::{OkfError, Result};
use std::path::Path;
use std::process::Command;

pub trait Git {
    /// Locate the worktree root containing `anchor`. Test ports default to the anchor itself.
    fn worktree_root(&self, anchor: &Path) -> Result<std::path::PathBuf> {
        Ok(anchor.to_path_buf())
    }
    /// `git hash-object <path>` — the blob object id (git-path fingerprint).
    fn hash_object(&self, path: &Path) -> Result<String>;
    /// `git log -1 --format=%H -- <path>` — last commit touching a path (git-commit).
    fn last_commit(&self, path: &Path) -> Result<String>;
    /// `git show <rev>:<path>` — file contents at a ref (diff).
    fn show(&self, rev: &str, path: &Path) -> Result<Vec<u8>>;
    /// `git ls-tree -r --name-only <rev>` — every tracked path at a ref (diff).
    fn ls_tree(&self, rev: &str) -> Result<Vec<String>>;
}

/// Production git, shelling out to the `git` CLI. A missing `git` on PATH surfaces as
/// [`OkfError::Environment`] (exit 3); a non-zero git exit surfaces as [`OkfError::Io`].
pub struct RealGit;

impl RealGit {
    fn run(cmd: &mut Command) -> Result<std::process::Output> {
        match cmd.output() {
            Ok(out) if out.status.success() => Ok(out),
            Ok(out) => Err(OkfError::Io(format!(
                "git failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(OkfError::Environment(
                "`git` executable not found on PATH".to_string(),
            )),
            Err(e) => Err(OkfError::Io(e.to_string())),
        }
    }

    fn repo_path(path: &Path) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
        let anchor = path.parent().unwrap_or_else(|| Path::new("."));
        let out = Self::run(
            Command::new("git")
                .arg("-C")
                .arg(anchor)
                .args(["rev-parse", "--show-toplevel"]),
        )?;
        let root = std::path::PathBuf::from(
            String::from_utf8_lossy(&out.stdout).trim().to_string(),
        );
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| OkfError::Environment(e.to_string()))?
                .join(path)
        };
        let relative = absolute.strip_prefix(&root).map_err(|_| {
            OkfError::Usage(format!("git source {} is outside worktree {}", path.display(), root.display()))
        })?;
        Ok((root, relative.to_path_buf()))
    }
}

impl Git for RealGit {
    fn worktree_root(&self, anchor: &Path) -> Result<std::path::PathBuf> {
        let out = Self::run(
            Command::new("git")
                .arg("-C")
                .arg(anchor)
                .args(["rev-parse", "--show-toplevel"]),
        )?;
        Ok(std::path::PathBuf::from(
            String::from_utf8_lossy(&out.stdout).trim().to_string(),
        ))
    }
    fn hash_object(&self, path: &Path) -> Result<String> {
        let (root, relative) = Self::repo_path(path)?;
        let out = Self::run(
            Command::new("git")
                .arg("-C")
                .arg(root)
                .arg("hash-object")
                .arg(relative),
        )?;
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    fn last_commit(&self, path: &Path) -> Result<String> {
        let (root, relative) = Self::repo_path(path)?;
        let out = Self::run(
            Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["log", "-1", "--format=%H", "--"])
                .arg(relative),
        )?;
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    fn show(&self, rev: &str, path: &Path) -> Result<Vec<u8>> {
        let spec = format!("{rev}:{}", path.display());
        let out = Self::run(Command::new("git").arg("show").arg(&spec))?;
        Ok(out.stdout)
    }

    fn ls_tree(&self, rev: &str) -> Result<Vec<String>> {
        let out = Self::run(Command::new("git").args(["ls-tree", "-r", "--name-only", rev]))?;
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect())
    }
}

/// In-memory git for hermetic tests: register exactly the answers a test needs.
#[derive(Default, Clone)]
pub struct FakeGit {
    worktree_root: Option<std::path::PathBuf>,
    hash_objects: std::collections::HashMap<std::path::PathBuf, String>,
    last_commits: std::collections::HashMap<std::path::PathBuf, String>,
    shows: std::collections::HashMap<(String, std::path::PathBuf), Vec<u8>>,
    trees: std::collections::HashMap<String, Vec<String>>,
}

impl FakeGit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_worktree_root(mut self, root: impl Into<std::path::PathBuf>) -> Self {
        self.worktree_root = Some(root.into());
        self
    }

    pub fn with_hash_object(
        mut self,
        path: impl Into<std::path::PathBuf>,
        sha: impl Into<String>,
    ) -> Self {
        self.hash_objects.insert(path.into(), sha.into());
        self
    }

    pub fn with_last_commit(
        mut self,
        path: impl Into<std::path::PathBuf>,
        sha: impl Into<String>,
    ) -> Self {
        self.last_commits.insert(path.into(), sha.into());
        self
    }

    pub fn with_show(
        mut self,
        rev: impl Into<String>,
        path: impl Into<std::path::PathBuf>,
        bytes: impl Into<Vec<u8>>,
    ) -> Self {
        self.shows.insert((rev.into(), path.into()), bytes.into());
        self
    }

    pub fn with_tree(mut self, rev: impl Into<String>, paths: Vec<String>) -> Self {
        self.trees.insert(rev.into(), paths);
        self
    }
}

impl Git for FakeGit {
    fn worktree_root(&self, anchor: &Path) -> Result<std::path::PathBuf> {
        Ok(self
            .worktree_root
            .clone()
            .unwrap_or_else(|| anchor.to_path_buf()))
    }
    fn hash_object(&self, path: &Path) -> Result<String> {
        self.hash_objects
            .get(path)
            .cloned()
            .ok_or_else(|| OkfError::Io(format!("no fake hash-object for {}", path.display())))
    }

    fn last_commit(&self, path: &Path) -> Result<String> {
        self.last_commits
            .get(path)
            .cloned()
            .ok_or_else(|| OkfError::Io(format!("no fake last-commit for {}", path.display())))
    }

    fn show(&self, rev: &str, path: &Path) -> Result<Vec<u8>> {
        self.shows
            .get(&(rev.to_string(), path.to_path_buf()))
            .cloned()
            .ok_or_else(|| OkfError::Io(format!("no fake show for {rev}:{}", path.display())))
    }

    fn ls_tree(&self, rev: &str) -> Result<Vec<String>> {
        Ok(self.trees.get(rev).cloned().unwrap_or_default())
    }
}
