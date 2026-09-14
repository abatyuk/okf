//! Resolve the bundle location: arg → `$OKF_BUNDLE` → config file → cwd.
//!
//! Precedence, highest first: an explicit argument, then the `OKF_BUNDLE` env var (a
//! session-level override), then the `bundle` key of the nearest `okf.toml` (a project
//! default), then the current directory.
use crate::bundle::config;
use crate::error::{OkfError, Result};
use std::path::{Path, PathBuf};

/// Environment variable that names a default bundle directory.
pub const ENV_BUNDLE: &str = "OKF_BUNDLE";

/// Resolve the bundle directory to operate on.
pub fn resolve_bundle(arg: Option<&str>) -> Result<PathBuf> {
    validated(resolve_bundle_target(arg)?)
}

/// Resolve a bundle target without requiring it to exist. This uses the same precedence as
/// [`resolve_bundle`] and is intended for creation commands such as `okf init`.
pub fn resolve_bundle_target(arg: Option<&str>) -> Result<PathBuf> {
    let env = std::env::var(ENV_BUNDLE).ok().filter(|s| !s.is_empty());
    let cwd = std::env::current_dir()
        .map_err(|e| OkfError::Environment(format!("cannot read current directory: {e}")))?;
    resolve_target_with(arg, env.as_deref(), &cwd)
}

/// Pure resolution core (no process globals) so precedence is unit-testable.
pub(crate) fn resolve_with(arg: Option<&str>, env: Option<&str>, cwd: &Path) -> Result<PathBuf> {
    validated(resolve_target_with(arg, env, cwd)?)
}

fn resolve_target_with(arg: Option<&str>, env: Option<&str>, cwd: &Path) -> Result<PathBuf> {
    if let Some(a) = arg {
        return Ok(PathBuf::from(a));
    }
    if let Some(e) = env {
        return Ok(PathBuf::from(e));
    }
    if let Some(bundle) = config::config_bundle(cwd)? {
        return Ok(bundle);
    }
    Ok(cwd.to_path_buf())
}

fn validated(path: PathBuf) -> Result<PathBuf> {
    if !path.exists() {
        return Err(OkfError::Environment(format!(
            "bundle path does not exist: {}",
            path.display()
        )));
    }
    if !path.is_dir() {
        return Err(OkfError::Environment(format!(
            "bundle path is not a directory: {}",
            path.display()
        )));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn arg_wins_over_everything() {
        let dir = tempdir();
        fs::write(dir.path().join("okf.toml"), "bundle = \"kb\"").unwrap();
        // arg points at the tempdir itself; config would point at a missing `kb`.
        let got = resolve_with(
            Some(dir.path().to_str().unwrap()),
            Some("/nope"),
            dir.path(),
        )
        .unwrap();
        assert_eq!(got, dir.path());
    }

    #[test]
    fn env_wins_over_config() {
        let dir = tempdir();
        let envdir = dir.path().join("envbundle");
        fs::create_dir(&envdir).unwrap();
        fs::write(dir.path().join("okf.toml"), "bundle = \".\"").unwrap();
        let got = resolve_with(None, Some(envdir.to_str().unwrap()), dir.path()).unwrap();
        assert_eq!(got, envdir);
    }

    #[test]
    fn config_used_when_no_arg_or_env() {
        let dir = tempdir();
        let kb = dir.path().join("docs").join("kb");
        fs::create_dir_all(&kb).unwrap();
        fs::write(dir.path().join("okf.toml"), "bundle = \"docs/kb\"").unwrap();
        let got = resolve_with(None, None, dir.path()).unwrap();
        assert_eq!(got, dir.path().join("docs/kb"));
    }

    #[test]
    fn target_resolution_allows_a_missing_configured_bundle() {
        let dir = tempdir();
        fs::write(dir.path().join("okf.toml"), "bundle = \"new/kb\"").unwrap();
        let got = resolve_target_with(None, None, dir.path()).unwrap();
        assert_eq!(got, dir.path().join("new/kb"));
    }

    #[test]
    fn config_found_by_walking_up() {
        let dir = tempdir();
        fs::write(dir.path().join("okf.toml"), "bundle = \".\"").unwrap();
        let nested = dir.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        // cwd is nested; config is at the root of the tempdir.
        let got = resolve_with(None, None, &nested).unwrap();
        assert_eq!(got, dir.path().to_path_buf());
    }

    #[test]
    fn falls_back_to_cwd_without_config() {
        let dir = tempdir();
        let got = resolve_with(None, None, dir.path()).unwrap();
        assert_eq!(got, dir.path());
    }

    #[test]
    fn malformed_config_is_usage_error() {
        let dir = tempdir();
        fs::write(dir.path().join("okf.toml"), "bundle = [not valid").unwrap();
        let err = resolve_with(None, None, dir.path()).unwrap_err();
        assert_eq!(err.exit_code(), 2);
    }

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }
}
