//! Document-body editing: whole-body replace/append/clear and section-aware edits.
//!
//! All operations take and return the concept **body** (everything after the frontmatter),
//! so the frontmatter and its key order are untouched. Section edits locate a heading via
//! [`find_section`] (matched by GitHub-style slug, so `"Rates"` and `"rates"` are the same
//! target) and splice by `\n`-delimited line, which round-trips exactly for both LF and CRLF
//! bodies. Content is normalized to end with a single trailing newline.
use crate::error::{OkfError, Result};
use crate::parse::markdown::find_section;

/// Replace the entire body with `text` (normalized to a single trailing newline).
pub fn set_body(text: &str) -> String {
    with_trailing_nl(text)
}

/// Append `text` to the body as a new block, separated by a blank line.
pub fn append_body(old: &str, text: &str) -> String {
    let block = text.trim_matches('\n');
    if old.trim_matches('\n').is_empty() {
        return with_trailing_nl(block);
    }
    let mut out = old.trim_end_matches('\n').to_string();
    out.push_str("\n\n");
    out.push_str(block);
    out.push('\n');
    out
}

/// Replace the content of the section under `heading` (the heading line is kept). Errors if no
/// heading matches.
pub fn set_section(body: &str, heading: &str, text: &str) -> Result<String> {
    let span = find_section(body, heading)
        .ok_or_else(|| OkfError::Usage(format!("edit: heading not found: {heading:?}")))?;
    let lines: Vec<&str> = body.split('\n').collect();
    let mut out: Vec<String> = lines[..span.content_start].iter().map(|s| s.to_string()).collect();
    out.push(String::new());
    out.extend(text.trim_matches('\n').split('\n').map(|s| s.to_string()));
    out.push(String::new());
    out.extend(lines[span.end..].iter().map(|s| s.to_string()));
    Ok(normalize(&out.join("\n")))
}

/// Append `text` to the end of the section under `heading`, separated by a blank line. Errors
/// if no heading matches.
pub fn append_section(body: &str, heading: &str, text: &str) -> Result<String> {
    let span = find_section(body, heading)
        .ok_or_else(|| OkfError::Usage(format!("edit: heading not found: {heading:?}")))?;
    let lines: Vec<&str> = body.split('\n').collect();
    // Trim trailing blank lines inside the section so the appended block sits right after
    // the existing content, then re-pad with one blank separator.
    let mut end = span.end;
    while end > span.content_start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    let mut out: Vec<String> = lines[..end].iter().map(|s| s.to_string()).collect();
    out.push(String::new());
    out.extend(text.trim_matches('\n').split('\n').map(|s| s.to_string()));
    out.push(String::new());
    out.extend(lines[span.end..].iter().map(|s| s.to_string()));
    Ok(normalize(&out.join("\n")))
}

/// Remove the section under `heading` entirely, including its heading line and nested
/// subsections. Errors if no heading matches.
pub fn remove_section(body: &str, heading: &str) -> Result<String> {
    let span = find_section(body, heading)
        .ok_or_else(|| OkfError::Usage(format!("edit: heading not found: {heading:?}")))?;
    let lines: Vec<&str> = body.split('\n').collect();
    let mut out: Vec<String> = lines[..span.heading_line].iter().map(|s| s.to_string()).collect();
    out.extend(lines[span.end..].iter().map(|s| s.to_string()));
    Ok(normalize(&out.join("\n")))
}

/// Ensure a single trailing newline (empty stays empty).
fn with_trailing_nl(s: &str) -> String {
    if s.is_empty() {
        String::new()
    } else {
        format!("{}\n", s.trim_end_matches('\n'))
    }
}

/// Collapse 3-or-more consecutive newlines to a blank-line pair and guarantee one trailing
/// newline, so splicing never leaves ragged runs of blank lines.
fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 1);
    let mut newlines = 0usize;
    for ch in s.chars() {
        if ch == '\n' {
            newlines += 1;
            if newlines <= 2 {
                out.push('\n');
            }
        } else {
            newlines = 0;
            out.push(ch);
        }
    }
    let trimmed = out.trim_end_matches('\n');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{trimmed}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# Title\n\nintro\n\n## Rates\n\nold body\n\n## Next\n\ntail\n";

    #[test]
    fn set_body_replaces_and_normalizes_trailing_newline() {
        assert_eq!(set_body("hello"), "hello\n");
        assert_eq!(set_body("hello\n\n\n"), "hello\n");
        assert_eq!(set_body(""), "");
    }

    #[test]
    fn append_body_adds_a_separated_block() {
        assert_eq!(append_body("intro\n", "more"), "intro\n\nmore\n");
        // Appending to an empty body just sets it.
        assert_eq!(append_body("", "first"), "first\n");
    }

    #[test]
    fn set_section_replaces_content_keeps_heading_and_siblings() {
        let out = set_section(DOC, "Rates", "new one\nnew two").unwrap();
        assert!(out.contains("## Rates\n\nnew one\nnew two\n\n## Next"));
        assert!(!out.contains("old body"));
        // Untouched sections survive.
        assert!(out.contains("# Title\n\nintro"));
        assert!(out.contains("## Next\n\ntail"));
    }

    #[test]
    fn set_section_matches_by_slug() {
        let out = set_section(DOC, "rates", "x").unwrap();
        assert!(out.contains("## Rates\n\nx\n"));
    }

    #[test]
    fn append_section_inserts_after_existing_content() {
        let out = append_section(DOC, "Rates", "added").unwrap();
        assert!(out.contains("## Rates\n\nold body\n\nadded\n\n## Next"));
    }

    #[test]
    fn remove_section_drops_heading_and_body() {
        let out = remove_section(DOC, "Rates").unwrap();
        assert!(!out.contains("## Rates"));
        assert!(!out.contains("old body"));
        assert!(out.contains("## Next\n\ntail"));
    }

    #[test]
    fn missing_heading_is_a_usage_error() {
        assert!(set_section(DOC, "nope", "x").is_err());
        assert!(append_section(DOC, "nope", "x").is_err());
        assert!(remove_section(DOC, "nope").is_err());
    }

    #[test]
    fn section_edit_includes_nested_subsections() {
        let doc = "## A\n\nbody\n\n### deep\n\nx\n\n## B\n\ny\n";
        let out = remove_section(doc, "A").unwrap();
        assert!(!out.contains("### deep"));
        assert!(out.contains("## B"));
    }
}
