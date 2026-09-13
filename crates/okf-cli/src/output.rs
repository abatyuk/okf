//! Render core results as human text or NDJSON (`--json`).
//!
//! NDJSON records are `serde_json::Value` objects (built with `preserve_order`), one per
//! output line. A `concept` record mirrors the concept's frontmatter verbatim plus computed
//! `id`/`trust_tier` (via [`concept_record`]); other record types have documented shapes.
use okf_core::error::Result;
use okf_core::model::concept::Concept;
use okf_core::output::record::concept_record;
use okf_core::parse::writer;
use serde_json::Value as Json;
use std::cell::RefCell;
use std::io::{BufWriter, Write};

thread_local! {
    static STDOUT: RefCell<BufWriter<std::io::Stdout>> =
        RefCell::new(BufWriter::with_capacity(64 * 1024, std::io::stdout()));
}

#[derive(Debug)]
struct Heading<'a> {
    line: usize,
    level: usize,
    text: &'a str,
}

/// Print one NDJSON record (object per line).
pub fn print_line(value: &Json) -> Result<()> {
    STDOUT.with(|output| {
        let mut output = output.borrow_mut();
        serde_json::to_writer(&mut *output, value)
            .map_err(|error| okf_core::error::OkfError::Internal(error.to_string()))?;
        output.write_all(b"\n").map_err(Into::into)
    })
}

pub fn flush() -> Result<()> {
    STDOUT.with(|output| output.borrow_mut().flush().map_err(Into::into))
}

pub fn print_text(args: std::fmt::Arguments<'_>) -> Result<()> {
    STDOUT.with(|output| output.borrow_mut().write_fmt(args).map_err(Into::into))
}

pub fn print_text_line(args: std::fmt::Arguments<'_>) -> Result<()> {
    STDOUT.with(|output| {
        let mut output = output.borrow_mut();
        output.write_fmt(args)?;
        output.write_all(b"\n").map_err(Into::into)
    })
}

/// Print a sequence of NDJSON records.
pub fn print_lines(values: &[Json]) -> Result<()> {
    for v in values {
        print_line(v)?;
    }
    Ok(())
}

/// Print a list of concepts as either NDJSON `concept` records or a human table.
pub fn print_concepts(concepts: &[&Concept], json: bool) -> Result<()> {
    if json {
        for c in concepts {
            print_line(&concept_record(c))?;
        }
        return Ok(());
    }

    if concepts.is_empty() {
        println!("(no concepts)");
        return Ok(());
    }

    let id_w = concepts
        .iter()
        .map(|c| c.id.0.len())
        .max()
        .unwrap_or(2)
        .max(2);
    let ty_w = concepts
        .iter()
        .map(|c| c.concept_type().unwrap_or("-").len())
        .max()
        .unwrap_or(4)
        .max(4);
    let tier_w = 17; // widest tier string, "machine-confirmed"
    let status_w = concepts
        .iter()
        .map(|c| c.effective_status().len())
        .max()
        .unwrap_or(6)
        .max(6);

    let stdout = std::io::stdout();
    let mut output = BufWriter::with_capacity(64 * 1024, stdout.lock());
    writeln!(
        output,
        "{:<id_w$}  {:<ty_w$}  {:<tier_w$}  {}",
        "ID",
        "TYPE",
        "TRUST",
        format_args!("{:<status_w$}  {}", "STATUS", "TITLE")
    )?;
    for c in concepts {
        writeln!(
            output,
            "{:<id_w$}  {:<ty_w$}  {:<tier_w$}  {}",
            c.id.0,
            c.concept_type().unwrap_or("-"),
            c.trust_tier().as_str(),
            format_args!(
                "{:<status_w$}  {}",
                c.effective_status(),
                c.title().unwrap_or("")
            ),
        )?;
    }
    output.flush().map_err(Into::into)
}

/// Print a single concept's full content as an NDJSON record or human text.
pub fn print_concept(concept: &Concept, json: bool) -> Result<()> {
    if json {
        return print_line(&concept_record(concept));
    }
    println!("# {}", concept.id.0);
    println!("trust_tier: {}", concept.trust_tier().as_str());
    println!();
    print!("{}", writer::write_concept(concept)?);
    Ok(())
}

/// Print only the Markdown heading structure, with physical line numbers in the serialized
/// concept document. ATX-like text inside fenced code blocks is ignored.
pub fn print_concept_outline(concept: &Concept, json: bool) -> Result<()> {
    let source = writer::write_concept(concept)?;
    let headings = headings(&source);
    if json {
        let outline: Vec<Json> = headings
            .iter()
            .map(|h| serde_json::json!({"line": h.line, "level": h.level, "text": h.text}))
            .collect();
        return print_line(&serde_json::json!({
            "kind": "outline",
            "id": concept.id.0,
            "trust_tier": concept.trust_tier().as_str(),
            "headings": outline,
        }));
    }
    println!("# {}", concept.id.0);
    println!("trust_tier: {}", concept.trust_tier().as_str());
    println!();
    for h in headings {
        println!("{}: {} {}", h.line, "#".repeat(h.level), h.text);
    }
    Ok(())
}

/// Print an inclusive, 1-based slice of the serialized concept document.
pub fn print_concept_lines(concept: &Concept, start: usize, end: usize, json: bool) -> Result<()> {
    let source = writer::write_concept(concept)?;
    let total = source.lines().count();
    if start > total {
        return Err(okf_core::error::OkfError::Usage(format!(
            "--lines starts at {start}, but {} has only {total} lines",
            concept.id.0
        )));
    }
    let actual_end = end.min(total);
    let selected: Vec<(usize, &str)> = source
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let n = i + 1;
            (n >= start && n <= actual_end).then_some((n, line))
        })
        .collect();
    if json {
        let lines: Vec<Json> = selected
            .iter()
            .map(|(line, text)| serde_json::json!({"line": line, "text": text}))
            .collect();
        return print_line(&serde_json::json!({
            "kind": "line-range",
            "id": concept.id.0,
            "trust_tier": concept.trust_tier().as_str(),
            "start": start,
            "end": actual_end,
            "lines": lines,
        }));
    }
    for (line, text) in selected {
        println!("{line}: {text}");
    }
    Ok(())
}

fn headings(source: &str) -> Vec<Heading<'_>> {
    let mut result = Vec::new();
    let mut in_frontmatter = false;
    let mut frontmatter_closed = false;
    let mut in_fence = false;
    for (i, line) in source.lines().enumerate() {
        if i == 0 && line.trim_end() == "---" {
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter {
            if line.trim_end() == "---" {
                in_frontmatter = false;
                frontmatter_closed = true;
            }
            continue;
        }
        if !frontmatter_closed {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let level = trimmed.chars().take_while(|c| *c == '#').count();
        if level == 0 || level > 6 {
            continue;
        }
        let rest = &trimmed[level..];
        if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
            continue;
        }
        result.push(Heading {
            line: i + 1,
            level,
            text: rest.trim().trim_end_matches('#').trim_end(),
        });
    }
    result
}
