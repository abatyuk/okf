//! line-range and markdown-heading fingerprints.
//!
//! Both extract a text region, apply the shared canonicalization, then SHA-256 it, so the
//! same logical content hashes identically across platforms and EOL settings.
use super::canonicalize::canonical_sha256;
use crate::error::{OkfError, Result};
use crate::model::source::Fingerprint;
use crate::parse::markdown::{parse_atx, slugify};

/// `line-range`: hash of the 1-based inclusive `[start, end]` line region, canonicalized.
/// Recorded under `content_sha256`.
pub fn line_range_fp(bytes: &[u8], start: usize, end: usize) -> Result<Fingerprint> {
    let text = String::from_utf8_lossy(bytes);
    let region = extract_line_range(&text, start, end);
    Ok(Fingerprint::from_pairs(vec![(
        "content_sha256".to_string(),
        canonical_sha256(&region),
    )]))
}

/// `markdown-heading`: hash of the section body for a heading (identified by its slug),
/// canonicalized. Recorded under `section_sha256`. The heading line itself is excluded, so
/// re-titling the heading alone does not churn the fingerprint.
pub fn markdown_heading_fp(bytes: &[u8], slug: &str) -> Result<Fingerprint> {
    let text = String::from_utf8_lossy(bytes);
    let region = extract_heading_section(&text, slug)
        .ok_or_else(|| OkfError::Usage(format!("markdown heading not found: #{slug}")))?;
    Ok(Fingerprint::from_pairs(vec![(
        "section_sha256".to_string(),
        canonical_sha256(&region),
    )]))
}

/// Extract the 1-based inclusive `[start, end]` line region (before canonicalization).
fn extract_line_range(text: &str, start: usize, end: usize) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = normalized.split('\n').collect();
    if start == 0 || start > lines.len() {
        return String::new();
    }
    let s = start - 1;
    let e = end.min(lines.len());
    if s >= e {
        return String::new();
    }
    lines[s..e].join("\n")
}

/// Region = lines after the matching heading up to (excluding) the next heading of
/// equal-or-shallower level; nested subsections are included; the heading line is excluded.
fn extract_heading_section(text: &str, slug: &str) -> Option<String> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut in_fence = false;
    let mut target_level: Option<usize> = None;
    let mut collected: Vec<&str> = Vec::new();

    for line in normalized.split('\n') {
        let trimmed = line.trim_start();
        let is_fence = trimmed.starts_with("```") || trimmed.starts_with("~~~");
        if is_fence {
            in_fence = !in_fence;
            if target_level.is_some() {
                collected.push(line);
            }
            continue;
        }

        if !in_fence {
            if let Some((level, htext)) = parse_atx(line) {
                match target_level {
                    None => {
                        if slugify(htext) == slug {
                            target_level = Some(level);
                        }
                        // Either way, the heading line itself is not collected.
                        continue;
                    }
                    Some(tl) => {
                        if level <= tl {
                            break; // next equal-or-shallower heading ends the section.
                        }
                        // Deeper heading: part of a nested subsection — include it.
                        collected.push(line);
                        continue;
                    }
                }
            }
        }

        if target_level.is_some() {
            collected.push(line);
        }
    }

    target_level.map(|_| collected.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_range_extracts_inclusive_region() {
        let src = "a\nb\nc\nd\ne\n";
        let fp = line_range_fp(src.as_bytes(), 2, 4).unwrap();
        // region == "b\nc\nd"
        let expect = crate::fingerprint::canonicalize::canonical_sha256("b\nc\nd");
        assert_eq!(fp.get("content_sha256"), Some(expect.as_str()));
    }

    #[test]
    fn line_range_is_eol_stable() {
        let lf = line_range_fp(b"x\ny\nz\n", 1, 3).unwrap();
        let crlf = line_range_fp(b"x\r\ny\r\nz\r\n", 1, 3).unwrap();
        assert_eq!(lf, crlf);
    }

    const DOC: &str = "\
# Title

intro line

## Reimbursement rates

body one
body two

### Nested detail

deeper text

## Next section

other
";

    #[test]
    fn heading_section_includes_nested_excludes_next() {
        let fp = markdown_heading_fp(DOC.as_bytes(), "reimbursement-rates").unwrap();
        let region = "body one\nbody two\n\n### Nested detail\n\ndeeper text";
        let expect = crate::fingerprint::canonicalize::canonical_sha256(region);
        assert_eq!(fp.get("section_sha256"), Some(expect.as_str()));
    }

    #[test]
    fn retitling_heading_does_not_churn() {
        // Same body, heading text changed — the heading line is excluded, so hash is stable.
        let a = "## Old Title\n\nbody\n";
        let b = "## Brand New Title\n\nbody\n";
        let fa = markdown_heading_fp(a.as_bytes(), "old-title").unwrap();
        let fb = markdown_heading_fp(b.as_bytes(), "brand-new-title").unwrap();
        assert_eq!(fa, fb);
    }

    #[test]
    fn missing_heading_errors() {
        assert!(markdown_heading_fp(DOC.as_bytes(), "does-not-exist").is_err());
    }

    #[test]
    fn atx_inside_fence_is_ignored() {
        let doc = "## Sec\n\n```\n## not a heading\n```\n\n## Next\n";
        let fp = markdown_heading_fp(doc.as_bytes(), "sec").unwrap();
        let region = "```\n## not a heading\n```";
        let expect = crate::fingerprint::canonicalize::canonical_sha256(region);
        assert_eq!(fp.get("section_sha256"), Some(expect.as_str()));
    }

    #[test]
    fn slugify_matches_github_style() {
        assert_eq!(slugify("Reimbursement Rates"), "reimbursement-rates");
        assert_eq!(slugify("Foo: Bar (baz)!"), "foo-bar-baz");
    }
}
