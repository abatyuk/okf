//! Split frontmatter/body; build the heading tree; extract sections.
//!
//! Beyond the frontmatter/body split, this module hosts the shared ATX-heading primitives
//! (`parse_atx`, `slugify`) used by both `markdown-heading` fingerprints and section-aware
//! `okf edit`, plus [`find_section`], which locates a heading's line span for lossless
//! splicing.

/// Split a concept file's leading `---\n … \n---\n` frontmatter block from its body.
///
/// Returns `(Some(frontmatter_text), body)` when a well-formed leading block is present,
/// otherwise `(None, whole_content)`. The body is preserved verbatim (everything after the
/// closing delimiter line's newline). Frontmatter text excludes both delimiter lines.
pub fn split_frontmatter(content: &str) -> (Option<String>, String) {
    // The block must open on the very first line.
    let first_line_end = content.find('\n').map(|i| i + 1).unwrap_or(content.len());
    let first_line = &content[..first_line_end];
    if first_line.trim_end() != "---" {
        return (None, content.to_string());
    }

    let open_end = first_line_end;
    let mut pos = open_end;
    while pos < content.len() {
        let nl = content[pos..]
            .find('\n')
            .map(|i| pos + i + 1)
            .unwrap_or(content.len());
        let line = &content[pos..nl];
        if line.trim_end() == "---" {
            let fm = content[open_end..pos].to_string();
            let body = content[nl..].to_string();
            return (Some(fm), body);
        }
        pos = nl;
    }

    // No closing delimiter — be permissive and treat everything as body.
    (None, content.to_string())
}

/// A located ATX-heading section within a body, expressed in `body.split('\n')` line
/// indices so callers can splice losslessly (rejoining with `\n` is exact).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SectionSpan {
    /// ATX level (1–6) of the matched heading.
    pub level: usize,
    /// Index of the heading line itself.
    pub heading_line: usize,
    /// Index of the first content line after the heading (`heading_line + 1`).
    pub content_start: usize,
    /// Exclusive end index: the next equal-or-shallower heading, or the line count.
    pub end: usize,
}

/// Locate the first heading whose slug equals `slugify(heading)`, honoring fenced code blocks
/// (ATX headings inside ```` ``` ````/`~~~` fences are ignored). Returns line indices into
/// `body.split('\n')`; `None` if no heading matches. Nested subsections are part of the span.
pub(crate) fn find_section(body: &str, heading: &str) -> Option<SectionSpan> {
    let want = slugify(heading);
    let lines: Vec<&str> = body.split('\n').collect();
    let mut in_fence = false;
    for i in 0..lines.len() {
        if is_fence(lines[i]) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some((level, htext)) = parse_atx(lines[i]) {
            if slugify(htext) == want {
                let end = section_end(&lines, i + 1, level);
                return Some(SectionSpan {
                    level,
                    heading_line: i,
                    content_start: i + 1,
                    end,
                });
            }
        }
    }
    None
}

/// The exclusive end of a section that starts after `from`: the first equal-or-shallower ATX
/// heading (fences honored), or `lines.len()`.
fn section_end(lines: &[&str], from: usize, level: usize) -> usize {
    let mut in_fence = false;
    for (j, line) in lines.iter().enumerate().skip(from) {
        if is_fence(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some((lvl, _)) = parse_atx(line) {
            if lvl <= level {
                return j;
            }
        }
    }
    lines.len()
}

/// True for a code-fence line (```` ``` ```` or `~~~`, possibly indented).
fn is_fence(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("```") || t.starts_with("~~~")
}

/// Parse an ATX heading line, returning `(level, text)`. Requires 1–6 leading `#` followed by
/// whitespace (or an otherwise-empty heading); the trailing ATX closing `#` run is trimmed.
pub(crate) fn parse_atx(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &trimmed[hashes..];
    if rest.is_empty() {
        return Some((hashes, ""));
    }
    if !rest.starts_with([' ', '\t']) {
        return None; // e.g. "#hashtag" is not a heading.
    }
    let text = rest.trim().trim_end_matches('#').trim_end();
    Some((hashes, text))
}

/// GitHub-style heading slug: lowercase; alphanumerics kept; spaces/`-`/`_` → `-`; other
/// punctuation dropped; consecutive/edge hyphens collapsed.
pub(crate) fn slugify(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() {
            for lc in c.to_lowercase() {
                out.push(lc);
            }
        } else if c == ' ' || c == '-' || c == '_' {
            out.push('-');
        }
    }
    let mut slug = String::with_capacity(out.len());
    let mut prev_dash = false;
    for c in out.chars() {
        if c == '-' {
            if !prev_dash {
                slug.push('-');
            }
            prev_dash = true;
        } else {
            slug.push(c);
            prev_dash = false;
        }
    }
    slug.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_frontmatter_and_body() {
        let (fm, body) = split_frontmatter("---\ntype: table\n---\nhello\n");
        assert_eq!(fm.as_deref(), Some("type: table\n"));
        assert_eq!(body, "hello\n");
    }

    #[test]
    fn no_frontmatter() {
        let (fm, body) = split_frontmatter("just body\n");
        assert!(fm.is_none());
        assert_eq!(body, "just body\n");
    }

    const DOC: &str = "# Title\n\nintro\n\n## Rates\n\nbody one\n\n### Nested\n\ndeep\n\n## Next\n\ntail\n";

    #[test]
    fn find_section_spans_nested_stops_at_equal_level() {
        let s = find_section(DOC, "Rates").unwrap();
        assert_eq!(s.level, 2);
        let lines: Vec<&str> = DOC.split('\n').collect();
        assert_eq!(lines[s.heading_line], "## Rates");
        // Section content runs through the nested block up to (excluding) "## Next".
        assert_eq!(lines[s.end], "## Next");
        assert!(lines[s.content_start..s.end].contains(&"### Nested"));
    }

    #[test]
    fn find_section_matches_by_slug_and_misses_cleanly() {
        assert_eq!(find_section(DOC, "rates").unwrap().level, 2);
        assert!(find_section(DOC, "does-not-exist").is_none());
    }

    #[test]
    fn find_section_ignores_headings_in_fences() {
        let doc = "## Sec\n\n```\n## fake\n```\n\n## Real\n";
        let s = find_section(doc, "Real").unwrap();
        let lines: Vec<&str> = doc.split('\n').collect();
        assert_eq!(lines[s.heading_line], "## Real");
    }

    #[test]
    fn slugify_matches_github_style() {
        assert_eq!(slugify("Reimbursement Rates"), "reimbursement-rates");
        assert_eq!(slugify("Foo: Bar (baz)!"), "foo-bar-baz");
    }
}
