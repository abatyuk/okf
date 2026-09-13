//! Drift detection — recorded vs. recomputed fingerprint, plus `stale_after` expiry and
//! missing artifacts.
//!
//! For each concept, its typed `sources[]` are parsed, each source's *current* fingerprint is
//! recomputed via [`fingerprint::Engine`], and the result is compared against the fingerprint
//! recorded at last sync. A source whose recompute fails (missing file, unknown kind, git
//! error) is reported as a missing/uncomputable artifact. A concept whose `stale_after`
//! timestamp has passed is flagged as expired.
//!
//! All effects flow through injected ports, so the whole check is hermetically testable with
//! the fakes ([`FakeFs`]/[`FakeGit`]/[`FixedClock`]). [`check_stale`] is a convenience
//! constructor that wires the real ports.
//!
//! [`FakeFs`]: crate::ports::fs::FakeFs
//! [`FakeGit`]: crate::ports::git::FakeGit
//! [`FixedClock`]: crate::ports::clock::FixedClock
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use crate::bundle::loader::Bundle;
use crate::fingerprint::{Engine, Fingerprinter};
use crate::model::concept::Concept;
use crate::model::source::{parse_sources, Fingerprint, Source, SourceKind};
use crate::model::standard::parse_timestamp;
use crate::ports::clock::{Clock, SystemClock};
use crate::ports::fs::{FileSystem, RealFs};
use crate::ports::git::{Git, RealGit};
use crate::ports::net::Net;

/// Why a source (or concept) is considered out of sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftKind {
    /// The recomputed fingerprint differs from the recorded one.
    Drifted,
    /// The artifact could not be read / fingerprinted (missing file, git error, unknown kind).
    Missing,
    /// The concept's `stale_after` timestamp has passed.
    Expired,
}

impl DriftKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DriftKind::Drifted => "drifted",
            DriftKind::Missing => "missing",
            DriftKind::Expired => "expired",
        }
    }
}

/// Drift detected on one `sources[]` entry of a concept.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceDrift {
    /// The source's `resource` string.
    pub resource: String,
    /// The source `kind` as written on disk.
    pub kind: String,
    /// [`DriftKind::Drifted`] or [`DriftKind::Missing`].
    pub drift: DriftKind,
    /// The fingerprint recorded at last sync.
    pub recorded: Fingerprint,
    /// The freshly recomputed fingerprint (absent when recompute failed).
    pub current: Option<Fingerprint>,
    /// Human-readable detail (never printed by core).
    pub message: String,
}

/// The drift status of a single concept: its expired flag plus per-source drift. A concept
/// with `is_stale() == false` is fully in sync and is omitted from [`StaleReport`].
#[derive(Debug, Clone, PartialEq)]
pub struct ConceptDrift {
    /// The concept id (e.g. `/computations/mileage`).
    pub concept: String,
    /// The `stale_after` value that has passed, if the concept expired.
    pub expired: Option<String>,
    /// Per-source drift (only sources that drifted or went missing).
    pub sources: Vec<SourceDrift>,
}

impl ConceptDrift {
    /// True if anything about this concept is out of sync.
    pub fn is_stale(&self) -> bool {
        self.expired.is_some() || !self.sources.is_empty()
    }
}

/// The typed outcome of a staleness check: one entry per drifted concept (in-sync concepts
/// are omitted), in bundle order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StaleReport {
    pub concepts: Vec<ConceptDrift>,
}

impl StaleReport {
    /// True when nothing drifted.
    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }
}

/// A drift checker over injected ports. `root` is the base directory that source resources
/// resolve against (normally the bundle root); `net` is optional (only `url` sources need it).
pub struct StaleChecker<'a> {
    root: &'a Path,
    fs: &'a dyn FileSystem,
    git: &'a dyn Git,
    net: Option<&'a dyn Net>,
    clock: &'a dyn Clock,
}

type FingerprintCache = HashMap<(String, String), std::result::Result<Fingerprint, String>>;

impl<'a> StaleChecker<'a> {
    /// Build a checker from explicit ports (hermetic tests pass the fakes).
    pub fn new(
        root: &'a Path,
        fs: &'a dyn FileSystem,
        git: &'a dyn Git,
        net: Option<&'a dyn Net>,
        clock: &'a dyn Clock,
    ) -> Self {
        Self {
            root,
            fs,
            git,
            net,
            clock,
        }
    }

    /// Check a whole bundle, returning drift for every out-of-sync concept (bundle order).
    pub fn check_bundle(&self, bundle: &Bundle) -> StaleReport {
        let engine = Engine::new(self.root, self.fs, self.git, self.net);
        let mut cache = FingerprintCache::new();
        prime_git_cache(bundle, &engine, &mut cache);
        let concepts = bundle
            .concepts
            .iter()
            .map(|c| self.check_concept_with(c, &engine, &mut cache))
            .filter(ConceptDrift::is_stale)
            .collect();
        StaleReport { concepts }
    }

    /// Check one concept: recompute each source fingerprint and evaluate `stale_after`.
    pub fn check_concept(&self, concept: &Concept) -> ConceptDrift {
        let engine = Engine::new(self.root, self.fs, self.git, self.net);
        let mut cache = FingerprintCache::new();
        self.check_concept_with(concept, &engine, &mut cache)
    }

    fn check_concept_with(
        &self,
        concept: &Concept,
        engine: &Engine,
        cache: &mut FingerprintCache,
    ) -> ConceptDrift {
        let mut sources = Vec::new();

        if let Some(value) = concept.frontmatter.get("sources") {
            for source in parse_sources(value) {
                if let Some(drift) = self.check_source(engine, &source, cache) {
                    sources.push(drift);
                }
            }
        }

        ConceptDrift {
            concept: concept.id.0.clone(),
            expired: self.expiry(concept),
            sources,
        }
    }

    /// Recompute and compare one source. Returns `None` when the source is in sync or has no
    /// recorded fingerprint to compare against.
    fn check_source(
        &self,
        engine: &Engine,
        source: &Source,
        cache: &mut FingerprintCache,
    ) -> Option<SourceDrift> {
        // A source with nothing recorded has no baseline to drift from — skip it.
        if source.fingerprint.is_empty() {
            return None;
        }

        let base = SourceDrift {
            resource: source.resource.clone(),
            kind: source.kind.as_kind_str().to_string(),
            drift: DriftKind::Drifted,
            recorded: source.fingerprint.clone(),
            current: None,
            message: String::new(),
        };

        let key = cache_key(source);
        let current = cache
            .entry(key)
            .or_insert_with(|| engine.fingerprint(source).map_err(|e| e.to_string()));

        match current {
            Ok(current) => {
                if fingerprints_match(&source.fingerprint, current) {
                    None
                } else {
                    Some(SourceDrift {
                        drift: DriftKind::Drifted,
                        message: format!(
                            "recorded {} but recomputed {}",
                            render_fp(&source.fingerprint),
                            render_fp(current)
                        ),
                        current: Some(current.clone()),
                        ..base
                    })
                }
            }
            Err(message) => Some(SourceDrift {
                drift: DriftKind::Missing,
                message: format!("cannot fingerprint artifact: {message}"),
                ..base
            }),
        }
    }

    /// The concept's `stale_after` value if it lies in the past relative to the clock's "now".
    fn expiry(&self, concept: &Concept) -> Option<String> {
        let stale_after = concept.frontmatter.get_str("stale_after")?.trim();
        if stale_after.is_empty() {
            return None;
        }
        let now = parse_timestamp(&self.clock.now_rfc3339())?;
        let deadline = parse_timestamp(stale_after)?;
        if now >= deadline {
            Some(stale_after.to_string())
        } else {
            None
        }
    }
}

fn cache_key(source: &Source) -> (String, String) {
    (
        source.kind.as_kind_str().to_string(),
        source.resource.clone(),
    )
}

fn prime_git_cache(bundle: &Bundle, engine: &Engine, cache: &mut FingerprintCache) {
    let mut paths = BTreeMap::new();
    let mut commits = BTreeMap::new();
    for concept in &bundle.concepts {
        let Some(value) = concept.frontmatter.get("sources") else {
            continue;
        };
        for source in parse_sources(value) {
            if source.fingerprint.is_empty() {
                continue;
            }
            match source.kind {
                SourceKind::GitPath => {
                    paths.entry(cache_key(&source)).or_insert(source);
                }
                SourceKind::GitCommit => {
                    commits.entry(cache_key(&source)).or_insert(source);
                }
                _ => {}
            }
        }
    }

    let path_sources: Vec<Source> = paths.into_values().collect();
    let path_results = engine.fingerprint_git_paths(&path_sources);
    let commit_sources: Vec<Source> = commits.into_values().collect();
    let commit_results = engine.fingerprint_git_commits(&commit_sources);
    for (source, result) in path_sources
        .into_iter()
        .zip(path_results)
        .chain(commit_sources.into_iter().zip(commit_results))
    {
        cache.insert(
            cache_key(&source),
            result.map_err(|error| error.to_string()),
        );
    }
}

/// Every field recorded on `recorded` must be present with an equal value on `current`.
/// `current` may carry additional fields (e.g. a `url` source records several validators).
fn fingerprints_match(recorded: &Fingerprint, current: &Fingerprint) -> bool {
    recorded
        .fields
        .iter()
        .all(|(k, v)| current.get(k) == Some(v.as_str()))
}

/// Compact `k=v,k=v` rendering of a fingerprint for drift messages.
fn render_fp(fp: &Fingerprint) -> String {
    if fp.is_empty() {
        return "<none>".to_string();
    }
    fp.fields
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// Convenience: run a staleness check with the real ports (`RealFs`/`RealGit`/`SystemClock`,
/// no network) against the bundle's own root.
pub fn check_stale(bundle: &Bundle) -> StaleReport {
    let fs = RealFs;
    let git = RealGit;
    let clock = SystemClock;
    let checker = StaleChecker::new(&bundle.root, &fs, &git, None, &clock);
    checker.check_bundle(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result as OkfResult;
    use crate::model::concept::ConceptId;
    use crate::model::frontmatter::Frontmatter;
    use crate::ports::clock::FixedClock;
    use crate::ports::fs::FakeFs;
    use crate::ports::git::{FakeGit, Git};
    use indexmap::IndexMap;
    use serde_yaml::Value;
    use std::cell::Cell;
    use std::path::PathBuf;

    fn concept(id: &str, yaml: &str) -> Concept {
        let map: IndexMap<String, Value> = serde_yaml::from_str(yaml).unwrap();
        Concept {
            id: ConceptId::from_relative(id),
            frontmatter: Frontmatter::from_map(map),
            body: String::new(),
        }
    }

    fn bundle(concepts: Vec<Concept>) -> Bundle {
        Bundle {
            root: PathBuf::from(""),
            concepts,
        }
    }

    const NOW: &str = "2026-09-07T00:00:00Z";

    fn check(bundle: &Bundle, fs: &FakeFs, git: &FakeGit) -> StaleReport {
        let clock = FixedClock(NOW.to_string());
        StaleChecker::new(Path::new(""), fs, git, None, &clock).check_bundle(bundle)
    }

    #[test]
    fn git_path_in_sync_is_not_reported() {
        let c = concept(
            "computations/mileage",
            "type: Computation\nsources:\n- resource: src/x.py\n  kind: git-path\n  fingerprint:\n    blob_sha: aaa\n",
        );
        let git = FakeGit::new().with_hash_object("src/x.py", "aaa");
        let report = check(&bundle(vec![c]), &FakeFs::new(), &git);
        assert!(report.is_empty(), "matching blob sha must not drift");
    }

    #[test]
    fn git_path_drift_is_detected() {
        let c = concept(
            "computations/mileage",
            "type: Computation\nsources:\n- resource: src/x.py\n  kind: git-path\n  fingerprint:\n    blob_sha: aaa\n",
        );
        // Recorded aaa, current bbb → drift.
        let git = FakeGit::new().with_hash_object("src/x.py", "bbb");
        let report = check(&bundle(vec![c]), &FakeFs::new(), &git);
        assert_eq!(report.concepts.len(), 1);
        let cd = &report.concepts[0];
        assert_eq!(cd.concept, "/computations/mileage");
        assert_eq!(cd.sources.len(), 1);
        assert_eq!(cd.sources[0].drift, DriftKind::Drifted);
        assert_eq!(
            cd.sources[0].current.as_ref().unwrap().get("blob_sha"),
            Some("bbb")
        );
    }

    struct CountingGit {
        worktree_roots: Cell<usize>,
        hashes: Cell<usize>,
        batches: Cell<usize>,
    }

    impl CountingGit {
        fn new() -> Self {
            Self {
                worktree_roots: Cell::new(0),
                hashes: Cell::new(0),
                batches: Cell::new(0),
            }
        }
    }

    impl Git for CountingGit {
        fn worktree_root(&self, anchor: &Path) -> OkfResult<PathBuf> {
            self.worktree_roots.set(self.worktree_roots.get() + 1);
            Ok(anchor.to_path_buf())
        }

        fn hash_object(&self, _path: &Path) -> OkfResult<String> {
            self.hashes.set(self.hashes.get() + 1);
            Ok("aaa".to_string())
        }

        fn hash_objects_in(&self, _root: &Path, paths: &[PathBuf]) -> Vec<OkfResult<String>> {
            self.batches.set(self.batches.get() + 1);
            self.hashes.set(self.hashes.get() + paths.len());
            paths.iter().map(|_| Ok("aaa".to_string())).collect()
        }

        fn last_commit(&self, _path: &Path) -> OkfResult<String> {
            unreachable!("not used by this test")
        }

        fn show(&self, _rev: &str, _path: &Path) -> OkfResult<Vec<u8>> {
            unreachable!("not used by this test")
        }

        fn ls_tree(&self, _rev: &str) -> OkfResult<Vec<String>> {
            unreachable!("not used by this test")
        }
    }

    #[test]
    fn bundle_check_reuses_git_root_and_duplicate_fingerprints() {
        let source = "type: Note\nsources:\n- resource: src/shared.py\n  kind: git-path\n  fingerprint:\n    blob_sha: aaa\n";
        let other = "type: Note\nsources:\n- resource: src/other.py\n  kind: git-path\n  fingerprint:\n    blob_sha: aaa\n";
        let bundle = bundle(vec![
            concept("a", source),
            concept("b", source),
            concept("c", other),
        ]);
        let git = CountingGit::new();
        let clock = FixedClock(NOW.to_string());

        let report = StaleChecker::new(Path::new("repo"), &FakeFs::new(), &git, None, &clock)
            .check_bundle(&bundle);

        assert!(report.is_empty());
        assert_eq!(git.worktree_roots.get(), 1, "resolve the worktree once");
        assert_eq!(git.hashes.get(), 2, "hash each distinct source once");
        assert_eq!(git.batches.get(), 1, "use one batch Git operation");
    }

    #[test]
    fn missing_artifact_is_flagged() {
        let c = concept(
            "docs/d",
            "type: Note\nsources:\n- resource: data/raw.bin\n  kind: file\n  fingerprint:\n    sha256: dead\n",
        );
        // FakeFs has no such file → fingerprint() errors → Missing.
        let report = check(&bundle(vec![c]), &FakeFs::new(), &FakeGit::new());
        assert_eq!(report.concepts.len(), 1);
        assert_eq!(report.concepts[0].sources[0].drift, DriftKind::Missing);
    }

    #[test]
    fn file_sha256_recompute_matches() {
        let c = concept(
            "docs/d",
            "type: Note\nsources:\n- resource: data/raw.bin\n  kind: file\n  fingerprint:\n    sha256: ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\n",
        );
        let fs = FakeFs::new().with_file("data/raw.bin", b"abc".to_vec());
        let report = check(&bundle(vec![c]), &fs, &FakeGit::new());
        assert!(
            report.is_empty(),
            "sha256 of 'abc' matches recorded → in sync"
        );
    }

    #[test]
    fn source_without_recorded_fingerprint_is_skipped() {
        let c = concept(
            "docs/d",
            "type: Note\nsources:\n- resource: src/x.py\n  kind: git-path\n",
        );
        // No recorded fingerprint → nothing to compare, even though git has no answer.
        let report = check(&bundle(vec![c]), &FakeFs::new(), &FakeGit::new());
        assert!(report.is_empty());
    }

    #[test]
    fn stale_after_expiry_is_flagged() {
        let c = concept(
            "policies/p",
            "type: Policy\nstale_after: 2026-08-01T00:00:00Z",
        );
        let report = check(&bundle(vec![c]), &FakeFs::new(), &FakeGit::new());
        assert_eq!(report.concepts.len(), 1);
        assert_eq!(
            report.concepts[0].expired.as_deref(),
            Some("2026-08-01T00:00:00Z")
        );
        assert!(report.concepts[0].sources.is_empty());
    }

    #[test]
    fn stale_after_in_future_is_not_flagged() {
        let c = concept(
            "policies/p",
            "type: Policy\nstale_after: 2027-01-01T00:00:00Z",
        );
        let report = check(&bundle(vec![c]), &FakeFs::new(), &FakeGit::new());
        assert!(report.is_empty());
    }

    #[test]
    fn stale_after_is_inclusive_and_offset_aware() {
        let c = concept("p", "type: Policy\nstale_after: 2026-09-06T17:00:00-07:00");
        let report = check(&bundle(vec![c]), &FakeFs::new(), &FakeGit::new());
        assert_eq!(report.concepts.len(), 1);
    }
}
