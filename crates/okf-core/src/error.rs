//! Error type and its mapping to CLI exit classes.
use thiserror::Error;

pub type Result<T> = std::result::Result<T, OkfError>;

/// All fallible operations in okf-core return this. Findings-driven exit codes
/// (0 vs 1) are decided by the CLI, not here.
#[derive(Debug, Clone, Error)]
pub enum OkfError {
    /// Bad usage / arguments (exit 2).
    #[error("usage error: {0}")]
    Usage(String),
    /// Environment problem — missing/unreadable bundle or ontology (exit 3).
    #[error("environment error: {0}")]
    Environment(String),
    /// I/O failure (exit 3).
    #[error("I/O error: {0}")]
    Io(String),
    /// YAML frontmatter/ontology parse failure (exit 3).
    #[error("YAML parse error: {0}")]
    Yaml(String),
    /// Internal invariant broken (exit 4).
    #[error("internal error: {0}")]
    Internal(String),
}

impl OkfError {
    /// Add the failing file without changing the error's exit class.
    pub fn at_path(self, path: &std::path::Path) -> Self {
        let context = |message| format!("{}: {message}", path.display());
        match self {
            Self::Usage(message) => Self::Usage(context(message)),
            Self::Environment(message) => Self::Environment(context(message)),
            Self::Io(message) => Self::Io(context(message)),
            Self::Yaml(message) => Self::Yaml(context(message)),
            Self::Internal(message) => Self::Internal(context(message)),
        }
    }

    /// Combine YAML failures from the files actually examined during a bundle load.
    pub fn yaml_files(messages: Vec<String>) -> Self {
        Self::Yaml(if messages.len() == 1 {
            messages.into_iter().next().unwrap()
        } else {
            format!(
                "{} concept files failed to parse:\n{}",
                messages.len(),
                messages.join("\n")
            )
        })
    }

    /// Exit class per ARCHITECTURE.md / INTENT.md.
    pub fn exit_code(&self) -> i32 {
        match self {
            OkfError::Usage(_) => 2,
            OkfError::Environment(_) | OkfError::Io(_) | OkfError::Yaml(_) => 3,
            OkfError::Internal(_) => 4,
        }
    }
}

impl From<std::io::Error> for OkfError {
    fn from(e: std::io::Error) -> Self {
        OkfError::Io(e.to_string())
    }
}
