//! Render bundle documentation in an external format.
//!
//! [`render_docs`] turns a [`Bundle`] into a single self-contained artifact string. Formats:
//!
//! - **`md`** — a consolidated markdown document: a `Contents` list followed by one section
//!   per concept (metadata + verbatim body), ordered by concept id. *Real.*
//! - **`html`** — the `md` document rendered to a self-contained HTML page (minimal inline
//!   CSS) via `pulldown-cmark`. *Real.*
//! - **`graphml`** — delegates to [`crate::graph::render`] with `RenderFormat::Graphml`
//!   (the link graph, not the prose). *Real.*
//! - **`obsidian`** — a consolidated markdown variant whose cross-references use Obsidian
//!   `[[wikilink]]` syntax (keyed by concept id), suitable for pasting into a vault. *Real.*
//! - **`pdf`** — returns a clear error; PDF needs an external renderer and is not in v1. The
//!   [`DocsFormat::Pdf`] arm is the documented seam where a `docs-pdf` feature would hook in.
//! - **`index`** — not an artifact: it *writes* `index.md` files into the bundle. Handled by
//!   [`crate::render::index::write_indexes`]; `render_docs` rejects it with guidance.
//!
//! All artifact outputs are deterministic: concepts are already sorted by id in the bundle,
//! and the graph emitters sort their nodes/edges.
use crate::bundle::loader::Bundle;
use crate::error::{OkfError, Result};
use crate::graph::build::build_graph;
use crate::graph::render::{render as graph_render, RenderFormat};
use crate::model::concept::Concept;
use crate::model::link::outbound_links;
use crate::ontology::schema::Ontology;

/// A documentation output format for `okf docs --format …`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocsFormat {
    /// Consolidated markdown document.
    Md,
    /// Self-contained HTML page.
    Html,
    /// GraphML of the link graph.
    Graphml,
    /// Consolidated markdown with Obsidian `[[wikilinks]]`.
    Obsidian,
    /// Not available in v1 (documented seam).
    Pdf,
    /// Write `index.md` files into the bundle (handled by [`crate::render::index`]).
    Index,
}

impl DocsFormat {
    /// Parse a `--format` value, case-insensitive. `markdown` is accepted for `md`.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Some(DocsFormat::Md),
            "html" => Some(DocsFormat::Html),
            "graphml" => Some(DocsFormat::Graphml),
            "obsidian" => Some(DocsFormat::Obsidian),
            "pdf" => Some(DocsFormat::Pdf),
            "index" => Some(DocsFormat::Index),
            _ => None,
        }
    }

    /// Stable format name.
    pub fn as_str(&self) -> &'static str {
        match self {
            DocsFormat::Md => "md",
            DocsFormat::Html => "html",
            DocsFormat::Graphml => "graphml",
            DocsFormat::Obsidian => "obsidian",
            DocsFormat::Pdf => "pdf",
            DocsFormat::Index => "index",
        }
    }

    /// Whether this format writes files into the bundle (`index`) rather than returning an
    /// artifact string. The CLI should route write formats to
    /// [`crate::render::index::write_indexes`] instead of [`render_docs`].
    pub fn is_write(&self) -> bool {
        matches!(self, DocsFormat::Index)
    }
}

/// Render the bundle's documentation as an artifact string in `format`.
///
/// Returns an error for `pdf` (unsupported in v1) and for `index` (a write format — call
/// [`crate::render::index::write_indexes`]).
pub fn render_docs(
    bundle: &Bundle,
    ontology: Option<&Ontology>,
    format: DocsFormat,
) -> Result<String> {
    match format {
        DocsFormat::Md => Ok(render_markdown(bundle)),
        DocsFormat::Obsidian => Ok(render_obsidian(bundle, ontology)),
        DocsFormat::Html => Ok(render_html(bundle)),
        DocsFormat::Graphml => Ok(render_graphml(bundle, ontology)),
        DocsFormat::Pdf => Err(OkfError::Usage(
            "pdf requires the `docs-pdf` feature / external renderer; not available in v1"
                .to_string(),
        )),
        DocsFormat::Index => Err(OkfError::Usage(
            "index writes index.md files into the bundle; call render::index::write_indexes"
                .to_string(),
        )),
    }
}

/// Title to show for a concept: its `title`, else its id.
fn concept_title(c: &Concept) -> &str {
    c.title().unwrap_or_else(|| c.id.as_str())
}

/// One concept's markdown section: an `##` heading, a metadata list, then the verbatim body.
fn concept_section(c: &Concept, out: &mut String) {
    out.push_str(&format!("## {}\n\n", concept_title(c)));
    out.push_str(&format!("- **ID:** `{}`\n", c.id.as_str()));
    if let Some(ty) = c.concept_type() {
        out.push_str(&format!("- **Type:** {ty}\n"));
    }
    out.push_str(&format!("- **Trust tier:** {}\n", c.trust_tier().as_str()));
    if let Some(d) = c.description() {
        out.push_str(&format!("- **Description:** {d}\n"));
    }
    out.push('\n');
    let body = c.body.trim();
    if !body.is_empty() {
        out.push_str(body);
        out.push_str("\n\n");
    }
}

/// Consolidated markdown for the whole bundle.
fn render_markdown(bundle: &Bundle) -> String {
    let mut out = String::from("# OKF Documentation\n\n");
    out.push_str(&format!("{} concept(s).\n\n", bundle.concepts.len()));

    out.push_str("## Contents\n\n");
    for c in &bundle.concepts {
        match c.concept_type() {
            Some(ty) if !ty.is_empty() => out.push_str(&format!("* {} — {ty}\n", concept_title(c))),
            _ => out.push_str(&format!("* {}\n", concept_title(c))),
        }
    }
    out.push('\n');

    for c in &bundle.concepts {
        out.push_str("---\n\n");
        concept_section(c, &mut out);
    }
    out
}

/// Consolidated markdown with Obsidian-style `[[wikilinks]]`. Contents entries and each
/// concept's outbound references are emitted as wikilinks keyed by concept id, so splitting
/// the document into per-note files in a vault keeps the links resolvable.
fn render_obsidian(bundle: &Bundle, ontology: Option<&Ontology>) -> String {
    let mut out = String::from("# OKF Vault Index\n\n");
    out.push_str(
        "> Consolidated bundle rendered for an Obsidian vault. \
         Cross-references use `[[wikilinks]]` keyed by concept id.\n\n",
    );

    out.push_str("## Contents\n\n");
    for c in &bundle.concepts {
        out.push_str(&format!("* [[{}|{}]]\n", c.id.as_str(), concept_title(c)));
    }
    out.push('\n');

    for c in &bundle.concepts {
        out.push_str("---\n\n");
        out.push_str(&format!("## {}\n\n", concept_title(c)));
        out.push_str(&format!("- **ID:** `{}`\n", c.id.as_str()));
        if let Some(ty) = c.concept_type() {
            out.push_str(&format!("- **Type:** {ty}\n"));
        }
        out.push_str(&format!("- **Trust tier:** {}\n", c.trust_tier().as_str()));
        if let Some(d) = c.description() {
            out.push_str(&format!("- **Description:** {d}\n"));
        }
        let links = outbound_links(c, ontology);
        if !links.is_empty() {
            let joined = links
                .iter()
                .map(|l| format!("[[{}]]", l.as_str()))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("- **Links:** {joined}\n"));
        }
        out.push('\n');
        let body = c.body.trim();
        if !body.is_empty() {
            out.push_str(body);
            out.push_str("\n\n");
        }
    }
    out
}

/// Minimal, self-contained page CSS.
const HTML_CSS: &str = "body{font:16px/1.6 -apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;max-width:48rem;margin:2rem auto;padding:0 1rem;color:#1a1a1a}\
h1,h2{line-height:1.25}h2{margin-top:2rem;border-bottom:1px solid #eee;padding-bottom:.2rem}\
code{background:#f4f4f4;padding:.1em .3em;border-radius:3px}hr{border:0;border-top:1px solid #eee;margin:2rem 0}\
a{color:#0645ad}";

/// Render the consolidated markdown to a self-contained HTML page.
fn render_html(bundle: &Bundle) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let md = render_markdown(bundle);
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&md, opts);
    let mut body_html = String::new();
    html::push_html(&mut body_html, parser);

    let mut out = String::from(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n\
         <meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>OKF Documentation</title>\n<style>\n",
    );
    out.push_str(HTML_CSS);
    out.push_str("\n</style>\n</head>\n<body>\n");
    out.push_str(&body_html);
    out.push_str("</body>\n</html>\n");
    out
}

/// Delegate the graph rendering to the shared emitter (do not reimplement).
fn render_graphml(bundle: &Bundle, ontology: Option<&Ontology>) -> String {
    let graph = build_graph(bundle, ontology);
    graph_render(&graph, RenderFormat::Graphml, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::loader::load_bundle;
    use std::path::PathBuf;

    fn sample() -> Bundle {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample-bundle");
        load_bundle(&root).unwrap()
    }

    #[test]
    fn format_parse_roundtrip() {
        for f in [
            DocsFormat::Md,
            DocsFormat::Html,
            DocsFormat::Graphml,
            DocsFormat::Obsidian,
            DocsFormat::Pdf,
            DocsFormat::Index,
        ] {
            assert_eq!(DocsFormat::parse(f.as_str()), Some(f));
        }
        assert_eq!(DocsFormat::parse("MARKDOWN"), Some(DocsFormat::Md));
        assert_eq!(DocsFormat::parse("nope"), None);
        assert!(DocsFormat::Index.is_write());
        assert!(!DocsFormat::Md.is_write());
    }

    #[test]
    fn md_is_nonempty_and_deterministic() {
        let b = sample();
        let a = render_docs(&b, None, DocsFormat::Md).unwrap();
        let c = render_docs(&b, None, DocsFormat::Md).unwrap();
        assert_eq!(a, c, "markdown output must be deterministic");
        assert!(a.starts_with("# OKF Documentation\n"));
        assert!(a.contains("## Contents"));
        // One concept section per concept, ordered by id (customers before revenue).
        assert!(a.contains("## Customers"));
        assert!(a.contains("- **Type:** BigQuery Table"));
        assert!(a.contains("- **Trust tier:** human-reviewed"));
        // Ordered by concept id: /metrics/revenue precedes /tables/customers.
        let cust = a.find("## Customers").unwrap();
        let rev = a.find("## Revenue").unwrap();
        assert!(rev < cust, "sections should follow bundle id order");
    }

    #[test]
    fn html_wraps_rendered_markdown() {
        let b = sample();
        let a = render_docs(&b, None, DocsFormat::Html).unwrap();
        let c = render_docs(&b, None, DocsFormat::Html).unwrap();
        assert_eq!(a, c, "html output must be deterministic");
        assert!(a.starts_with("<!DOCTYPE html>"));
        assert!(a.contains("<title>OKF Documentation</title>"));
        assert!(a.contains("<style>"));
        assert!(a.contains("<h1>OKF Documentation</h1>"));
        assert!(a.contains("<h2>Customers</h2>"));
        assert!(a.trim_end().ends_with("</html>"));
    }

    #[test]
    fn graphml_delegates_to_graph_renderer() {
        let b = sample();
        let a = render_docs(&b, None, DocsFormat::Graphml).unwrap();
        assert!(a.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml"));
        assert!(a.contains("<graph id=\"okf\" edgedefault=\"directed\">"));
        assert_eq!(a, render_docs(&b, None, DocsFormat::Graphml).unwrap());
    }

    #[test]
    fn obsidian_uses_wikilinks() {
        let b = sample();
        let a = render_docs(&b, None, DocsFormat::Obsidian).unwrap();
        assert_eq!(a, render_docs(&b, None, DocsFormat::Obsidian).unwrap());
        assert!(a.contains("[[/tables/customers|Customers]]"));
        assert!(a.contains("[[wikilinks]]"));
    }

    #[test]
    fn pdf_returns_clear_error() {
        let b = sample();
        let err = render_docs(&b, None, DocsFormat::Pdf).unwrap_err();
        assert!(matches!(err, OkfError::Usage(_)));
        assert!(err.to_string().contains("pdf"));
        assert!(err.to_string().contains("not available in v1"));
    }

    #[test]
    fn index_format_is_a_write_not_an_artifact() {
        let b = sample();
        let err = render_docs(&b, None, DocsFormat::Index).unwrap_err();
        assert!(matches!(err, OkfError::Usage(_)));
        assert!(err.to_string().contains("write_indexes"));
    }
}
