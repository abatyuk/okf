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
use std::path::Path;

use crate::bundle::loader::Bundle;
use crate::fingerprint::{Engine, Fingerprinter};
use crate::model::concept::Concept;
use crate::model::source::{parse_sources, Fingerprint, Source};
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
        let concepts = bundle
            .concepts
            .iter()
            .map(|c| self.check_concept(c))
            .filter(ConceptDrift::is_stale)
            .collect();
        StaleReport { concepts }
    }

    /// Check one concept: recompute each source fingerprint and evaluate `stale_after`.
    pub fn check_concept(&self, concept: &Concept) -> ConceptDrift {
        let engine = Engine::new(self.root, self.fs, self.git, self.net);
        let mut sources = Vec::new();

        if let Some(value) = concept.frontmatter.get("sources") {
            for source in parse_sources(value) {
                if let Some(drift) = self.check_source(&engine, &source) {
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
    fn check_source(&self, engine: &Engine, source: &Source) -> Option<SourceDrift> {
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

        match engine.fingerprint(source) {
            Ok(current) => {
                if fingerprints_match(&source.fingerprint, &current) {
                    None
                } else {
                    Some(SourceDrift {
                        drift: DriftKind::Drifted,
                        message: format!(
                            "recorded {} but recomputed {}",
                            render_fp(&source.fingerprint),
                            render_fp(&current)
                        ),
                        current: Some(current),
                        ..base
                    })
                }
            }
            Err(e) => Some(SourceDrift {
                drift: DriftKind::Missing,
                message: format!("cannot fingerprint artifact: {e}"),
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
        let now = self.clock.now_rfc3339();
        // ISO-8601 timestamps sort lexicographically, so a plain string comparison over the
        // shared prefix decides expiry without a date dependency.
        if now.as_str() > stale_after {
            Some(stale_after.to_string())
        } else {
            None
        }
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
    use crate::model::concept::ConceptId;
    use crate::model::frontmatter::Frontmatter;
    use crate::ports::clock::FixedClock;
    use crate::ports::fs::FakeFs;
    use crate::ports::git::FakeGit;
    use indexmap::IndexMap;
    use serde_yaml::Value;
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
        assert_eq!(cd.sources[0].current.as_ref().unwrap().get("blob_sha"), Some("bbb"));
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
        assert!(report.is_empty(), "sha256 of 'abc' matches recorded → in sync");
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
        let c = concept("policies/p", "type: Policy\nstale_after: 2026-08-01");
        let report = check(&bundle(vec![c]), &FakeFs::new(), &FakeGit::new());
        assert_eq!(report.concepts.len(), 1);
        assert_eq!(report.concepts[0].expired.as_deref(), Some("2026-08-01"));
        assert!(report.concepts[0].sources.is_empty());
    }

    #[test]
    fn stale_after_in_future_is_not_flagged() {
        let c = concept("policies/p", "type: Policy\nstale_after: 2027-01-01");
        let report = check(&bundle(vec![c]), &FakeFs::new(), &FakeGit::new());
        assert!(report.is_empty());
    }
}
