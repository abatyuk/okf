//! git-commit and git-path fingerprints via the `git` CLI (through `ports::Git`).
use crate::error::Result;
use crate::model::source::Fingerprint;
use crate::ports::git::Git;
use std::path::Path;

/// `git-path`: git's own blob object id (`git hash-object <path>`). Git-native, deterministic,
/// respects `.gitattributes` EOL rules. Recorded under `blob_sha`.
pub fn git_path_fp(git: &dyn Git, path: &Path) -> Result<Fingerprint> {
    let sha = git.hash_object(path)?;
    Ok(Fingerprint::from_pairs(vec![("blob_sha".to_string(), sha)]))
}

/// `git-commit`: last commit touching the path (`git log -1 --format=%H -- <path>`).
/// Recorded under `commit_sha`. No hashing.
pub fn git_commit_fp(git: &dyn Git, path: &Path) -> Result<Fingerprint> {
    let sha = git.last_commit(path)?;
    Ok(Fingerprint::from_pairs(vec![(
        "commit_sha".to_string(),
        sha,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::git::FakeGit;

    #[test]
    fn git_path_reads_blob_sha() {
        let git = FakeGit::new().with_hash_object("src/x.py", "3f9a1c");
        let fp = git_path_fp(&git, Path::new("src/x.py")).unwrap();
        assert_eq!(fp.get("blob_sha"), Some("3f9a1c"));
    }

    #[test]
    fn git_commit_reads_commit_sha() {
        let git = FakeGit::new().with_last_commit("src/x.py", "deadbeef");
        let fp = git_commit_fp(&git, Path::new("src/x.py")).unwrap();
        assert_eq!(fp.get("commit_sha"), Some("deadbeef"));
    }
}
