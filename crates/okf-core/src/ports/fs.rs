//! Filesystem effects.
use crate::error::{OkfError, Result};
use std::path::Path;

pub trait FileSystem {
    fn read(&self, path: &Path) -> Result<Vec<u8>>;
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<()>;
}

/// Production filesystem, backed by `std::fs`.
pub struct RealFs;

impl FileSystem for RealFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        std::fs::read(path).map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        std::fs::write(path, bytes).map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))
    }
}

/// In-memory filesystem for hermetic tests.
#[derive(Default, Clone)]
pub struct FakeFs {
    files: std::collections::HashMap<std::path::PathBuf, Vec<u8>>,
}

impl FakeFs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder-style seed of a file.
    pub fn with_file(mut self, path: impl Into<std::path::PathBuf>, bytes: impl Into<Vec<u8>>) -> Self {
        self.files.insert(path.into(), bytes.into());
        self
    }

    pub fn insert(&mut self, path: impl Into<std::path::PathBuf>, bytes: impl Into<Vec<u8>>) {
        self.files.insert(path.into(), bytes.into());
    }
}

impl FileSystem for FakeFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| OkfError::Io(format!("no such file (fake): {}", path.display())))
    }

    fn write(&self, _path: &Path, _bytes: &[u8]) -> Result<()> {
        // Fakes are read-only for fingerprinting; writes are a no-op success.
        Ok(())
    }
}
