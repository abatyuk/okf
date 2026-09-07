//! Render core results as human text or NDJSON (`--json`).
//!
//! NDJSON records are `serde_json::Value` objects (built with `preserve_order`), one per
//! output line. A `concept` record mirrors the concept's frontmatter verbatim plus computed
//! `id`/`trust_tier` (via [`concept_record`]); other record types have documented shapes.
use okf_core::error::Result;
use okf_core::model::concept::Concept;
use okf_core::output::ndjson;
use okf_core::output::record::concept_record;
use okf_core::parse::writer;
use serde_json::Value as Json;

/// Print one NDJSON record (object per line).
pub fn print_line(value: &Json) -> Result<()> {
    print!("{}", ndjson::to_line(value)?);
    println!();
    Ok(())
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

    let id_w = concepts.iter().map(|c| c.id.0.len()).max().unwrap_or(2).max(2);
    let ty_w = concepts
        .iter()
        .map(|c| c.concept_type().unwrap_or("-").len())
        .max()
        .unwrap_or(4)
        .max(4);
    let tier_w = 17; // widest tier string, "machine-confirmed"

    println!("{:<id_w$}  {:<ty_w$}  {:<tier_w$}  {}", "ID", "TYPE", "TRUST", "TITLE");
    for c in concepts {
        println!(
            "{:<id_w$}  {:<ty_w$}  {:<tier_w$}  {}",
            c.id.0,
            c.concept_type().unwrap_or("-"),
            c.trust_tier().as_str(),
            c.title().unwrap_or(""),
        );
    }
    Ok(())
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
