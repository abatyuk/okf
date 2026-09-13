//! Git effects — a thin wrapper over the `git` CLI (subprocess), not a git library.
//! Runtime-optional: only git-based source kinds and `okf diff` need it.
use crate::error::{OkfError, Result};
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub trait Git {
    /// Locate the worktree root containing `anchor`. Test ports default to the anchor itself.
    fn worktree_root(&self, anchor: &Path) -> Result<std::path::PathBuf> {
        Ok(anchor.to_path_buf())
    }
    /// `git hash-object <path>` — the blob object id (git-path fingerprint).
    fn hash_object(&self, path: &Path) -> Result<String>;
    /// `git hash-object <path>` when the worktree root is already known.
    ///
    /// The default preserves compatibility for test and third-party ports. Production overrides
    /// it to avoid rediscovering the same worktree for every source in a bundle.
    fn hash_object_in(&self, root: &Path, path: &Path) -> Result<String> {
        let _ = root;
        self.hash_object(path)
    }
    /// Hash several paths while reusing one Git operation when the port supports batching.
    fn hash_objects_in(&self, root: &Path, paths: &[PathBuf]) -> Vec<Result<String>> {
        paths
            .iter()
            .map(|path| self.hash_object_in(root, path))
            .collect()
    }
    /// `git log -1 --format=%H -- <path>` — last commit touching a path (git-commit).
    fn last_commit(&self, path: &Path) -> Result<String>;
    /// `git log -1 --format=%H -- <path>` when the worktree root is already known.
    fn last_commit_in(&self, root: &Path, path: &Path) -> Result<String> {
        let _ = root;
        self.last_commit(path)
    }
    /// Find the latest commit for several paths with one history traversal when supported.
    fn last_commits_in(&self, root: &Path, paths: &[PathBuf]) -> Vec<Result<String>> {
        paths
            .iter()
            .map(|path| self.last_commit_in(root, path))
            .collect()
    }
    /// `git show <rev>:<path>` — file contents at a ref (diff).
    fn show(&self, rev: &str, path: &Path) -> Result<Vec<u8>>;
    /// Read several blobs from one revision, preserving input order.
    fn show_many_in(&self, root: &Path, rev: &str, paths: &[PathBuf]) -> Result<Vec<Vec<u8>>> {
        let _ = root;
        paths.iter().map(|path| self.show(rev, path)).collect()
    }
    /// `git ls-tree -r --name-only <rev>` — every tracked path at a ref (diff).
    fn ls_tree(&self, rev: &str) -> Result<Vec<String>>;
    /// Enumerate tracked paths below a bundle prefix from a known worktree root.
    fn ls_tree_in(&self, root: &Path, rev: &str, prefix: &Path) -> Result<Vec<String>> {
        let _ = root;
        let paths = self.ls_tree(rev)?;
        if prefix.as_os_str().is_empty() {
            Ok(paths)
        } else {
            Ok(paths
                .into_iter()
                .filter(|path| Path::new(path).starts_with(prefix))
                .collect())
        }
    }
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
        let root =
            std::path::PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string());
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| OkfError::Environment(e.to_string()))?
                .join(path)
        };
        let relative = absolute.strip_prefix(&root).map_err(|_| {
            OkfError::Usage(format!(
                "git source {} is outside worktree {}",
                path.display(),
                root.display()
            ))
        })?;
        Ok((root, relative.to_path_buf()))
    }

    fn relative_to<'a>(root: &Path, path: &'a Path) -> Result<&'a Path> {
        if path.is_absolute() {
            path.strip_prefix(root).map_err(|_| {
                OkfError::Usage(format!(
                    "git source {} is outside worktree {}",
                    path.display(),
                    root.display()
                ))
            })
        } else {
            Ok(path)
        }
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
        self.hash_object_in(&root, &relative)
    }

    fn hash_object_in(&self, root: &Path, path: &Path) -> Result<String> {
        let relative = Self::relative_to(root, path)?;
        let out = Self::run(
            Command::new("git")
                .arg("-C")
                .arg(root)
                .arg("hash-object")
                .arg(relative),
        )?;
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    fn hash_objects_in(&self, root: &Path, paths: &[PathBuf]) -> Vec<Result<String>> {
        if paths.is_empty() {
            return Vec::new();
        }
        let mut results: Vec<Option<Result<String>>> = (0..paths.len()).map(|_| None).collect();
        let mut relative = Vec::new();
        for (index, path) in paths.iter().enumerate() {
            match Self::relative_to(root, path) {
                Ok(rel)
                    if !rel
                        .components()
                        .any(|component| matches!(component, std::path::Component::ParentDir))
                        && path.is_file() =>
                {
                    relative.push((index, rel))
                }
                Ok(_) => {
                    results[index] = Some(Err(OkfError::Io(format!(
                        "cannot hash missing or out-of-worktree path {}",
                        path.display()
                    ))));
                }
                Err(error) => results[index] = Some(Err(error)),
            }
        }
        if relative.is_empty() {
            return results.into_iter().map(Option::unwrap).collect();
        }
        let mut child = match Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["hash-object", "--stdin-paths"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                for (index, _) in relative {
                    results[index] = Some(Err(OkfError::Io(error.to_string())));
                }
                return results.into_iter().map(Option::unwrap).collect();
            }
        };
        let mut stdin = child.stdin.take().expect("piped git stdin");
        let requests: Vec<String> = relative
            .iter()
            .map(|(_, path)| format!("{}\n", path.to_string_lossy()))
            .collect();
        let writer = std::thread::spawn(move || -> std::io::Result<()> {
            for request in requests {
                stdin.write_all(request.as_bytes())?;
            }
            Ok(())
        });
        let output = match child.wait_with_output() {
            Ok(output) if output.status.success() => output,
            Ok(output) => {
                let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
                for (index, _) in relative {
                    results[index] = Some(Err(OkfError::Io(format!("git failed: {error}"))));
                }
                return results.into_iter().map(Option::unwrap).collect();
            }
            Err(error) => {
                for (index, _) in relative {
                    results[index] = Some(Err(OkfError::Io(error.to_string())));
                }
                return results.into_iter().map(Option::unwrap).collect();
            }
        };
        match writer.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                for (index, _) in relative {
                    results[index] = Some(Err(OkfError::Io(error.to_string())));
                }
                return results.into_iter().map(Option::unwrap).collect();
            }
            Err(_) => {
                for (index, _) in relative {
                    results[index] = Some(Err(OkfError::Internal(
                        "git hash-object writer panicked".to_string(),
                    )));
                }
                return results.into_iter().map(Option::unwrap).collect();
            }
        }
        let hashes: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect();
        if hashes.len() != relative.len() {
            let error = OkfError::Internal(format!(
                "git hash-object returned {} hashes for {} paths",
                hashes.len(),
                relative.len()
            ));
            for (index, _) in relative {
                results[index] = Some(Err(OkfError::Internal(error.to_string())));
            }
            return results.into_iter().map(Option::unwrap).collect();
        }
        for ((index, _), hash) in relative.into_iter().zip(hashes) {
            results[index] = Some(Ok(hash));
        }
        results.into_iter().map(Option::unwrap).collect()
    }

    fn last_commit(&self, path: &Path) -> Result<String> {
        let (root, relative) = Self::repo_path(path)?;
        self.last_commit_in(&root, &relative)
    }

    fn last_commit_in(&self, root: &Path, path: &Path) -> Result<String> {
        let relative = Self::relative_to(root, path)?;
        let out = Self::run(
            Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["log", "-1", "--format=%H", "--"])
                .arg(relative),
        )?;
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    fn last_commits_in(&self, root: &Path, paths: &[PathBuf]) -> Vec<Result<String>> {
        if paths.is_empty() {
            return Vec::new();
        }
        let mut results: Vec<Option<Result<String>>> = (0..paths.len()).map(|_| None).collect();
        let mut relative = Vec::new();
        for (index, path) in paths.iter().enumerate() {
            match Self::relative_to(root, path) {
                Ok(rel)
                    if !rel
                        .components()
                        .any(|component| matches!(component, std::path::Component::ParentDir)) =>
                {
                    relative.push((index, rel))
                }
                Ok(_) => results[index] = Some(self.last_commit_in(root, path)),
                Err(error) => results[index] = Some(Err(error)),
            }
        }
        if relative.is_empty() {
            return results.into_iter().map(Option::unwrap).collect();
        }
        let output = Self::run(
            Command::new("git")
                .arg("-C")
                .arg(root)
                .args([
                    "log",
                    "--format=%x1e%H",
                    "--name-only",
                    "--no-renames",
                    "--",
                ])
                .args(relative.iter().map(|(_, path)| path)),
        );
        let output = match output {
            Ok(output) => output,
            Err(error) => {
                for (index, _) in relative {
                    results[index] = Some(Err(OkfError::Io(error.to_string())));
                }
                return results.into_iter().map(Option::unwrap).collect();
            }
        };

        let mut wanted: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();
        for (index, path) in &relative {
            let key = path
                .to_string_lossy()
                .replace('\\', "/")
                .trim_start_matches("./")
                .to_string();
            wanted.entry(key).or_default().push(*index);
        }
        let mut commits: std::collections::HashMap<usize, String> =
            std::collections::HashMap::new();
        let mut commit = "";
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if let Some(hash) = line.strip_prefix('\u{1e}') {
                commit = hash.trim();
            } else if !commit.is_empty() {
                if let Some(indices) = wanted.get(line.trim()) {
                    for index in indices {
                        commits.entry(*index).or_insert_with(|| commit.to_string());
                    }
                }
            }
        }
        for (index, _) in relative {
            results[index] = Some(Ok(commits.remove(&index).unwrap_or_default()));
        }
        results.into_iter().map(Option::unwrap).collect()
    }

    fn show(&self, rev: &str, path: &Path) -> Result<Vec<u8>> {
        let spec = format!("{rev}:{}", path.display());
        let out = Self::run(Command::new("git").arg("show").arg(&spec))?;
        Ok(out.stdout)
    }

    fn show_many_in(&self, root: &Path, rev: &str, paths: &[PathBuf]) -> Result<Vec<Vec<u8>>> {
        if paths.is_empty() {
            return Ok(Vec::new());
        }
        let mut child = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| OkfError::Io(error.to_string()))?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| OkfError::Internal("git cat-file stdin unavailable".to_string()))?;
        let requests: Vec<String> = paths
            .iter()
            .map(|path| format!("{rev}:{}\n", path.to_string_lossy()))
            .collect();
        let writer = std::thread::spawn(move || -> std::io::Result<()> {
            for request in requests {
                stdin.write_all(request.as_bytes())?;
            }
            Ok(())
        });

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| OkfError::Internal("git cat-file stdout unavailable".to_string()))?;
        let mut reader = std::io::BufReader::new(stdout);
        let mut blobs = Vec::with_capacity(paths.len());
        for path in paths {
            let mut header = String::new();
            reader.read_line(&mut header)?;
            if header.trim_end().ends_with(" missing") {
                return Err(OkfError::Io(format!(
                    "git object missing at {rev}:{}",
                    path.display()
                )));
            }
            let size = header
                .split_whitespace()
                .nth(2)
                .and_then(|size| size.parse::<usize>().ok())
                .ok_or_else(|| {
                    OkfError::Io(format!(
                        "unexpected git cat-file response: {}",
                        header.trim()
                    ))
                })?;
            let mut bytes = vec![0; size];
            reader.read_exact(&mut bytes)?;
            let mut newline = [0; 1];
            reader.read_exact(&mut newline)?;
            blobs.push(bytes);
        }
        writer
            .join()
            .map_err(|_| OkfError::Internal("git cat-file writer panicked".to_string()))??;
        let status = child
            .wait()
            .map_err(|error| OkfError::Io(error.to_string()))?;
        if !status.success() {
            let mut stderr = String::new();
            if let Some(mut pipe) = child.stderr.take() {
                let _ = pipe.read_to_string(&mut stderr);
            }
            return Err(OkfError::Io(format!("git failed: {}", stderr.trim())));
        }
        Ok(blobs)
    }

    fn ls_tree(&self, rev: &str) -> Result<Vec<String>> {
        let out = Self::run(Command::new("git").args(["ls-tree", "-r", "--name-only", rev]))?;
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect())
    }

    fn ls_tree_in(&self, root: &Path, rev: &str, prefix: &Path) -> Result<Vec<String>> {
        let mut command = Command::new("git");
        command
            .arg("-C")
            .arg(root)
            .args(["ls-tree", "-r", "--name-only", rev]);
        if !prefix.as_os_str().is_empty() {
            command.arg("--").arg(prefix);
        }
        let out = Self::run(&mut command)?;
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
