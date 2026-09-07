//! docs --format html|md|pdf|graphml|obsidian|index.
use crate::cli::DocsArgs;
use crate::output;
use okf_core::bundle::loader::load_bundle;
use okf_core::bundle::resolve::resolve_bundle;
use okf_core::error::{OkfError, Result};
use okf_core::render::docs::{render_docs, DocsFormat};
use okf_core::render::index::write_indexes;
use serde_json::json;

/// `okf docs [bundle] --format <fmt>`. Write formats (`index`) modify the bundle; the others
/// emit an artifact string to stdout.
pub fn run_docs(args: &DocsArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let format = DocsFormat::parse(&args.format).ok_or_else(|| {
        OkfError::Usage(format!(
            "unknown docs format {:?}: expected md, html, pdf, graphml, obsidian, or index",
            args.format
        ))
    })?;

    if format.is_write() {
        // `index` writes index.md files into the bundle.
        let written = write_indexes(&bundle)?;
        if json {
            for p in &written {
                output::print_line(&json!({
                    "kind": "change",
                    "op": "docs-index",
                    "path": p.to_string_lossy(),
                }))?;
            }
        } else {
            println!("wrote {} index.md file(s)", written.len());
            for p in &written {
                println!("  {}", p.display());
            }
        }
        return Ok(0);
    }

    let artifact = render_docs(&bundle, format)?;
    print!("{artifact}");
    Ok(0)
}
