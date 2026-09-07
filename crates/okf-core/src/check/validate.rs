//! Conformance validation — the three OKF **hard** rules, and nothing else.
//!
//! OKF is deliberately permissive: a consumer MUST NOT reject a bundle for missing optional
//! fields, unknown `type` values, unknown keys, broken links, or a missing `index.md`. Those
//! are *opinions* and live in [`crate::check::lint`]. `validate` enforces only what the spec
//! makes mandatory:
//!
//! 1. **Parseable frontmatter** — every non-reserved `.md` whose content opens a `---` block
//!    must contain parseable YAML.
//! 2. **Non-empty `type`** — every non-reserved `.md` (a concept) must carry a non-empty
//!    `type` string.
//! 3. **Reserved structure** — the reserved filenames `index.md` / `log.md` are structural,
//!    not concepts, so they MUST NOT declare a `type`.
//!
//! It MUST exit 0 (be conformant) on a spec-conformant bundle even if that bundle has broken
//! links, unknown types, or missing optional fields. The result is typed and printing-free;
//! the CLI maps a non-conformant report to exit 1.
use std::path::Path;

use ignore::WalkBuilder;
use serde_yaml::Value;

use crate::error::{OkfError, Result};
use crate::parse::markdown::split_frontmatter;
use crate::parse::yaml::parse_frontmatter;

/// Reserved, structural filenames that are never concepts.
const RESERVED: [&str; 2] = ["index.md", "log.md"];

/// Which of the three conformance rules a [`Violation`] breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidateRule {
    /// A `---` frontmatter block is present but its YAML does not parse.
    UnparseableFrontmatter,
    /// A concept has no non-empty `type`.
    MissingType,
    /// A reserved file (`index.md` / `log.md`) declares a `type`, masquerading as a concept.
    ReservedIsConcept,
}

impl ValidateRule {
    /// Stable machine identifier for the rule.
    pub fn as_str(self) -> &'static str {
        match self {
            ValidateRule::UnparseableFrontmatter => "unparseable-frontmatter",
            ValidateRule::MissingType => "missing-type",
            ValidateRule::ReservedIsConcept => "reserved-is-concept",
        }
    }
}

/// A single conformance failure, tied to a bundle-relative file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Bundle-relative path of the offending file (forward slashes).
    pub file: String,
    /// The rule that was broken.
    pub rule: ValidateRule,
    /// Human-readable detail (never printed by core; surfaced by the CLI).
    pub message: String,
}

/// The typed outcome of validating a bundle: the (possibly empty) set of nonconformant files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidateReport {
    pub violations: Vec<Violation>,
}

impl ValidateReport {
    /// True when the bundle satisfies all three hard rules.
    pub fn is_conformant(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Validate a single document by its bundle-relative path and raw content, returning any
/// conformance violations. Pure: no I/O, so it is trivially testable. Reserved-vs-concept
/// status is decided from the file's base name.
pub fn validate_document(rel_path: &str, content: &str) -> Vec<Violation> {
    let file = rel_path.replace('\\', "/");
    let base = file.rsplit('/').next().unwrap_or(&file).to_string();
    let mut out = Vec::new();

    let is_reserved = RESERVED.contains(&base.as_str());

    // Parse the frontmatter (if any) once.
    let (fm_text, _body) = split_frontmatter(content);
    let parsed = fm_text.as_deref().map(parse_frontmatter);

    if is_reserved {
        // Rule 3: reserved files are structural. They are only nonconformant if they parse
        // *and* declare a concept `type`. Unparseable/empty reserved files are fine (they are
        // simply not concepts).
        if let Some(Ok(map)) = parsed.as_ref().map(|r| r.as_ref()) {
            if has_nonempty_type(map) {
                out.push(Violation {
                    file,
                    rule: ValidateRule::ReservedIsConcept,
                    message: format!("reserved file {base:?} must not declare a `type`"),
                });
            }
        }
        return out;
    }

    // Non-reserved `.md` == a concept.
    match parsed {
        // A `---` block that does not parse breaks rule 1; we cannot judge `type`, so stop.
        Some(Err(e)) => {
            out.push(Violation {
                file,
                rule: ValidateRule::UnparseableFrontmatter,
                message: format!("frontmatter YAML does not parse: {e}"),
            });
        }
        // Parseable frontmatter, or no frontmatter block at all → check rule 2.
        Some(Ok(map)) => {
            if !has_nonempty_type(&map) {
                out.push(Violation {
                    file,
                    rule: ValidateRule::MissingType,
                    message: "missing or empty `type`".to_string(),
                });
            }
        }
        None => {
            out.push(Violation {
                file,
                rule: ValidateRule::MissingType,
                message: "no frontmatter; missing `type`".to_string(),
            });
        }
    }

    out
}

/// True when the frontmatter map has a `type` key holding a non-empty string.
fn has_nonempty_type(map: &indexmap::IndexMap<String, Value>) -> bool {
    matches!(map.get("type"), Some(Value::String(s)) if !s.trim().is_empty())
}

/// Validate every `*.md` under a bundle root (reserved files included, per rule 3), collecting
/// all nonconformant files. Unlike the bundle loader, a single unparseable file does not abort
/// the walk — every offender is reported. Errors only on an unusable root or an I/O failure.
pub fn validate_bundle(root: &Path) -> Result<ValidateReport> {
    if !root.is_dir() {
        return Err(OkfError::Environment(format!(
            "bundle path is not a directory: {}",
            root.display()
        )));
    }

    let mut violations = Vec::new();
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .parents(true)
        .build();

    for entry in walker {
        let entry = entry.map_err(|e| OkfError::Io(e.to_string()))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map_err(|e| OkfError::Internal(e.to_string()))?
            .to_string_lossy()
            .to_string();
        let content = std::fs::read_to_string(path)
            .map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))?;
        violations.extend(validate_document(&rel, &content));
    }

    // Deterministic ordering by file, then rule.
    violations.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then_with(|| a.rule.as_str().cmp(b.rule.as_str()))
    });

    Ok(ValidateReport { violations })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conformant_concept_has_no_violations() {
        let v = validate_document("tables/customers.md", "---\ntype: Table\n---\nbody\n");
        assert!(v.is_empty());
    }

    #[test]
    fn permissive_ignores_broken_links_and_unknown_types() {
        // Unknown type + a broken link in the body is still spec-conformant.
        let v = validate_document(
            "notes/n.md",
            "---\ntype: Wibble\n---\nsee [x](/missing.md)\n",
        );
        assert!(v.is_empty(), "validate must not flag unknown types or broken links");
    }

    #[test]
    fn missing_type_flagged() {
        let v = validate_document("notes/n.md", "---\ntitle: hi\n---\nbody\n");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, ValidateRule::MissingType);
    }

    #[test]
    fn empty_type_flagged() {
        let v = validate_document("notes/n.md", "---\ntype: \"  \"\n---\n");
        assert_eq!(v[0].rule, ValidateRule::MissingType);
    }

    #[test]
    fn no_frontmatter_is_missing_type() {
        let v = validate_document("notes/n.md", "just a body, no frontmatter\n");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, ValidateRule::MissingType);
    }

    #[test]
    fn unparseable_frontmatter_flagged() {
        // A tab-indented mapping is invalid YAML.
        let v = validate_document("notes/n.md", "---\ntype: Table\n\tbad: : :\n---\n");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, ValidateRule::UnparseableFrontmatter);
    }

    #[test]
    fn reserved_without_type_is_fine() {
        let v = validate_document("index.md", "---\ntitle: Home\n---\nwelcome\n");
        assert!(v.is_empty());
        let v = validate_document("sub/log.md", "# changelog\n");
        assert!(v.is_empty());
    }

    #[test]
    fn reserved_with_type_flagged() {
        let v = validate_document("index.md", "---\ntype: Table\n---\n");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, ValidateRule::ReservedIsConcept);
    }
}
