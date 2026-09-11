//! Integration tests for the check layer: conformance `validate` over fixture bundles, the
//! lint rule engine (with and without an ontology), drift detection via the public
//! `StaleChecker` over fake ports, and the `--fail-on` threshold logic.
use std::path::{Path, PathBuf};

use okf_core::bundle::loader::{load_bundle, Bundle};
use okf_core::check::lint::{lint_bundle, meets_threshold, FailOn, Finding, LintConfig, Severity};
use okf_core::check::stale::{DriftKind, StaleChecker};
use okf_core::check::validate::{validate_bundle, ValidateRule};
use okf_core::model::concept::{Concept, ConceptId};
use okf_core::model::frontmatter::Frontmatter;
use okf_core::ontology::load::load_ontology;
use okf_core::ports::clock::FixedClock;
use okf_core::ports::fs::FakeFs;
use okf_core::ports::git::FakeGit;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn count(findings: &[Finding], rule: &str) -> usize {
    findings.iter().filter(|f| f.rule == rule).count()
}

// --- validate (conformance only) -------------------------------------------

#[test]
fn conformant_bundle_validates_clean() {
    let report = validate_bundle(&fixtures().join("sample-bundle")).unwrap();
    assert!(
        report.is_conformant(),
        "sample-bundle must be spec-conformant, got {:?}",
        report.violations
    );
}

#[test]
fn linked_bundle_with_broken_links_still_conformant() {
    // linked-bundle has a broken link and an unknown `Note` type — both spec-legal.
    let report = validate_bundle(&fixtures().join("linked-bundle")).unwrap();
    assert!(
        report.is_conformant(),
        "validate must not flag opinions: {:?}",
        report.violations
    );
}

#[test]
fn nonconformant_bundle_reports_each_offender() {
    let report = validate_bundle(&fixtures().join("nonconformant-bundle")).unwrap();
    assert!(!report.is_conformant());

    let rule_for = |file: &str| {
        report
            .violations
            .iter()
            .find(|v| v.file == file)
            .map(|v| v.rule)
    };
    assert_eq!(
        rule_for("bad/missing_type.md"),
        Some(ValidateRule::MissingType)
    );
    assert_eq!(
        rule_for("bad/unparseable.md"),
        Some(ValidateRule::UnparseableFrontmatter)
    );
    assert_eq!(rule_for("index.md"), Some(ValidateRule::ReservedIsConcept));
    // The conformant file is not flagged despite its broken link.
    assert!(rule_for("good/ok.md").is_none());
    assert_eq!(report.violations.len(), 3);
}

#[test]
fn validate_includes_gitignored_markdown_and_reports_invalid_utf8_per_file() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join(".gitignore"), "ignored.md\n").unwrap();
    std::fs::write(root.path().join("ignored.md"), "no frontmatter\n").unwrap();
    std::fs::write(root.path().join("binary.md"), [0xff, 0xfe]).unwrap();
    let report = validate_bundle(root.path()).unwrap();
    assert!(report.violations.iter().any(|v| v.file == "ignored.md"));
    assert!(report
        .violations
        .iter()
        .any(|v| v.file == "binary.md" && v.rule == ValidateRule::InvalidUtf8));
}

// --- lint engine + rules ----------------------------------------------------

#[test]
fn lint_without_ontology_fires_expected_rules() {
    let bundle = load_bundle(&fixtures().join("linked-bundle")).unwrap();
    let findings = lint_bundle(&bundle, None, &LintConfig::default());

    // Exactly the travel → /tables/ghost broken link.
    assert_eq!(count(&findings, "broken-link"), 1);
    let bl = findings.iter().find(|f| f.rule == "broken-link").unwrap();
    assert_eq!(bl.concept.as_deref(), Some("/policies/travel"));
    assert!(bl.message.contains("/tables/ghost"));

    // Every concept lacks `description`; all carry a `title`.
    assert_eq!(
        count(&findings, "missing-description"),
        bundle.concepts.len()
    );
    assert_eq!(count(&findings, "missing-title"), 0);

    // Only the orphan note is disconnected.
    assert_eq!(count(&findings, "orphan"), 1);
    let orphan = findings.iter().find(|f| f.rule == "orphan").unwrap();
    assert_eq!(orphan.concept.as_deref(), Some("/notes/orphan"));

    // No ontology supplied → the ontology rule is skipped.
    assert_eq!(count(&findings, "ontology-violation"), 0);
}

#[test]
fn lint_with_ontology_flags_violations() {
    let bundle = load_bundle(&fixtures().join("linked-bundle")).unwrap();
    let ontology = load_ontology(&fixtures().join("ontology/ontology.yaml")).unwrap();
    let findings = lint_bundle(&bundle, Some(&ontology), &LintConfig::default());

    let onto: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.rule == "ontology-violation")
        .collect();
    assert!(!onto.is_empty(), "ontology present → violations expected");

    let msgs: Vec<&str> = onto.iter().map(|f| f.message.as_str()).collect();
    // `Note` is not defined in the ontology.
    assert!(msgs
        .iter()
        .any(|m| m.contains("Note") && m.contains("not defined")));
    // Policy requires the `description` built-in, which travel lacks.
    assert!(msgs.iter().any(|m| m.contains("description")));
    // Computation requires `runtime`, which mileage lacks.
    assert!(msgs.iter().any(|m| m.contains("runtime")));

    // The non-ontology rules still run alongside.
    assert_eq!(count(&findings, "broken-link"), 1);
}

#[test]
fn lint_severity_overrides_apply() {
    let bundle = load_bundle(&fixtures().join("linked-bundle")).unwrap();
    let config = LintConfig {
        orphan: Severity::Error,
        ..LintConfig::default()
    }; // bump orphan from Info to Error
    let findings = lint_bundle(&bundle, None, &config);
    let orphan = findings.iter().find(|f| f.rule == "orphan").unwrap();
    assert_eq!(orphan.severity, Severity::Error);
}

// --- fail-on threshold ------------------------------------------------------

#[test]
fn fail_on_threshold_drives_exit_decision() {
    let bundle = load_bundle(&fixtures().join("linked-bundle")).unwrap();
    let findings = lint_bundle(&bundle, None, &LintConfig::default());

    // Default config: a broken-link is Error, so the default (Error) threshold trips.
    assert!(meets_threshold(&findings, FailOn::Error));
    assert!(meets_threshold(&findings, FailOn::Warn));
    assert!(meets_threshold(&findings, FailOn::Any));
    assert!(!meets_threshold(&findings, FailOn::Never));
}

// --- stale (drift) via the public StaleChecker over fakes -------------------

fn concept(id: &str, yaml: &str) -> Concept {
    let map: indexmap::IndexMap<String, serde_yaml::Value> = serde_yaml::from_str(yaml).unwrap();
    Concept {
        id: ConceptId::from_relative(id),
        frontmatter: Frontmatter::from_map(map),
        body: String::new(),
    }
}

#[test]
fn stale_checker_detects_drift_over_fakes() {
    let bundle = Bundle {
        root: PathBuf::from(""),
        concepts: vec![
            concept(
                "computations/synced",
                "type: Computation\nsources:\n- resource: src/a.py\n  kind: git-path\n  fingerprint:\n    blob_sha: keep\n",
            ),
            concept(
                "computations/drifted",
                "type: Computation\nsources:\n- resource: src/b.py\n  kind: git-path\n  fingerprint:\n    blob_sha: old\n",
            ),
        ],
    };
    let fs = FakeFs::new();
    let git = FakeGit::new()
        .with_hash_object("src/a.py", "keep") // matches recorded → in sync
        .with_hash_object("src/b.py", "new"); // differs from recorded → drift
    let clock = FixedClock("2026-09-07T00:00:00Z".to_string());

    let checker = StaleChecker::new(Path::new(""), &fs, &git, None, &clock);
    let report = checker.check_bundle(&bundle);

    assert_eq!(
        report.concepts.len(),
        1,
        "only the drifted concept is reported"
    );
    let drift = &report.concepts[0];
    assert_eq!(drift.concept, "/computations/drifted");
    assert_eq!(drift.sources[0].drift, DriftKind::Drifted);
}
