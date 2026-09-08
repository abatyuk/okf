//! search, list, show, backlinks, graph, resolve.
use crate::cli::{BundleArgs, GraphArgs, IdArgs, ResolveArgs, SearchArgs};
use crate::output;
use okf_core::bundle::loader::load_bundle;
use okf_core::bundle::resolve::resolve_bundle;
use okf_core::error::{OkfError, Result};
use okf_core::graph::backlinks::backlinks_of;
use okf_core::graph::build::build_graph;
use okf_core::graph::render::{render, RenderFormat};
use okf_core::model::concept::Concept;
use okf_core::ontology::load::try_load;
use okf_core::output::record::yaml_to_json;
use okf_core::query::resolve::resolve as resolve_link;
use okf_core::query::search::{search, SearchFilter};
use okf_core::query::show::show;
use serde_json::json;

/// `okf list [bundle]` — list all concepts (search with no filter).
pub fn run_list(args: &BundleArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let results = search(&bundle, &SearchFilter::default());
    output::print_concepts(&results, json)?;
    Ok(0)
}

/// `okf search [bundle] [--type] [--tag] [--text] [--field k=v]`.
pub fn run_search(args: &SearchArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let filter = SearchFilter {
        type_: args.type_.clone(),
        tag: args.tag.clone(),
        text: args.text.clone(),
    };
    let mut results = search(&bundle, &filter);

    // `--field key=value` is a CLI-side post-filter (core `SearchFilter` has no field slot).
    let field_filters = parse_field_filters(&args.field)?;
    if !field_filters.is_empty() {
        results.retain(|c| field_filters.iter().all(|(k, v)| field_matches(c, k, v)));
    }

    output::print_concepts(&results, json)?;
    Ok(0)
}

/// `okf show <concept> [bundle]`.
pub fn run_show(args: &IdArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    match show(&bundle, &args.concept) {
        Some(concept) => {
            output::print_concept(concept, json)?;
            Ok(0)
        }
        None => Err(OkfError::Usage(format!(
            "concept not found: {}",
            args.concept
        ))),
    }
}

/// `okf backlinks <concept> [bundle]` — concepts that link to the given concept.
pub fn run_backlinks(args: &IdArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let ontology = try_load(&root)?;
    let ids = backlinks_of(&bundle, ontology.as_ref(), &args.concept);
    let concepts: Vec<&Concept> = ids.iter().filter_map(|id| bundle.get(&id.0)).collect();
    output::print_concepts(&concepts, json)?;
    Ok(0)
}

/// `okf graph [bundle] [subtree] --format`. Emits a graph artifact (not NDJSON).
pub fn run_graph(args: &GraphArgs, _json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let ontology = try_load(&root)?;
    let format = RenderFormat::parse(&args.format).ok_or_else(|| {
        OkfError::Usage(format!(
            "unknown graph format {:?}: expected mermaid, dot, or graphml",
            args.format
        ))
    })?;
    let graph = build_graph(&bundle, ontology.as_ref());
    let out = render(&graph, format, args.subtree.as_deref());
    print!("{out}");
    Ok(0)
}

/// `okf resolve <link> [bundle] [--from <ctx>]`.
pub fn run_resolve(args: &ResolveArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let r = resolve_link(&bundle, args.from.as_deref(), &args.link);
    if json {
        output::print_line(&json!({
            "kind": "resolved",
            "id": r.id.0,
            "path": r.path.to_string_lossy(),
            "exists": r.exists,
        }))?;
    } else {
        println!(
            "{}\t{}\t{}",
            r.id.0,
            r.path.display(),
            if r.exists { "exists" } else { "missing" }
        );
    }
    Ok(0)
}

/// Parse `--field key=value` arguments.
fn parse_field_filters(raw: &[String]) -> Result<Vec<(String, String)>> {
    raw.iter()
        .map(|f| match f.split_once('=') {
            Some((k, v)) if !k.trim().is_empty() => Ok((k.trim().to_string(), v.to_string())),
            _ => Err(OkfError::Usage(format!(
                "invalid --field {f:?}: expected key=value"
            ))),
        })
        .collect()
}

/// Whether a concept's frontmatter `key` holds (or, for a sequence, contains) `value`.
fn field_matches(concept: &Concept, key: &str, value: &str) -> bool {
    let Some(val) = concept.frontmatter.get(key) else {
        return false;
    };
    match yaml_to_json(val) {
        serde_json::Value::Array(items) => items.iter().any(|i| json_scalar_eq(i, value)),
        other => json_scalar_eq(&other, value),
    }
}

fn json_scalar_eq(v: &serde_json::Value, want: &str) -> bool {
    match v {
        serde_json::Value::String(s) => s == want,
        serde_json::Value::Bool(b) => b.to_string() == want,
        serde_json::Value::Number(n) => n.to_string() == want,
        _ => false,
    }
}
