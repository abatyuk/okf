//! Specification-derived OKF v0.2 contract cases.
//!
//! Each case cites the section whose normative outcome it protects. Optional-family mistakes
//! remain consumable and lint-only; §11 hard failures are validation failures.

use okf_core::bundle::loader::load_bundle;
use okf_core::check::lint::{lint_bundle, LintConfig};
use okf_core::check::validate::validate_bundle;
use okf_core::model::trust::TrustTier;

fn bundle(files: &[(&str, &[u8])]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for (path, content) in files {
        let target = root.path().join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, content).unwrap();
    }
    root
}

#[test]
fn section_11_minimal_unknown_and_broken_are_conformant() {
    let root = bundle(&[(
        "custom.md",
        b"---\ntype: Completely Unknown\nextension: {anything: true}\n---\n[future](/missing.md)\n",
    )]);
    assert!(validate_bundle(root.path()).unwrap().is_conformant());
}

#[test]
fn sections_8_9_reserved_structures_are_enforced() {
    let valid = bundle(&[
        ("index.md", b"---\nokf_version: '0.2'\n---\n# Concepts\n"),
        (
            "log.md",
            b"# Log\n\n## 2026-09-11\n* New\n\n## 2026-09-10\n* Old\n",
        ),
    ]);
    assert!(validate_bundle(valid.path()).unwrap().is_conformant());

    let invalid = bundle(&[
        ("index.md", b"plain prose\n"),
        ("log.md", b"# Log\n## yesterday\n* item\n"),
    ]);
    assert!(!validate_bundle(invalid.path()).unwrap().is_conformant());
}

#[test]
fn section_12_unknown_well_formed_version_is_best_effort() {
    let root = bundle(&[
        ("index.md", b"---\nokf_version: '9.7'\n---\n# Concepts\n"),
        ("a.md", b"---\ntype: T\n---\n"),
    ]);
    assert!(validate_bundle(root.path()).unwrap().is_conformant());
    assert_eq!(load_bundle(root.path()).unwrap().concepts.len(), 1);
}

#[test]
fn section_5_optional_family_errors_are_lint_only() {
    let root = bundle(&[(
        "a.md",
        b"---\ntype: T\nstatus: invented\nstale_after: 2026-01-01\nverified: {by: human:x}\n---\n",
    )]);
    assert!(validate_bundle(root.path()).unwrap().is_conformant());
    let loaded = load_bundle(root.path()).unwrap();
    assert_eq!(loaded.concepts[0].trust_tier(), TrustTier::Unverified);
    assert!(lint_bundle(&loaded, None, &LintConfig::default())
        .iter()
        .any(|finding| finding.rule == "okf-v02"));
}

#[test]
fn section_5_bare_verified_mapping_is_normalized() {
    let root = bundle(&[(
        "a.md",
        b"---\ntype: T\nverified: {by: human:x, at: 2026-09-11T00:00:00Z}\n---\n",
    )]);
    let loaded = load_bundle(root.path()).unwrap();
    assert_eq!(loaded.concepts[0].trust_tier(), TrustTier::HumanReviewed);
}

#[test]
fn section_10_attested_runtime_is_advisory_to_bundle_conformance() {
    let root = bundle(&[(
        "c.md",
        b"---\ntype: Attested Computation\n---\n# Computation\n\n```sql\nselect 1\n```\n",
    )]);
    assert!(validate_bundle(root.path()).unwrap().is_conformant());
    let loaded = load_bundle(root.path()).unwrap();
    assert!(lint_bundle(&loaded, None, &LintConfig::default())
        .iter()
        .any(|finding| finding.message.contains("runtime")));
}
