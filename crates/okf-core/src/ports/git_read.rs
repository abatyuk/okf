//! Scoped local Git reads. Nested operations share discovery, object reads and failures;
//! dropping the outer session discards everything so later commands observe fresh state.
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};
use std::rc::{Rc, Weak};

type CachedOutput = Result<Output, (io::ErrorKind, String)>;
#[derive(Default)]
struct Cache {
    commands: HashMap<(PathBuf, Vec<String>), CachedOutput>,
    blobs: HashMap<(PathBuf, String), Option<Vec<u8>>>,
    readers: HashMap<PathBuf, Option<Batch>>,
}
thread_local! {
    static ACTIVE: RefCell<Weak<RefCell<Cache>>> = const { RefCell::new(Weak::new()) };
}
/// Own this guard for one read operation; nested guards reuse the same cache.
pub struct ReadSession(Rc<RefCell<Cache>>);
impl Default for ReadSession {
    fn default() -> Self {
        Self::new()
    }
}
impl ReadSession {
    pub fn new() -> Self {
        Self(ACTIVE.with(|active| {
            if let Some(cache) = active.borrow().upgrade() {
                return cache;
            }
            let cache = Rc::new(RefCell::new(Cache::default()));
            *active.borrow_mut() = Rc::downgrade(&cache);
            cache
        }))
    }
}
fn command(root: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(root)
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_TERMINAL_PROMPT", "0");
    cmd
}
/// Read-only commands only; never use this cache for Git mutations.
pub(crate) fn output(root: &Path, args: &[&str]) -> io::Result<Output> {
    let cache = ACTIVE.with(|active| active.borrow().upgrade());
    let Some(cache) = cache else {
        return command(root).args(args).output();
    };
    let key = (
        root.to_path_buf(),
        args.iter().map(|arg| arg.to_string()).collect(),
    );
    let mut cache = cache.borrow_mut();
    cache
        .commands
        .entry(key)
        .or_insert_with(|| {
            command(root)
                .args(args)
                .output()
                .map_err(|error| (error.kind(), error.to_string()))
        })
        .clone()
        .map_err(|(kind, message)| io::Error::new(kind, message))
}
/// Read one immutable object through a persistent cat-file process, caching missing objects too.
pub(crate) fn blob(root: &Path, revision: &str, path: &Path) -> Option<Vec<u8>> {
    let session = ReadSession::new();
    let spec = format!("{revision}:{}", path.to_str()?);
    let key = (root.to_path_buf(), spec.clone());
    let mut cache = session.0.borrow_mut();
    if let Some(bytes) = cache.blobs.get(&key) {
        return bytes.clone();
    }
    // The line-based batch protocol cannot represent these names. Keep Git's argv semantics.
    let bytes = if spec.contains(['\n', '\r', '\0']) {
        command(root)
            .args(["show", &spec])
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| out.stdout)
    } else {
        cache
            .readers
            .entry(root.to_path_buf())
            .or_insert_with(|| Batch::new(root).ok())
            .as_mut()
            .and_then(|reader| reader.read(&spec).ok().flatten())
    };
    cache.blobs.insert(key, bytes.clone());
    bytes
}
struct Batch {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}
impl Batch {
    fn new(root: &Path) -> io::Result<Self> {
        let mut child = command(root)
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Ok(Self {
            child,
            stdin,
            stdout,
        })
    }
    fn read(&mut self, spec: &str) -> io::Result<Option<Vec<u8>>> {
        writeln!(self.stdin, "{spec}")?;
        self.stdin.flush()?;
        let mut header = String::new();
        self.stdout.read_line(&mut header)?;
        if header.ends_with(" missing\n") {
            return Ok(None);
        }
        let fields: Vec<_> = header.split_whitespace().collect();
        let size = fields
            .get(2)
            .and_then(|n| n.parse::<usize>().ok())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid cat-file response")
            })?;
        let mut bytes = vec![0; size];
        self.stdout.read_exact(&mut bytes)?;
        let mut newline = [0];
        self.stdout.read_exact(&mut newline)?;
        if fields.get(1) != Some(&"blob") {
            return Ok(None);
        }
        Ok(Some(bytes))
    }
}
impl Drop for Batch {
    fn drop(&mut self) {
        // Terminate and reap even if the outer operation exits early on a parse error.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git(root: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
    }
    fn commit(root: &Path) {
        git(root, &["add", "."]);
        git(
            root,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.test",
                "commit",
                "-qm",
                "fixture",
            ],
        );
    }
    #[test]
    fn nested_sessions_share_reads_but_later_operations_observe_new_state() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        git(root, &["init", "-q"]);
        std::fs::write(root.join("a"), b"one\0\n").unwrap();
        commit(root);
        let session = ReadSession::new();
        let weak = Rc::downgrade(&session.0);
        {
            let nested = ReadSession::new();
            assert!(Rc::ptr_eq(&session.0, &nested.0));
            assert_eq!(
                blob(root, "HEAD", Path::new("a")),
                Some(b"one\0\n".to_vec())
            );
            assert_eq!(blob(root, "HEAD", Path::new("missing")), None);
        }
        // Failed reads and symbolic refs are stable within this read operation.
        std::fs::write(root.join("a"), b"two").unwrap();
        std::fs::write(root.join("missing"), b"present").unwrap();
        commit(root);
        assert_eq!(
            blob(root, "HEAD", Path::new("a")),
            Some(b"one\0\n".to_vec())
        );
        assert_eq!(blob(root, "HEAD", Path::new("missing")), None);
        drop(session);
        assert!(weak.upgrade().is_none());
        let _next = ReadSession::new();
        assert_eq!(blob(root, "HEAD", Path::new("a")), Some(b"two".to_vec()));
        assert_eq!(
            blob(root, "HEAD", Path::new("missing")),
            Some(b"present".to_vec())
        );
    }
    #[test]
    fn batch_reads_preserve_bytes_and_support_whitespace_names() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        git(root, &["init", "-q"]);
        for name in ["space name", "unicode-λ", "newline\nname"] {
            std::fs::write(root.join(name), [0, 255, b'\n', b'X']).unwrap();
        }
        commit(root);
        let _reads = ReadSession::new();
        for name in ["space name", "unicode-λ", "newline\nname"] {
            assert_eq!(
                blob(root, "HEAD", Path::new(name)),
                Some(vec![0, 255, b'\n', b'X'])
            );
        }
        assert_eq!(blob(root, "HEAD", Path::new("not-present")), None);
        assert_eq!(
            blob(root, "HEAD", Path::new("space name")),
            Some(vec![0, 255, b'\n', b'X'])
        );
    }
}
