//! Source fingerprinting, dispatched on `SourceKind`.
//!
//! [`Engine`] wires the effect ports (fs / git / net) and a base directory, and computes the
//! *current* fingerprint for a source per ARCHITECTURE.md's canonicalization rules. Compare
//! it against a source's recorded [`Fingerprint`] to detect drift (the `stale` wave).
pub mod canonicalize;
pub mod git;
pub mod text;
pub mod url;

use crate::error::{OkfError, Result};
use crate::model::source::{Fingerprint, Source, SourceKind};
use crate::ports::fs::FileSystem;
use crate::ports::git::Git;
use crate::ports::net::Net;
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

type FileCache = HashMap<PathBuf, std::result::Result<Arc<[u8]>, String>>;

/// Computes the current fingerprint for a source, dispatching on its kind.
pub trait Fingerprinter {
    fn fingerprint(&self, source: &Source) -> Result<Fingerprint>;
}

/// A fingerprinting engine over the effect ports. `root` is the bundle base for file/text
/// sources; git sources resolve from the worktree root. `net` is optional (only URLs need it).
pub struct Engine<'a> {
    pub root: &'a Path,
    pub fs: &'a dyn FileSystem,
    pub git: &'a dyn Git,
    pub net: Option<&'a dyn Net>,
    git_root: OnceCell<PathBuf>,
    files: RefCell<FileCache>,
}

impl<'a> Engine<'a> {
    /// Build an engine. Pass `None` for `net` when `url-sources` are not in play.
    pub fn new(
        root: &'a Path,
        fs: &'a dyn FileSystem,
        git: &'a dyn Git,
        net: Option<&'a dyn Net>,
    ) -> Self {
        Self {
            root,
            fs,
            git,
            net,
            git_root: OnceCell::new(),
            files: RefCell::new(HashMap::new()),
        }
    }

    fn resolve(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    fn read(&self, path: PathBuf) -> Result<Arc<[u8]>> {
        if let Some(cached) = self.files.borrow().get(&path) {
            return cached
                .clone()
                .map_err(|message| OkfError::Io(message.to_string()));
        }
        let result = self
            .fs
            .read(&path)
            .map(Arc::<[u8]>::from)
            .map_err(|error| error.to_string());
        self.files.borrow_mut().insert(path, result.clone());
        result.map_err(OkfError::Io)
    }

    /// Resolve the containing worktree once for the lifetime of this engine. Bundle-wide
    /// operations may fingerprint hundreds of sources from the same repository.
    fn git_root(&self) -> Result<&Path> {
        if self.git_root.get().is_none() {
            let root = self.git.worktree_root(self.root)?;
            // This engine is single-threaded (`OnceCell` is intentionally unsynchronized), so a
            // failed set would indicate an internal programming error.
            self.git_root.set(root).map_err(|_| {
                OkfError::Internal("git worktree root initialized twice".to_string())
            })?;
        }
        Ok(self
            .git_root
            .get()
            .expect("git worktree root was initialized")
            .as_path())
    }

    pub(crate) fn fingerprint_git_paths(&self, sources: &[Source]) -> Vec<Result<Fingerprint>> {
        let root = match self.git_root() {
            Ok(root) => root,
            Err(error) => {
                let message = error.to_string();
                return sources
                    .iter()
                    .map(|_| Err(OkfError::Io(message.clone())))
                    .collect();
            }
        };
        let paths: Vec<PathBuf> = sources
            .iter()
            .map(|source| root.join(split_fragment(&source.resource).0))
            .collect();
        self.git
            .hash_objects_in(root, &paths)
            .into_iter()
            .map(|result| {
                result.map(|sha| Fingerprint::from_pairs(vec![("blob_sha".to_string(), sha)]))
            })
            .collect()
    }

    pub(crate) fn fingerprint_git_commits(&self, sources: &[Source]) -> Vec<Result<Fingerprint>> {
        let root = match self.git_root() {
            Ok(root) => root,
            Err(error) => {
                let message = error.to_string();
                return sources
                    .iter()
                    .map(|_| Err(OkfError::Io(message.clone())))
                    .collect();
            }
        };
        let paths: Vec<PathBuf> = sources
            .iter()
            .map(|source| root.join(split_fragment(&source.resource).0))
            .collect();
        self.git
            .last_commits_in(root, &paths)
            .into_iter()
            .map(|result| {
                result.map(|sha| Fingerprint::from_pairs(vec![("commit_sha".to_string(), sha)]))
            })
            .collect()
    }
}

impl Fingerprinter for Engine<'_> {
    fn fingerprint(&self, source: &Source) -> Result<Fingerprint> {
        let (path_part, fragment) = split_fragment(&source.resource);
        match &source.kind {
            SourceKind::GitPath => {
                let git_root = self.git_root()?;
                git::git_path_fp_in(self.git, git_root, &git_root.join(path_part))
            }
            SourceKind::GitCommit => {
                let git_root = self.git_root()?;
                git::git_commit_fp_in(self.git, git_root, &git_root.join(path_part))
            }
            SourceKind::File => {
                let bytes = self.read(self.resolve(path_part))?;
                Ok(Fingerprint::from_pairs(vec![(
                    "sha256".to_string(),
                    canonicalize::sha256_hex(&bytes),
                )]))
            }
            SourceKind::LineRange => {
                let frag = fragment.ok_or_else(|| {
                    OkfError::Usage(format!(
                        "line-range source needs a `#Lstart-Lend` fragment: {}",
                        source.resource
                    ))
                })?;
                let (start, end) = parse_line_range(frag)?;
                let bytes = self.read(self.resolve(path_part))?;
                text::line_range_fp(&bytes, start, end)
            }
            SourceKind::MarkdownHeading => {
                let slug = fragment.ok_or_else(|| {
                    OkfError::Usage(format!(
                        "markdown-heading source needs a `#slug` fragment: {}",
                        source.resource
                    ))
                })?;
                let bytes = self.read(self.resolve(path_part))?;
                text::markdown_heading_fp(&bytes, slug)
            }
            SourceKind::Url => url::url_fp(self.net, &source.resource),
            SourceKind::Other(k) => Err(OkfError::Usage(format!(
                "no fingerprint rule for source kind `{k}`"
            ))),
        }
    }
}

/// Split a resource into its path/URL part and optional `#fragment`.
fn split_fragment(resource: &str) -> (&str, Option<&str>) {
    match resource.split_once('#') {
        Some((path, frag)) => (path, Some(frag)),
        None => (resource, None),
    }
}

/// Parse a line-range fragment like `L40-88`, `L40-L88`, or `L40` (single line).
fn parse_line_range(fragment: &str) -> Result<(usize, usize)> {
    let parse_num = |s: &str| -> Result<usize> {
        s.trim()
            .trim_start_matches(['L', 'l'])
            .parse::<usize>()
            .map_err(|_| OkfError::Usage(format!("bad line-range fragment: {fragment}")))
    };
    match fragment.split_once('-') {
        Some((a, b)) => Ok((parse_num(a)?, parse_num(b)?)),
        None => {
            let n = parse_num(fragment)?;
            Ok((n, n))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::source::Fingerprint as Fp;
    use crate::ports::fs::FakeFs;
    use crate::ports::git::FakeGit;
    use crate::ports::net::FakeNet;

    fn src(resource: &str, kind: SourceKind) -> Source {
        Source {
            resource: resource.to_string(),
            kind,
            fingerprint: Fp::default(),
            extra: Default::default(),
        }
    }

    #[test]
    fn dispatches_file_sha256() {
        let fs = FakeFs::new().with_file("data/raw.bin", b"abc".to_vec());
        let git = FakeGit::new();
        let net = FakeNet::new();
        let eng = Engine::new(Path::new(""), &fs, &git, Some(&net));
        let fp = eng
            .fingerprint(&src("data/raw.bin", SourceKind::File))
            .unwrap();
        assert_eq!(
            fp.get("sha256"),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
    }

    #[test]
    fn dispatches_git_path() {
        let fs = FakeFs::new();
        let git = FakeGit::new().with_hash_object("src/x.py", "3f9a1c");
        let eng = Engine::new(Path::new(""), &fs, &git, None);
        let fp = eng
            .fingerprint(&src("src/x.py", SourceKind::GitPath))
            .unwrap();
        assert_eq!(fp.get("blob_sha"), Some("3f9a1c"));
    }

    #[test]
    fn git_paths_are_relative_to_worktree_not_bundle() {
        let fs = FakeFs::new();
        let git = FakeGit::new()
            .with_worktree_root("repo")
            .with_hash_object("repo/docs/x.md", "abc123");
        let eng = Engine::new(Path::new("repo/knowledge"), &fs, &git, None);
        let fp = eng
            .fingerprint(&src("docs/x.md", SourceKind::GitPath))
            .unwrap();
        assert_eq!(fp.get("blob_sha"), Some("abc123"));
    }

    #[test]
    fn dispatches_line_range_with_fragment() {
        let fs = FakeFs::new().with_file("src/x.py", b"a\nb\nc\nd\n".to_vec());
        let git = FakeGit::new();
        let eng = Engine::new(Path::new(""), &fs, &git, None);
        let fp = eng
            .fingerprint(&src("src/x.py#L2-3", SourceKind::LineRange))
            .unwrap();
        let expect = canonicalize::canonical_sha256("b\nc");
        assert_eq!(fp.get("content_sha256"), Some(expect.as_str()));
    }

    #[test]
    fn dispatches_markdown_heading_with_fragment() {
        let doc = "## Rates\n\nbody\n\n## Next\n";
        let fs = FakeFs::new().with_file("docs/p.md", doc.as_bytes().to_vec());
        let git = FakeGit::new();
        let eng = Engine::new(Path::new(""), &fs, &git, None);
        let fp = eng
            .fingerprint(&src("docs/p.md#rates", SourceKind::MarkdownHeading))
            .unwrap();
        assert_eq!(
            fp.get("section_sha256"),
            Some(canonicalize::canonical_sha256("body").as_str())
        );
    }

    #[test]
    fn unknown_kind_errors() {
        let fs = FakeFs::new();
        let git = FakeGit::new();
        let eng = Engine::new(Path::new(""), &fs, &git, None);
        assert!(eng
            .fingerprint(&src("x", SourceKind::Other("sql".to_string())))
            .is_err());
    }

    #[test]
    fn root_is_joined() {
        let fs = FakeFs::new().with_file("repo/data.txt", b"abc".to_vec());
        let git = FakeGit::new();
        let eng = Engine::new(Path::new("repo"), &fs, &git, None);
        let fp = eng.fingerprint(&src("data.txt", SourceKind::File)).unwrap();
        assert_eq!(
            fp.get("sha256"),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
    }

    #[test]
    fn parse_line_range_forms() {
        assert_eq!(parse_line_range("L40-88").unwrap(), (40, 88));
        assert_eq!(parse_line_range("L40-L88").unwrap(), (40, 88));
        assert_eq!(parse_line_range("40").unwrap(), (40, 40));
        assert!(parse_line_range("Lx").is_err());
    }
}
