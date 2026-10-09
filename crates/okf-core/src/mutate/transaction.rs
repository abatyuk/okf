//! Recoverable publication of a prevalidated set of files within one bundle.
//!
//! Each file is atomically replaced; the set is journaled, not a filesystem-wide atomic
//! snapshot. Cooperating writers refuse a pending journal. Recovery rolls back only when
//! every file still matches its before/after image, never overwriting intervening edits.
use crate::error::{OkfError, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

pub const JOURNAL: &str = ".okf-transaction";
pub const LOCK: &str = ".okf-transaction.lock";

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    changes: Vec<FileChange>,
    permissions: std::collections::BTreeMap<PathBuf, SavedPermissions>,
    created_dirs: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedPermissions {
    readonly: bool,
    #[cfg(unix)]
    mode: u32,
}
impl SavedPermissions {
    fn capture(path: &Path) -> Result<Self> {
        let permissions = std::fs::metadata(path)?.permissions();
        Ok(Self {
            readonly: permissions.readonly(),
            #[cfg(unix)]
            mode: {
                use std::os::unix::fs::PermissionsExt;
                permissions.mode()
            },
        })
    }
    fn restore(&self, path: &Path) -> Result<()> {
        let mut permissions = std::fs::metadata(path)?.permissions();
        permissions.set_readonly(self.readonly);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            permissions.set_mode(self.mode);
        }
        std::fs::set_permissions(path, permissions)?;
        std::fs::File::open(path)?.sync_all()?;
        Ok(())
    }
}

// This stable inode is never unlinked: removing a lock file allows another caller
// to lock a different inode while an earlier holder still owns the old one.
/// Unlock explicitly before closing the descriptor: another thread can fork while a
/// lock is held, temporarily retaining the descriptor until that child execs.
#[derive(Debug)]
pub struct WriterGuard(std::fs::File);
impl Drop for WriterGuard {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

fn acquire_lock(root: &Path) -> Result<WriterGuard> {
    let path = root.join(LOCK);
    let file = match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let meta = std::fs::symlink_metadata(&path)?;
            if !meta.is_file() || meta.file_type().is_symlink() {
                return Err(usage("transaction lock must be a regular file"));
            }
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)?
        }
        Err(e) => return Err(e.into()),
    };
    file.try_lock().map_err(|e| {
        usage(format!(
            "transaction is still active; operation refused: {e}"
        ))
    })?;
    Ok(WriterGuard(file))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileChange {
    pub path: PathBuf,
    pub before: Option<Vec<u8>>,
    pub after: Option<Vec<u8>>,
}

fn usage(message: impl Into<String>) -> OkfError {
    OkfError::Usage(message.into())
}

/// Reject symlink components, special files, traversal, and reserved transaction paths.
pub fn checked_path(root: &Path, relative: &Path) -> Result<PathBuf> {
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(usage(format!(
            "transaction path must be relative without traversal: {}",
            relative.display()
        )));
    }
    if relative
        .components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
    {
        return Err(usage(format!(
            "transaction cannot edit hidden/control path: {}",
            relative.display()
        )));
    }
    let mut path = root.to_path_buf();
    let count = relative.components().count();
    for (i, component) in relative.components().enumerate() {
        path.push(component);
        match std::fs::symlink_metadata(&path) {
            Ok(m)
                if m.file_type().is_symlink()
                    || (i + 1 < count && !m.is_dir())
                    || (i + 1 == count && !m.is_file()) =>
            {
                return Err(usage(format!(
                    "transaction requires regular files and real directories: {}",
                    path.display()
                )));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(path)
}

pub fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn sync_dir(path: &Path) -> Result<()> {
    std::fs::File::open(path)?.sync_all()?;
    Ok(())
}

fn publish(root: &Path, change: &FileChange, bytes: &Option<Vec<u8>>) -> Result<()> {
    let path = checked_path(root, &change.path)?;
    if let Some(bytes) = bytes {
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let (temp, mut file) = loop {
            let seq = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let temp = parent.join(format!(".okf-publish-{}-{seq}", std::process::id()));
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
            {
                Ok(f) => break (temp, f),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        };
        let result = (|| -> Result<()> {
            if let Ok(meta) = std::fs::metadata(&path) {
                file.set_permissions(meta.permissions())?;
            }
            file.write_all(bytes)?;
            file.sync_all()?;
            drop(file);
            std::fs::rename(&temp, &path)?;
            sync_dir(parent)
        })();
        let _ = std::fs::remove_file(&temp);
        result?;
    } else if path.exists() {
        std::fs::remove_file(&path)?;
        sync_dir(path.parent().unwrap())?;
    }
    Ok(())
}

pub fn ensure_idle(root: &Path) -> Result<()> {
    if std::fs::symlink_metadata(root.join(JOURNAL)).is_ok() {
        return Err(usage(
            "pending transaction; run `okf changeset recover` before writing",
        ));
    }
    Ok(())
}

/// Hold across an ordinary mutation's complete read/modify/write operation.
/// Callers must not invoke `commit` while holding this guard: transactional
/// operations acquire the same lock internally. An absent root is an error;
/// bundle initialization creates its root before acquiring this guard.
pub fn writer_lock(root: &Path) -> Result<WriterGuard> {
    let lock = acquire_lock(root)?;
    ensure_idle(root)?;
    Ok(lock)
}

/// Read-only publication checks shared by previews and commits. File/directory
/// type transitions are unsupported, even if another change removes the blocker.
pub fn preflight(root: &Path, changes: &[FileChange]) -> Result<()> {
    ensure_idle(root)?;
    let mut paths = std::collections::BTreeSet::new();
    for c in changes {
        if !paths.insert(c.path.clone()) {
            return Err(usage("duplicate transaction path"));
        }
        let path = checked_path(root, &c.path)?;
        if read_optional(&path)? != c.before {
            return Err(usage(format!(
                "file changed since planning: {}",
                c.path.display()
            )));
        }
    }
    Ok(())
}

/// Publish guarded before/after images. On a normal failure roll back; on interruption
/// retain the durable journal for an explicit `recover` invocation.
pub fn commit(root: &Path, changes: Vec<FileChange>) -> Result<()> {
    commit_with_hook(root, changes, &mut |_| Ok(()))
}

fn commit_with_hook(
    root: &Path,
    changes: Vec<FileChange>,
    after_publish: &mut dyn FnMut(usize) -> Result<()>,
) -> Result<()> {
    let _root_lock = acquire_lock(root)?;
    ensure_idle(root)?;
    let changes: Vec<_> = changes
        .into_iter()
        .filter(|c| c.before != c.after)
        .collect();
    if changes.is_empty() {
        return Ok(());
    }
    preflight(root, &changes)?;
    let mut permissions = std::collections::BTreeMap::new();
    let mut created_dirs = std::collections::BTreeSet::new();
    for change in &changes {
        if change.before.is_some() {
            permissions.insert(
                change.path.clone(),
                SavedPermissions::capture(&root.join(&change.path))?,
            );
        }
        if change.after.is_some() {
            let mut parent = change.path.parent();
            while let Some(path) = parent.filter(|p| !p.as_os_str().is_empty()) {
                if !root.join(path).exists() {
                    created_dirs.insert(path.to_path_buf());
                }
                parent = path.parent();
            }
        }
    }
    let manifest = Manifest {
        changes: changes.clone(),
        permissions,
        created_dirs: created_dirs.into_iter().collect(),
    };
    let journal = root.join(JOURNAL);
    std::fs::create_dir(&journal)
        .map_err(|e| usage(format!("cannot acquire transaction journal: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&journal, std::fs::Permissions::from_mode(0o700))?;
    }
    let lock = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(journal.join("lock"))?;
    lock.try_lock()
        .map_err(|e| usage(format!("transaction is active: {e}")))?;
    let setup = (|| -> Result<()> {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(journal.join("manifest.tmp"))?;
        serde_json::to_writer(&mut f, &manifest).map_err(|e| OkfError::Internal(e.to_string()))?;
        f.write_all(b"\n")?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(journal.join("manifest.tmp"), journal.join("manifest.json"))?;
        sync_dir(&journal)?;
        sync_dir(root)
    })();
    if let Err(e) = setup {
        let _ = std::fs::remove_dir_all(&journal);
        return Err(e);
    }
    let result = (|| -> Result<()> {
        // Recheck every expectation after acquiring the journal, before any publication.
        for c in &changes {
            if read_optional(&checked_path(root, &c.path)?)? != c.before {
                return Err(usage(format!(
                    "file changed since planning: {}",
                    c.path.display()
                )));
            }
        }
        for (index, c) in changes.iter().enumerate() {
            if read_optional(&checked_path(root, &c.path)?)? != c.before {
                return Err(usage(format!(
                    "concurrent file change: {}",
                    c.path.display()
                )));
            }
            publish(root, c, &c.after)?;
            after_publish(index)?;
        }
        // Persist directory entries for newly created ancestor chains before commit.
        for directory in manifest.created_dirs.iter().rev() {
            sync_dir(&root.join(directory))?;
        }
        sync_dir(root)?;
        Ok(())
    })();
    if let Err(error) = result {
        return match recover_locked(root) {
            Ok(_) => Err(error),
            Err(recovery) => Err(OkfError::Io(format!(
                "{error}; rollback requires attention: {recovery}; journal retained"
            ))),
        };
    }
    let marker = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(journal.join("committed"))?;
    marker.sync_all()?;
    sync_dir(&journal)?;
    cleanup(&journal)?;
    sync_dir(root)?;
    Ok(())
}

/// Restore before-images after interrupted publication, refusing intervening user edits.
pub fn recover(root: &Path) -> Result<Vec<PathBuf>> {
    let _root_lock = acquire_lock(root)?;
    let journal = root.join(JOURNAL);
    let meta = std::fs::symlink_metadata(&journal)?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err(usage("transaction journal must be a real directory"));
    }
    let lock_path = journal.join("lock");
    let _lock = match std::fs::symlink_metadata(&lock_path) {
        Ok(meta) => {
            if !meta.is_file() || meta.file_type().is_symlink() {
                return Err(usage("invalid transaction lock"));
            }
            let lock = std::fs::OpenOptions::new().write(true).open(lock_path)?;
            lock.try_lock().map_err(|e| {
                usage(format!(
                    "transaction is still active; recovery refused: {e}"
                ))
            })?;
            Some(lock)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
    };
    recover_locked(root)
}

fn recover_locked(root: &Path) -> Result<Vec<PathBuf>> {
    let journal = root.join(JOURNAL);
    let meta = std::fs::symlink_metadata(&journal)?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err(usage("transaction journal must be a real directory"));
    }
    if let Ok(marker) = std::fs::symlink_metadata(journal.join("committed")) {
        if !marker.is_file() || marker.file_type().is_symlink() {
            return Err(usage("invalid transaction commit marker"));
        }
        cleanup(&journal)?;
        sync_dir(root)?;
        return Ok(Vec::new());
    }
    let manifest = journal.join("manifest.json");
    let meta = match std::fs::symlink_metadata(&manifest) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Publication cannot begin until the complete manifest has been renamed.
            cleanup(&journal)?;
            sync_dir(root)?;
            return Ok(Vec::new());
        }
        Err(e) => return Err(e.into()),
    };
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err(usage("transaction manifest must be a regular file"));
    }
    let bytes = std::fs::read(&manifest)?;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum JournalData {
        Current(Manifest),
        Legacy(Vec<FileChange>),
    }
    let Manifest {
        changes,
        permissions,
        created_dirs,
    } = match serde_json::from_slice(&bytes)
        .map_err(|e| usage(format!("invalid recovery journal: {e}")))?
    {
        JournalData::Current(data) => data,
        JournalData::Legacy(changes) => Manifest {
            changes,
            permissions: Default::default(),
            created_dirs: Vec::new(),
        },
    };
    let mut paths = std::collections::BTreeSet::new();
    for c in &changes {
        if !paths.insert(c.path.clone()) {
            return Err(usage("duplicate recovery path"));
        }
        let current = read_optional(&checked_path(root, &c.path)?)?;
        if current != c.before && current != c.after {
            return Err(usage(format!(
                "recovery refuses intervening edit: {}",
                c.path.display()
            )));
        }
    }
    for directory in &created_dirs {
        checked_path(root, &directory.join("_okf_directory_check"))?;
    }
    for c in changes.iter().rev() {
        publish(root, c, &c.before)?;
        if c.before.is_some() {
            if let Some(saved) = permissions.get(&c.path) {
                saved.restore(&root.join(&c.path))?;
            }
        }
    }
    // Only remove directories created by this transaction, and only while empty.
    let mut created_dirs = created_dirs;
    created_dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    for directory in created_dirs {
        // Validate directory components using a synthetic regular-file child.
        checked_path(root, &directory.join("_okf_directory_check"))?;
        match std::fs::remove_dir(root.join(&directory)) {
            Ok(()) => {}
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
                ) => {}
            Err(e) => return Err(e.into()),
        }
    }
    cleanup(&journal)?;
    sync_dir(root)?;
    Ok(changes.into_iter().map(|c| c.path).collect())
}

fn cleanup(journal: &Path) -> Result<()> {
    for name in ["manifest.tmp", "manifest.json", "committed", "lock"] {
        match std::fs::remove_file(journal.join(name)) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    std::fs::remove_dir(journal)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn change(path: &str, before: Option<&str>, after: Option<&str>) -> FileChange {
        FileChange {
            path: path.into(),
            before: before.map(|s| s.as_bytes().to_vec()),
            after: after.map(|s| s.as_bytes().to_vec()),
        }
    }

    #[test]
    fn injected_publish_failure_restores_deleted_files_and_created_directories() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("old.txt"), "old bytes").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                root.path().join("old.txt"),
                fs::Permissions::from_mode(0o640),
            )
            .unwrap();
        }
        let mut published = 0;
        let error = commit_with_hook(
            root.path(),
            vec![
                change("old.txt", Some("old bytes"), None),
                change("new/deep/file.txt", None, Some("created")),
            ],
            &mut |index| {
                published += 1;
                if index == 1 {
                    Err(OkfError::Io("injected publication failure".into()))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("injected publication failure"));
        assert_eq!(published, 2);
        assert_eq!(
            fs::read_to_string(root.path().join("old.txt")).unwrap(),
            "old bytes"
        );
        assert!(!root.path().join("new").exists());
        assert!(!root.path().join(JOURNAL).exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(root.path().join("old.txt"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o640
            );
        }
    }

    #[test]
    fn recovery_and_competing_commit_are_refused_during_publication() {
        let root = tempfile::tempdir().unwrap();
        commit_with_hook(
            root.path(),
            vec![change("new.txt", None, Some("new"))],
            &mut |_| {
                assert!(recover(root.path())
                    .unwrap_err()
                    .to_string()
                    .contains("active"));
                assert!(
                    commit(root.path(), vec![change("other.txt", None, Some("other"))]).is_err()
                );
                assert_eq!(
                    fs::read_to_string(root.path().join("new.txt")).unwrap(),
                    "new"
                );
                Ok(())
            },
        )
        .unwrap();
        assert!(!root.path().join("other.txt").exists());
        assert!(!root.path().join(JOURNAL).exists());
    }

    #[test]
    fn recovery_cannot_remove_an_initializing_journal() {
        let root = tempfile::tempdir().unwrap();
        let lock = acquire_lock(root.path()).unwrap();
        fs::create_dir(root.path().join(JOURNAL)).unwrap();
        assert!(recover(root.path()).is_err());
        assert!(root.path().join(JOURNAL).exists());
        drop(lock);
        recover(root.path()).unwrap();
        assert!(!root.path().join(JOURNAL).exists());
    }

    #[test]
    fn incomplete_initialization_is_recoverable_without_touching_bundle_bytes() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("existing.txt"), "keep").unwrap();
        let journal = root.path().join(JOURNAL);
        fs::create_dir(&journal).unwrap();
        fs::write(journal.join("manifest.tmp"), "{partial").unwrap();
        recover(root.path()).unwrap();
        assert_eq!(
            fs::read_to_string(root.path().join("existing.txt")).unwrap(),
            "keep"
        );
        assert!(!journal.exists());
    }

    #[test]
    fn committed_cleanup_preserves_published_and_subsequently_edited_bytes() {
        let root = tempfile::tempdir().unwrap();
        let journal = root.path().join(JOURNAL);
        fs::create_dir(&journal).unwrap();
        fs::write(
            journal.join("manifest.json"),
            serde_json::to_vec(&vec![change("a.txt", Some("old"), Some("new"))]).unwrap(),
        )
        .unwrap();
        fs::write(journal.join("committed"), "").unwrap();
        fs::write(root.path().join("a.txt"), "subsequent edit").unwrap();
        assert!(recover(root.path()).unwrap().is_empty());
        assert_eq!(
            fs::read_to_string(root.path().join("a.txt")).unwrap(),
            "subsequent edit"
        );
        assert!(!journal.exists());
    }

    #[test]
    fn ordinary_writer_guard_excludes_transactions_and_other_writers() {
        let root = tempfile::tempdir().unwrap();
        let guard = writer_lock(root.path()).unwrap();
        assert!(writer_lock(root.path()).is_err());
        assert!(commit(root.path(), vec![change("new.txt", None, Some("new"))]).is_err());
        assert!(!root.path().join("new.txt").exists());
        drop(guard);
        commit(root.path(), vec![change("new.txt", None, Some("new"))]).unwrap();
    }

    #[test]
    fn dropping_guard_unlocks_even_if_a_duplicate_descriptor_remains() {
        let root = tempfile::tempdir().unwrap();
        let guard = writer_lock(root.path()).unwrap();
        let duplicate = guard.0.try_clone().unwrap();
        drop(guard);
        commit(root.path(), vec![change("new.txt", None, Some("new"))]).unwrap();
        drop(duplicate);
    }

    #[test]
    fn ordinary_writer_guard_refuses_pending_journal_and_absent_root() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join(JOURNAL)).unwrap();
        assert!(writer_lock(root.path())
            .unwrap_err()
            .to_string()
            .contains("pending transaction"));
        recover(root.path()).unwrap();
        assert!(writer_lock(root.path()).is_ok());
        let absent = root.path().join("absent");
        assert!(writer_lock(&absent).is_err());
        assert!(!absent.exists());
    }

    #[test]
    fn stale_before_image_never_creates_a_journal() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("a.txt"), "current").unwrap();
        assert!(commit(
            root.path(),
            vec![change("a.txt", Some("stale"), Some("new"))]
        )
        .is_err());
        assert_eq!(
            fs::read_to_string(root.path().join("a.txt")).unwrap(),
            "current"
        );
        assert!(!root.path().join(JOURNAL).exists());
    }

    #[cfg(unix)]
    #[test]
    fn replacing_a_file_preserves_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("a.txt"), "old").unwrap();
        fs::set_permissions(root.path().join("a.txt"), fs::Permissions::from_mode(0o600)).unwrap();
        commit(root.path(), vec![change("a.txt", Some("old"), Some("new"))]).unwrap();
        assert_eq!(
            fs::metadata(root.path().join("a.txt"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
