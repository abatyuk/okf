//! Deterministic concept search. List is search with no filters.
use crate::bundle::loader::Bundle;
use crate::model::concept::Concept;
use pulldown_cmark::{Event, Parser, TagEnd};
use serde_yaml::Value;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextMatchMode {
    #[default]
    Phrase,
    All,
    Any,
    Literal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchField {
    Id,
    Title,
    Description,
    Body,
    Frontmatter,
}

impl SearchField {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Title => "title",
            Self::Description => "description",
            Self::Body => "body",
            Self::Frontmatter => "frontmatter",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SearchSort {
    #[default]
    Relevance,
    Id,
}

#[derive(Debug, Clone)]
pub struct SearchFilter {
    pub type_: Option<String>,
    pub tag: Option<String>,
    pub text: Vec<String>,
    pub match_mode: TextMatchMode,
    pub fields: Vec<SearchField>,
    pub sort: SearchSort,
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self {
            type_: None,
            tag: None,
            text: Vec::new(),
            match_mode: TextMatchMode::Phrase,
            fields: vec![
                SearchField::Id,
                SearchField::Title,
                SearchField::Description,
                SearchField::Body,
            ],
            sort: SearchSort::Relevance,
        }
    }
}

impl SearchFilter {
    pub fn is_empty(&self) -> bool {
        self.type_.is_none() && self.tag.is_none() && self.text.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub query: String,
    pub field: String,
    /// One-based serialized document line; absent for metadata fields.
    pub line: Option<usize>,
    pub snippet: String,
}

#[derive(Debug)]
pub struct SearchHit<'a> {
    pub concept: &'a Concept,
    pub score: u32,
    pub matches: Vec<SearchMatch>,
}

#[derive(Debug)]
struct QueryUnit {
    display: String,
    raw: String,
    prose: String,
}

#[derive(Debug, Default, Clone)]
struct MappedText {
    text: String,
    /// Source byte offset for every byte in text.
    origins: Vec<usize>,
}

impl MappedText {
    fn raw(text: &str) -> Self {
        let mut mapped = Self::default();
        for (offset, ch) in text.char_indices() {
            mapped.push_char(ch, offset);
        }
        mapped
    }

    fn push_str(&mut self, text: &str, origin: usize) {
        self.text.push_str(text);
        self.origins.extend(std::iter::repeat_n(origin, text.len()));
    }

    fn push_char(&mut self, ch: char, origin: usize) {
        self.text.push(ch);
        self.origins
            .extend(std::iter::repeat_n(origin, ch.len_utf8()));
    }

    fn folded(&self) -> FoldedText<'_> {
        let mut text = String::with_capacity(self.text.len());
        let mut display_offsets = Vec::with_capacity(self.text.len());
        for (display_offset, ch) in self.text.char_indices() {
            for folded in ch.to_lowercase() {
                text.push(folded);
                display_offsets.extend(std::iter::repeat_n(display_offset, folded.len_utf8()));
            }
        }
        FoldedText {
            text,
            display: self,
            display_offsets,
        }
    }
}

struct FoldedText<'a> {
    text: String,
    display: &'a MappedText,
    display_offsets: Vec<usize>,
}

#[derive(Debug)]
struct LocatedMatch {
    source_offset: usize,
    snippet: String,
}

/// Collapse inline whitespace and soft wraps, retaining blank-line paragraph boundaries.
fn normalize_prose(text: &MappedText) -> MappedText {
    let mut normalized = MappedText::default();
    let mut chars = text.text.char_indices().peekable();
    let mut pending_whitespace = false;
    let mut whitespace_origin = 0usize;
    let mut line_endings = 0usize;

    while let Some((offset, ch)) = chars.next() {
        if ch.is_whitespace() {
            if !pending_whitespace {
                whitespace_origin = text.origins[offset];
            }
            pending_whitespace = true;
            match ch {
                '\n' => line_endings += 1,
                '\r' => {
                    line_endings += 1;
                    if chars.peek().is_some_and(|(_, next)| *next == '\n') {
                        chars.next();
                    }
                }
                _ => {}
            }
            continue;
        }

        if pending_whitespace && !normalized.text.is_empty() {
            normalized.push_char(
                if line_endings >= 2 { '\n' } else { ' ' },
                whitespace_origin,
            );
        }
        pending_whitespace = false;
        line_endings = 0;
        normalized.push_char(ch, text.origins[offset]);
    }
    normalized
}

/// Extract reader-visible Markdown while keeping approximate source offsets.
fn markdown_text(source: &str) -> MappedText {
    let mut visible = MappedText::default();
    for (event, range) in Parser::new(source).into_offset_iter() {
        match event {
            Event::Text(text) | Event::Code(text) => visible.push_str(&text, range.start),
            Event::SoftBreak | Event::HardBreak => visible.push_char(' ', range.start),
            Event::FootnoteReference(label) => visible.push_str(&label, range.start),
            Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::CodeBlock
                | TagEnd::Item
                | TagEnd::FootnoteDefinition
                | TagEnd::TableCell
                | TagEnd::TableRow,
            )
            | Event::Rule => visible.push_str("\n\n", range.start),
            _ => {}
        }
    }
    visible
}

fn query_units(filter: &SearchFilter) -> Vec<QueryUnit> {
    let values: Vec<String> = match filter.match_mode {
        TextMatchMode::Phrase | TextMatchMode::Literal => filter.text.clone(),
        TextMatchMode::All | TextMatchMode::Any => filter
            .text
            .iter()
            .flat_map(|value| value.split_whitespace().map(str::to_string))
            .collect(),
    };

    values
        .into_iter()
        .filter_map(|display| {
            let raw = display.to_lowercase();
            let prose = normalize_prose(&MappedText::raw(&raw)).text;
            (!raw.is_empty()).then_some(QueryUnit {
                display,
                raw,
                prose,
            })
        })
        .collect()
}

fn locate(view: &MappedText, needle: &str) -> Option<LocatedMatch> {
    if needle.is_empty() {
        return None;
    }
    let folded = view.folded();
    let folded_offset = folded.text.find(needle)?;
    let display_offset = folded.display_offsets[folded_offset];
    let source_offset = folded.display.origins[display_offset];
    Some(LocatedMatch {
        source_offset,
        snippet: snippet(&folded.display.text, display_offset),
    })
}

fn locate_in_field(
    raw: &str,
    markdown: bool,
    unit: &QueryUnit,
    literal: bool,
) -> Option<LocatedMatch> {
    let raw_view = MappedText::raw(raw);
    if let Some(found) = locate(&raw_view, &unit.raw) {
        return Some(found);
    }
    if literal {
        return None;
    }
    if let Some(found) = locate(&normalize_prose(&raw_view), &unit.prose) {
        return Some(found);
    }
    markdown
        .then(|| normalize_prose(&markdown_text(raw)))
        .and_then(|visible| locate(&visible, &unit.prose))
}

fn snippet(text: &str, match_offset: usize) -> String {
    const BEFORE: usize = 48;
    const AFTER: usize = 96;
    let chars: Vec<char> = text.chars().collect();
    let at = text[..match_offset].chars().count();
    let start = at.saturating_sub(BEFORE);
    let end = (at + AFTER).min(chars.len());
    let mut result: String = chars[start..end].iter().collect();
    result = result.split_whitespace().collect::<Vec<_>>().join(" ");
    if start > 0 {
        result.insert(0, '…');
    }
    if end < chars.len() {
        result.push('…');
    }
    result
}

fn body_line(body: &str, source_offset: usize) -> usize {
    body.as_bytes()[..source_offset.min(body.len())]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

fn yaml_text(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        _ => serde_yaml::to_string(value)
            .unwrap_or_default()
            .trim()
            .to_string(),
    }
}

fn relevance(field: SearchField, candidate: &str, unit: &QueryUnit) -> u32 {
    let candidate = normalize_prose(&MappedText::raw(&candidate.to_lowercase())).text;
    let (candidate, query) = if field == SearchField::Id {
        (
            candidate.trim_start_matches('/'),
            unit.prose.trim_start_matches('/'),
        )
    } else {
        (candidate.as_str(), unit.prose.as_str())
    };
    let exact = candidate == query;
    let prefix = candidate.starts_with(query);
    match field {
        SearchField::Id if exact => 1_200,
        SearchField::Title if exact => 1_100,
        SearchField::Title if prefix => 1_000,
        SearchField::Id if prefix => 950,
        SearchField::Title => 900,
        SearchField::Id => 850,
        SearchField::Description => 600,
        SearchField::Frontmatter => 400,
        SearchField::Body => 300,
    }
}

struct Candidate<'a> {
    field: SearchField,
    label: String,
    value: &'a str,
    markdown: bool,
    body: Option<(&'a str, usize)>,
}

fn add_candidate(
    candidates: &mut Vec<(SearchMatch, u32)>,
    candidate: Candidate<'_>,
    unit: &QueryUnit,
    literal: bool,
) {
    if let Some(found) = locate_in_field(candidate.value, candidate.markdown, unit, literal) {
        candidates.push((
            SearchMatch {
                query: unit.display.clone(),
                field: candidate.label,
                line: candidate
                    .body
                    .map(|(body, line_offset)| body_line(body, found.source_offset) + line_offset),
                snippet: found.snippet,
            },
            relevance(candidate.field, candidate.value, unit),
        ));
    }
}

fn best_match(
    concept: &Concept,
    filter: &SearchFilter,
    unit: &QueryUnit,
    body_line_offset: usize,
) -> Option<(SearchMatch, u32)> {
    let literal = filter.match_mode == TextMatchMode::Literal;
    let mut candidates: Vec<(SearchMatch, u32)> = Vec::new();

    for field in &filter.fields {
        match field {
            SearchField::Id => add_candidate(
                &mut candidates,
                Candidate {
                    field: *field,
                    label: field.as_str().to_string(),
                    value: &concept.id.0,
                    markdown: false,
                    body: None,
                },
                unit,
                literal,
            ),
            SearchField::Title => add_candidate(
                &mut candidates,
                Candidate {
                    field: *field,
                    label: field.as_str().to_string(),
                    value: concept.title().unwrap_or(""),
                    markdown: false,
                    body: None,
                },
                unit,
                literal,
            ),
            SearchField::Description => add_candidate(
                &mut candidates,
                Candidate {
                    field: *field,
                    label: field.as_str().to_string(),
                    value: concept.description().unwrap_or(""),
                    markdown: false,
                    body: None,
                },
                unit,
                literal,
            ),
            SearchField::Body => add_candidate(
                &mut candidates,
                Candidate {
                    field: *field,
                    label: field.as_str().to_string(),
                    value: &concept.body,
                    markdown: true,
                    body: Some((&concept.body, body_line_offset)),
                },
                unit,
                literal,
            ),
            SearchField::Frontmatter => {
                for (key, value) in &concept.frontmatter.map {
                    let value = yaml_text(value);
                    add_candidate(
                        &mut candidates,
                        Candidate {
                            field: *field,
                            label: format!("frontmatter.{key}"),
                            value: &value,
                            markdown: false,
                            body: None,
                        },
                        unit,
                        literal,
                    );
                }
            }
        }
    }

    candidates
        .into_iter()
        .max_by(|(left_match, left_score), (right_match, right_score)| {
            left_score
                .cmp(right_score)
                .then_with(|| right_match.field.cmp(&left_match.field))
        })
}

fn body_line_offset(concept: &Concept) -> usize {
    if concept.frontmatter.map.is_empty() {
        2
    } else {
        serde_yaml::to_string(&concept.frontmatter.map)
            .map(|frontmatter| frontmatter.lines().count() + 2)
            .unwrap_or(2)
    }
}

fn structured_match(concept: &Concept, filter: &SearchFilter) -> bool {
    if let Some(ty) = &filter.type_ {
        if concept.concept_type() != Some(ty.as_str()) {
            return false;
        }
    }
    if let Some(tag) = &filter.tag {
        if !concept.has_tag(tag) {
            return false;
        }
    }
    true
}

/// Return matching concepts with deterministic relevance and bounded match evidence.
pub fn search_detailed<'a>(bundle: &'a Bundle, filter: &SearchFilter) -> Vec<SearchHit<'a>> {
    let units = query_units(filter);
    let mut hits: Vec<SearchHit<'a>> = bundle
        .concepts
        .iter()
        .filter(|concept| structured_match(concept, filter))
        .filter_map(|concept| {
            let body_line_offset =
                if !units.is_empty() && filter.fields.contains(&SearchField::Body) {
                    body_line_offset(concept)
                } else {
                    0
                };
            let matched: Vec<(SearchMatch, u32)> = units
                .iter()
                .filter_map(|unit| best_match(concept, filter, unit, body_line_offset))
                .collect();
            let accepted = if units.is_empty() {
                filter.text.is_empty()
            } else {
                match filter.match_mode {
                    TextMatchMode::Any => !matched.is_empty(),
                    TextMatchMode::Phrase | TextMatchMode::All | TextMatchMode::Literal => {
                        matched.len() == units.len()
                    }
                }
            };
            accepted.then(|| SearchHit {
                concept,
                score: matched.iter().map(|(_, score)| score).sum(),
                matches: matched.into_iter().map(|(evidence, _)| evidence).collect(),
            })
        })
        .collect();

    match filter.sort {
        SearchSort::Relevance => hits.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.concept.id.0.cmp(&right.concept.id.0))
        }),
        SearchSort::Id => hits.sort_by(|left, right| left.concept.id.0.cmp(&right.concept.id.0)),
    }
    hits
}

/// Return matching concepts without match evidence.
pub fn search<'a>(bundle: &'a Bundle, filter: &SearchFilter) -> Vec<&'a Concept> {
    search_detailed(bundle, filter)
        .into_iter()
        .map(|hit| hit.concept)
        .collect()
}

pub fn list(bundle: &Bundle) -> Vec<&Concept> {
    bundle.concepts.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::{
        markdown_text, normalize_prose, search_detailed, MappedText, SearchField, SearchFilter,
        SearchSort, TextMatchMode,
    };
    use crate::bundle::loader::load_bundle;
    use std::path::PathBuf;

    fn fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample-bundle")
    }

    #[test]
    fn prose_normalization_collapses_whitespace_but_preserves_paragraphs() {
        assert_eq!(
            normalize_prose(&MappedText::raw("  two\t \nword  ")).text,
            "two word"
        );
        assert_eq!(
            normalize_prose(&MappedText::raw("two\r\n  \r\nword")).text,
            "two\nword"
        );
    }

    #[test]
    fn markdown_text_removes_inline_markup_and_preserves_blocks() {
        assert_eq!(
            normalize_prose(&markdown_text("travel **policy**\n\nnext paragraph")).text,
            "travel policy\nnext paragraph"
        );
    }

    #[test]
    fn match_modes_scopes_and_relevance_are_deterministic() {
        let bundle = load_bundle(&fixture()).unwrap();
        let all = search_detailed(
            &bundle,
            &SearchFilter {
                text: vec!["travel reimbursement".into()],
                match_mode: TextMatchMode::All,
                fields: vec![SearchField::Body],
                ..Default::default()
            },
        );
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].concept.id.0, "/policies/travel");
        assert_eq!(all[0].matches.len(), 2);

        let title_only = search_detailed(
            &bundle,
            &SearchFilter {
                text: vec!["reimbursement".into()],
                fields: vec![SearchField::Title],
                ..Default::default()
            },
        );
        assert!(title_only.is_empty());

        let whitespace_only = search_detailed(
            &bundle,
            &SearchFilter {
                text: vec!["   ".into()],
                ..Default::default()
            },
        );
        assert!(whitespace_only.is_empty());

        let by_id = search_detailed(
            &bundle,
            &SearchFilter {
                sort: SearchSort::Id,
                ..Default::default()
            },
        );
        assert!(by_id
            .windows(2)
            .all(|pair| pair[0].concept.id.0 < pair[1].concept.id.0));
    }
}
