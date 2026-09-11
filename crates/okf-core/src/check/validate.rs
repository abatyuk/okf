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
//!    not concepts. They MUST NOT declare a `type`; only the bundle-root `index.md` may have
//!    frontmatter, and it may contain only `okf_version`.
//!
//! It MUST exit 0 (be conformant) on a spec-conformant bundle even if that bundle has broken
//! links, unknown types, or missing optional fields. The result is typed and printing-free;
//! the CLI maps a non-conformant report to exit 1.
use std::path::Path;

use pulldown_cmark::{Event, Parser, Tag};
use serde_yaml::Value;

use crate::bundle::walk::walk_markdown;
use crate::error::{OkfError, Result};
use crate::model::standard::parse_timestamp;
use crate::parse::markdown::split_frontmatter;
use crate::parse::yaml::parse_frontmatter;

/// Reserved, structural filenames that are never concepts.
const RESERVED: [&str; 2] = ["index.md", "log.md"];

/// Which of the three conformance rules a [`Violation`] breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidateRule {
    /// A concept does not contain a leading frontmatter block.
    MissingFrontmatter,
    /// A `---` frontmatter block is present but its YAML does not parse.
    UnparseableFrontmatter,
    /// A Markdown file is not valid UTF-8.
    InvalidUtf8,
    /// A concept has no non-empty `type`.
    MissingType,
    /// A reserved file (`index.md` / `log.md`) declares a `type`, masquerading as a concept.
    ReservedIsConcept,
    /// A reserved file carries frontmatter outside the root-index `okf_version` exception.
    ReservedFrontmatter,
    /// An index body does not follow §8's headed-section structure.
    InvalidIndex,
    /// A log body does not follow §9's date-grouped newest-first list structure.
    InvalidLog,
}

impl ValidateRule {
    /// Stable machine identifier for the rule.
    pub fn as_str(self) -> &'static str {
        match self {
            ValidateRule::MissingFrontmatter => "missing-frontmatter",
            ValidateRule::UnparseableFrontmatter => "unparseable-frontmatter",
            ValidateRule::InvalidUtf8 => "invalid-utf8",
            ValidateRule::MissingType => "missing-type",
            ValidateRule::ReservedIsConcept => "reserved-is-concept",
            ValidateRule::ReservedFrontmatter => "reserved-frontmatter",
            ValidateRule::InvalidIndex => "invalid-index",
            ValidateRule::InvalidLog => "invalid-log",
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
        // Rule 3: reserved files are structural. Only the bundle-root index may carry
        // frontmatter, and that exception is limited to an `okf_version` string.
        if fm_text.is_none() && content.lines().next() == Some("---") {
            out.push(Violation {
                file: file.clone(),
                rule: ValidateRule::ReservedFrontmatter,
                message: "reserved-file frontmatter is unclosed".to_string(),
            });
            return out;
        }
        match parsed.as_ref().map(|r| r.as_ref()) {
            Some(Ok(map)) if has_nonempty_type(map) => out.push(Violation {
                file: file.clone(),
                rule: ValidateRule::ReservedIsConcept,
                message: format!("reserved file {base:?} must not declare a `type`"),
            }),
            Some(Ok(map)) if !valid_root_index_frontmatter(&file, base.as_str(), map) => {
                out.push(Violation {
                    file: file.clone(),
                    rule: ValidateRule::ReservedFrontmatter,
                    message: "only the bundle-root index.md may carry frontmatter, limited to a string okf_version".to_string(),
                })
            }
            Some(Err(_)) => out.push(Violation {
                file: file.clone(),
                rule: ValidateRule::ReservedFrontmatter,
                message: "reserved-file frontmatter is malformed or not permitted".to_string(),
            }),
            _ => {}
        }
        let body = split_frontmatter(content).1;
        if base == "index.md" && !valid_index_body(&body) {
            out.push(Violation {
                file: file.clone(),
                rule: ValidateRule::InvalidIndex,
                message: "index.md must contain at least one Markdown heading section".to_string(),
            });
        }
        if base == "log.md" {
            if let Err(message) = valid_log_body(&body) {
                out.push(Violation {
                    file: file.clone(),
                    rule: ValidateRule::InvalidLog,
                    message,
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
                file: file.clone(),
                rule: ValidateRule::MissingFrontmatter,
                message: "concept has no leading YAML frontmatter block".to_string(),
            });
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

fn valid_root_index_frontmatter(
    file: &str,
    base: &str,
    map: &indexmap::IndexMap<String, Value>,
) -> bool {
    file == "index.md"
        && base == "index.md"
        && map.len() == 1
        && matches!(map.get("okf_version"), Some(Value::String(version)) if valid_version(version))
}

fn valid_version(version: &str) -> bool {
    let Some((major, minor)) = version.trim().split_once('.') else {
        return false;
    };
    !major.is_empty()
        && !minor.is_empty()
        && major.chars().all(|c| c.is_ascii_digit())
        && minor.chars().all(|c| c.is_ascii_digit())
}

fn valid_index_body(body: &str) -> bool {
    Parser::new(body).any(|event| matches!(event, Event::Start(Tag::Heading { .. })))
}

fn valid_log_body(body: &str) -> std::result::Result<(), String> {
    let mut dates = Vec::new();
    let mut entries_per_date = Vec::new();
    let mut in_date = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            if heading.len() != 10 || parse_timestamp(&format!("{heading}T00:00:00Z")).is_none() {
                return Err(format!(
                    "log date heading must be ISO YYYY-MM-DD: {heading:?}"
                ));
            }
            dates.push(heading.to_string());
            entries_per_date.push(0usize);
            in_date = true;
        } else if trimmed.starts_with('#') && !trimmed.starts_with("# ") {
            return Err(
                "log.md may contain one title and level-two date headings only".to_string(),
            );
        } else if in_date && (trimmed.starts_with("* ") || trimmed.starts_with("- ")) {
            *entries_per_date.last_mut().unwrap() += 1;
        }
    }
    if dates.is_empty() {
        return Err("log.md must contain at least one ## YYYY-MM-DD date group".to_string());
    }
    if entries_per_date.contains(&0) {
        return Err("each log date group must contain at least one flat list entry".to_string());
    }
    if dates.windows(2).any(|pair| pair[0] < pair[1]) {
        return Err("log date groups must be newest first".to_string());
    }
    Ok(())
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
    for rel_path in walk_markdown(root)? {
        let path = root.join(&rel_path);
        let rel = rel_path.to_string_lossy().to_string();
        let bytes =
            std::fs::read(&path).map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))?;
        match String::from_utf8(bytes) {
            Ok(content) => violations.extend(validate_document(&rel, &content)),
            Err(_) => violations.push(Violation {
                file: rel.replace('\\', "/"),
                rule: ValidateRule::InvalidUtf8,
                message: "Markdown document is not valid UTF-8".to_string(),
            }),
        }
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
        assert!(
            v.is_empty(),
            "validate must not flag unknown types or broken links"
        );
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
        assert_eq!(v.len(), 2);
        assert!(v.iter().any(|v| v.rule == ValidateRule::MissingFrontmatter));
        assert!(v.iter().any(|v| v.rule == ValidateRule::MissingType));
    }

    #[test]
    fn unparseable_frontmatter_flagged() {
        // A tab-indented mapping is invalid YAML.
        let v = validate_document("notes/n.md", "---\ntype: Table\n\tbad: : :\n---\n");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, ValidateRule::UnparseableFrontmatter);
    }

    #[test]
    fn reserved_structure_and_root_version_exception() {
        let v = validate_document("index.md", "---\nokf_version: \"0.2\"\n---\n# Welcome\n");
        assert!(v.is_empty());
        let v = validate_document("sub/log.md", "# changelog\n\n## 2026-01-01\n* Update\n");
        assert!(v.is_empty());

        let v = validate_document("sub/index.md", "---\nokf_version: \"0.2\"\n---\n");
        assert_eq!(v[0].rule, ValidateRule::ReservedFrontmatter);

        let v = validate_document("index.md", "---\ntitle: Home\n---\nwelcome\n");
        assert_eq!(v[0].rule, ValidateRule::ReservedFrontmatter);

        let v = validate_document("sub/index.md", "---\nunclosed\n");
        assert_eq!(v[0].rule, ValidateRule::ReservedFrontmatter);
    }

    #[test]
    fn reserved_with_type_flagged() {
        let v = validate_document("index.md", "---\ntype: Table\n---\n# Index\n");
        assert!(v.iter().any(|v| v.rule == ValidateRule::ReservedIsConcept));
    }

    #[test]
    fn validates_index_and_log_bodies() {
        assert_eq!(
            validate_document("index.md", "plain prose\n")[0].rule,
            ValidateRule::InvalidIndex
        );
        assert!(validate_document("index.md", "# Concepts\n").is_empty());
        let bad = validate_document(
            "log.md",
            "# Log\n## 2026-01-01\n* old\n## 2026-02-01\n* new\n",
        );
        assert!(bad.iter().any(|v| v.rule == ValidateRule::InvalidLog));
    }
}
