//! search, list, show, backlinks, links, graph, resolve.
use crate::cli::{
    BrowseArgs, BundleArgs, GraphArgs, IdArgs, ResolveArgs, SearchArgs, SearchFieldArg,
    SearchMatchArg, SearchSortArg, ShowArgs,
};
use crate::output;
use okf_core::bundle::loader::{concept_exists, load_bundle, load_bundle_metadata, load_concept};
use okf_core::bundle::resolve::resolve_bundle;
use okf_core::error::{OkfError, Result};
use okf_core::graph::backlinks::backlinks_of;
use okf_core::graph::build::build_graph;
use okf_core::graph::render::{render_neighborhood, GraphDirection, RenderFormat};
use okf_core::model::concept::Concept;
use okf_core::model::link::outbound_links;
use okf_core::ontology::load::try_load;
use okf_core::query::browse::browse;
use okf_core::query::resolve::resolve_at;
use okf_core::query::search::{
    search_detailed, SearchField, SearchFilter, SearchSort, TextMatchMode,
};
use serde_json::json;

/// `okf list [bundle]` — list all concepts (search with no filter).
pub fn run_list(args: &SearchArgs, json: bool) -> Result<i32> {
    run_search(args, json)
}

/// Search the selected primary bundle with typed metadata controls.
pub fn run_search(args: &SearchArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let (settings, _) = okf_core::bundle::settings::load_for(&root)?;
    run_search_in_scope(
        args,
        json,
        &[(
            settings
                .bundle
                .unwrap_or_else(|| format!("path:{}", root.display())),
            root,
            "working-tree".into(),
        )],
    )
}

/// One query-wide scan and page across the explicitly examined bundle/version scope.
pub fn run_search_in_scope(
    args: &SearchArgs,
    json_output: bool,
    scope: &[(String, std::path::PathBuf, String)],
) -> Result<i32> {
    use okf_core::query::metadata::{facet, project, Condition};
    if (args.facets || !args.facet.is_empty()) && !json_output {
        return Err(OkfError::Usage("--facets/--facet require --json".into()));
    }
    if args.offset.is_some() && args.limit.is_none_or(|n| n == 0) {
        return Err(OkfError::Usage("--offset requires positive --limit".into()));
    }
    if args.scan_limit == Some(0) {
        return Err(OkfError::Usage("--scan-limit must be positive".into()));
    }
    if args.expansion_edges == 0
        || args.expansion_edges > 100
        || args.expansion_targets == 0
        || args.expansion_targets > 1000
        || args.expansion_bytes == 0
        || args.expansion_bytes > 1048576
    {
        return Err(OkfError::Usage(
            "expansion bounds must be edges 1..100, targets 1..1000, bytes 1..1048576".into(),
        ));
    }
    let filter = SearchFilter {
        type_: args.type_.clone(),
        tag: args.tag.clone(),
        text: args.text.clone(),
        match_mode: match args.match_mode {
            SearchMatchArg::Phrase => TextMatchMode::Phrase,
            SearchMatchArg::All => TextMatchMode::All,
            SearchMatchArg::Any => TextMatchMode::Any,
            SearchMatchArg::Literal => TextMatchMode::Literal,
        },
        fields: args
            .in_
            .iter()
            .map(|f| match f {
                SearchFieldArg::Id => SearchField::Id,
                SearchFieldArg::Title => SearchField::Title,
                SearchFieldArg::Description => SearchField::Description,
                SearchFieldArg::Body => SearchField::Body,
                SearchFieldArg::Frontmatter => SearchField::Frontmatter,
            })
            .collect(),
        sort: match args.sort {
            SearchSortArg::Id => SearchSort::Id,
            SearchSortArg::Relevance => SearchSort::Relevance,
        },
    };
    let conditions = args
        .facet_filter
        .iter()
        .map(|s| Condition::parse(s))
        .collect::<Result<Vec<_>>>()?;
    let literal = parse_field_filters(&args.field)?;
    let roots: Vec<_> = scope
        .iter()
        .map(|(_, root, version)| okf_core::bundle::settings::load_for_at(root, version))
        .collect::<Result<_>>()?;
    let primary = roots
        .first()
        .ok_or_else(|| OkfError::Usage("query scope contains no available bundles".into()))?;
    let settings = &primary.0;
    let scan_limit = if args.full_scan {
        None
    } else {
        Some(
            args.scan_limit
                .or(settings.settings.query.scan_limit)
                .unwrap_or(1000),
        )
    };
    let mut loaded = Vec::new();
    let mut eligible = 0usize;
    let mut examined = 0usize;
    for (_, root, version) in scope {
        let remaining = scan_limit.map(|n| n.saturating_sub(examined));
        if let Some(revision) = version.strip_prefix("git:") {
            let (bundle, _, total) =
                okf_core::graph::catalog::load_revision_limited(root, revision, remaining)?
                    .ok_or_else(|| {
                        OkfError::Environment(format!("unavailable historical bundle {version}"))
                    })?;
            eligible += total;
            examined += bundle.concepts.len();
            loaded.push(bundle);
        } else {
            let paths = okf_core::bundle::walk::walk_markdown(root)?
                .into_iter()
                .filter(|p| {
                    !matches!(
                        p.file_name().and_then(|s| s.to_str()),
                        Some("index.md" | "log.md")
                    )
                })
                .collect::<Vec<_>>();
            eligible += paths.len();
            let n = remaining.unwrap_or(paths.len()).min(paths.len());
            let mut concepts = Vec::new();
            let mut failures = Vec::new();
            for path in paths.into_iter().take(n) {
                let id = okf_core::model::concept::ConceptId::from_relative(
                    &path.with_extension("").to_string_lossy(),
                );
                let file = root.join(&path);
                let bytes = std::fs::read_to_string(&file)
                    .map_err(|error| OkfError::from(error).at_path(&file))?;
                match okf_core::parse::parse_concept_file(id, &bytes, &file) {
                    Ok(concept) => concepts.push(concept),
                    Err(OkfError::Yaml(message)) => failures.push(message),
                    Err(error) => return Err(error),
                }
            }
            if !failures.is_empty() {
                return Err(OkfError::yaml_files(failures));
            }
            examined += concepts.len();
            loaded.push(okf_core::bundle::loader::Bundle {
                root: root.clone(),
                concepts,
            });
        }
    }
    let complete = examined == eligible;
    let scans = &loaded;
    let mut hits = Vec::new();
    for (i, bundle) in scans.iter().enumerate() {
        for hit in search_detailed(bundle, &filter) {
            if !literal
                .iter()
                .all(|(k, v)| field_matches(hit.concept, k, v))
            {
                continue;
            }
            let mut accepted = true;
            for condition in &conditions {
                if !condition.matches(hit.concept)? {
                    accepted = false;
                    break;
                }
            }
            if accepted {
                hits.push((i, hit));
            }
        }
    }
    hits.sort_by(|(li, left), (ri, right)| {
        let identity = || {
            scope[*li]
                .0
                .cmp(&scope[*ri].0)
                .then_with(|| left.concept.id.0.cmp(&right.concept.id.0))
                .then_with(|| scope[*li].2.cmp(&scope[*ri].2))
        };
        if filter.sort == SearchSort::Relevance {
            right.score.cmp(&left.score).then_with(identity)
        } else {
            identity()
        }
    });
    let matched = hits.len();
    let all: Vec<_> = hits.iter().map(|(_, h)| h.concept).collect();
    let offset = args.offset.unwrap_or(0);
    let limit = args.limit.unwrap_or(matched);
    let page_end = offset.saturating_add(limit).min(matched);
    let page = if offset >= matched {
        &hits[0..0]
    } else {
        &hits[offset..page_end]
    };
    let view = args
        .view
        .as_ref()
        .map(|name| {
            settings
                .settings
                .views
                .get(name)
                .ok_or_else(|| OkfError::Usage(format!("unknown view {name:?}")))
        })
        .transpose()?;
    let columns = if !args.column.is_empty() {
        args.column.clone()
    } else {
        view.map(|v| v.columns.clone()).unwrap_or_default()
    };
    let projection = if !args.project.is_empty() {
        args.project.clone()
    } else if json_output && view.is_some_and(|v| v.extended_output) {
        view.map(|v| v.columns.clone()).unwrap_or_default()
    } else {
        Vec::new()
    };
    let mut expands = args.expand.clone();
    if !args.no_expand
        && expands.is_empty()
        && (!json_output || view.is_some_and(|v| v.extended_output))
    {
        expands = view.map(|v| v.expand.clone()).unwrap_or_default();
    }
    let human_fields = if projection.is_empty() {
        &columns
    } else {
        &projection
    };
    let scoped_human = !json_output && scope.len() > 1;
    if !json_output && (!columns.is_empty() || (scoped_human && !projection.is_empty())) {
        let identity = if scoped_human {
            "BUNDLE\tVERSION\tID\t"
        } else {
            ""
        };
        output::print_text_line(format_args!("{identity}{}", human_fields.join("\t")))?;
    }
    let plain = !json_output && projection.is_empty() && columns.is_empty();
    if plain && scoped_human {
        output::print_text_line(format_args!(
            "BUNDLE\tVERSION\tID\tTYPE\tTRUST\tSTATUS\tTITLE"
        ))?;
    }
    if plain && !scoped_human {
        if args.text.is_empty() {
            output::print_concepts(
                &page.iter().map(|(_, h)| h.concept).collect::<Vec<_>>(),
                false,
            )?;
        } else {
            let hits = page
                .iter()
                .map(|(_, h)| okf_core::query::search::SearchHit {
                    concept: h.concept,
                    score: h.score,
                    matches: h.matches.clone(),
                })
                .collect::<Vec<_>>();
            output::print_search_hits(&hits, false)?;
        }
    }
    for (i, hit) in page {
        let (bundle_id, _, version) = &scope[*i];
        if plain {
            if scoped_human {
                output::print_text_line(format_args!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    bundle_id,
                    version,
                    hit.concept.id.0,
                    hit.concept.concept_type().unwrap_or("-"),
                    hit.concept.trust_tier().as_str(),
                    hit.concept.effective_status(),
                    hit.concept.title().unwrap_or("")
                ))?;
            }
            continue;
        }
        if scoped_human {
            output::print_text(format_args!(
                "{}\t{}\t{}\t",
                bundle_id, version, hit.concept.id.0
            ))?;
        }
        if !projection.is_empty() {
            let record = project(hit.concept, &projection, Some(bundle_id), version)?;
            if json_output {
                output::print_line(&record)?;
            } else {
                print_cells(hit.concept, &projection, Some(bundle_id), version)?;
            }
        } else if !json_output && !columns.is_empty() {
            print_cells(hit.concept, &columns, Some(bundle_id), version)?;
        } else {
            if json_output && scope.len() > 1 {
                output::print_line(
                    &json!({"kind":"concept-identity","schema_version":1,"id":hit.concept.id.0,"bundle":bundle_id,"version":version}),
                )?;
            }
            if args.text.is_empty() {
                output::print_concepts(&[hit.concept], json_output)?;
            } else {
                output::print_search_hits(std::slice::from_ref(hit), json_output)?;
            }
        }
    }
    if !expands.is_empty() {
        emit_expansion(args, json_output, &expands, page, &loaded, scope, &roots)?;
    }
    if args.facets || !args.facet.is_empty() {
        let fields = if args.facet.is_empty() {
            settings.settings.facets.fields.clone()
        } else {
            args.facet.clone()
        };
        for field in fields {
            let record = facet(
                &all,
                &field,
                settings.settings.facets.max_distinct_values,
                settings
                    .settings
                    .facets
                    .include_high_cardinality
                    .contains(&field),
                complete,
            )?;
            if record.get("truncated").and_then(serde_json::Value::as_bool) == Some(true) {
                query_warning(
                    json_output,
                    "facet-output-limit",
                    Some(1000),
                    record
                        .get("omitted_values")
                        .and_then(serde_json::Value::as_u64)
                        .map(|n| n as usize),
                )?;
            }
            output::print_line(&record)?;
        }
    }
    let has_more = complete && page_end < matched && offset < matched;
    let extended = args.offset.is_some()
        || args.facets
        || !args.facet.is_empty()
        || !args.facet_filter.is_empty()
        || !projection.is_empty()
        || !expands.is_empty()
        || args.scan_limit.is_some()
        || args.full_scan;
    if json_output && (extended || !complete) {
        output::print_line(
            &json!({"kind":"query-summary","schema_version":1,"total_matches":if complete{Some(matched)}else{None},"observed_matches":matched,"returned":page.len(),"offset":offset,"limit":args.limit,"has_more":has_more,"next_offset":if has_more{Some(offset+page.len())}else{None},"scan_complete":complete,"partial":!complete,"examined_documents":examined,"scan_limit":scan_limit,"filters":args.facet_filter,"scope":scope.iter().map(|(id,root,version)|json!({"bundle":id,"root":root,"version":version,"mutable":version=="working-tree"})).collect::<Vec<_>>(),"interpretation":roots.iter().map(|(s,_)|s).collect::<Vec<_>>() }),
        )?;
    }
    if !complete {
        query_warning(json_output, "scan-limit", scan_limit, None)?;
    }
    if page.len() < matched {
        query_warning(
            json_output,
            "result-limit",
            args.limit,
            Some(matched - page.len()),
        )?;
    }
    Ok(0)
}
fn query_warning(
    json_output: bool,
    reason: &str,
    limit: Option<usize>,
    omitted: Option<usize>,
) -> Result<()> {
    if json_output {
        output::print_line(
            &json!({"kind":"warning","schema_version":1,"reason":reason,"limit":limit,"omitted":omitted,"message":format!("query omitted requested results or context: {reason}")}),
        )
    } else {
        output::print_text_line(format_args!(
            "warning: {reason}; limit={limit:?}, omitted={omitted:?}"
        ))
    }
}
fn print_cells(
    concept: &Concept,
    fields: &[String],
    bundle: Option<&str>,
    version: &str,
) -> Result<()> {
    let record = okf_core::query::metadata::project(concept, fields, bundle, version)?;
    let cells = record["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            if c["present"] == false {
                return "—".into();
            }
            let values = if let Some(v) = c.get("value") {
                v.clone()
            } else {
                let values: Vec<_> = c["occurrences"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|o| o["value"].clone())
                    .collect();
                if values.len() == 1 {
                    values[0].clone()
                } else {
                    json!(values)
                }
            };
            let text = if let Some(s) = values.as_str() {
                s.to_owned()
            } else {
                values.to_string()
            };
            let text = text.replace('\n', "\\n").replace('\t', "\\t");
            if text.chars().count() > 160 {
                format!("{}…", text.chars().take(159).collect::<String>())
            } else {
                text
            }
        })
        .collect::<Vec<String>>();
    output::print_text_line(format_args!("{}", cells.join("\t")))
}

/// `okf show <concept> [bundle] [--outline | --lines START:END]`.
pub fn run_show(args: &ShowArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    match load_concept(&root, &args.concept)? {
        Some(concept) => {
            if !args.project.is_empty() {
                let (settings, _) = okf_core::bundle::settings::load_for(&root)?;
                let record = okf_core::query::metadata::project(
                    &concept,
                    &args.project,
                    settings.bundle.as_deref(),
                    "working-tree",
                )?;
                if json {
                    output::print_line(&record)?;
                } else {
                    print_cells(
                        &concept,
                        &args.project,
                        settings.bundle.as_deref(),
                        "working-tree",
                    )?;
                }
            } else if args.body {
                if json {
                    output::print_line(&json!({
                        "kind": "body", "id": concept.id.0, "body": concept.body,
                    }))?;
                } else {
                    output::print_text(format_args!("{}", concept.body))?;
                }
            } else if args.outline {
                output::print_concept_outline(&concept, json)?;
            } else if let Some(raw) = &args.lines {
                let (start, end) = parse_line_range(raw)?;
                output::print_concept_lines(&concept, start, end, json)?;
            } else if args.numbered {
                output::print_concept_lines(&concept, 1, usize::MAX, json)?;
            } else {
                output::print_concept(&concept, json)?;
            }
            Ok(0)
        }
        None => Err(OkfError::Usage(format!(
            "concept not found: {}",
            args.concept
        ))),
    }
}

/// `okf browse [bundle] [--directory <dir>]` — read or synthesize a structural index.
pub fn run_browse(args: &BrowseArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let result = browse(&root, Some(&args.directory))?;
    if json {
        output::print_line(&json!({
            "kind": "index",
            "directory": result.directory,
            "path": result.path.to_string_lossy(),
            "source": result.source.as_str(),
            "content": result.content,
        }))?;
    } else {
        output::print_text(format_args!("{}", result.content))?;
    }
    Ok(0)
}

/// Parse an inclusive, 1-based `START:END` range (or a single line `N`).
fn parse_line_range(raw: &str) -> Result<(usize, usize)> {
    let parse = |part: &str| {
        part.parse::<usize>()
            .ok()
            .filter(|n| *n > 0)
            .ok_or_else(|| {
                OkfError::Usage(format!(
                    "invalid --lines {raw:?}: expected START:END with positive line numbers"
                ))
            })
    };
    let (start, end) = match raw.split_once(':') {
        Some((start, end)) => (parse(start)?, parse(end)?),
        None => {
            let line = parse(raw)?;
            (line, line)
        }
    };
    if start > end {
        return Err(OkfError::Usage(format!(
            "invalid --lines {raw:?}: start must not exceed end"
        )));
    }
    Ok((start, end))
}

/// `okf backlinks <concept> [bundle]` — concepts that link to the given concept.
pub fn run_backlinks(args: &IdArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let (_, ontology) = okf_core::bundle::settings::load_for(&root)?;
    if args.details {
        if let Some(ontology) = &ontology {
            let target = okf_core::model::concept::ConceptId::parse(&args.concept)?;
            let mut matched = std::collections::BTreeSet::new();
            for concept in &bundle.concepts {
                for mut edge in okf_core::graph::relationships::relationships(concept, ontology)
                    .into_iter()
                    .filter(|e| e.target == target.0)
                {
                    edge.status = if bundle.get(&edge.target).is_some() {
                        "resolved"
                    } else {
                        "missing"
                    }
                    .into();
                    matched.insert(concept.id.0.clone());
                    if json {
                        output::print_line(
                            &json!({"kind":"relationship","schema_version":1,"direction":"incoming","bundle":okf_core::bundle::settings::load_for(&root)?.0.bundle,"version":"working-tree","edge":edge}),
                        )?;
                    } else {
                        output::print_text_line(format_args!(
                            "{} —{}→ {} (authored {})",
                            edge.target,
                            edge.inverse.as_deref().unwrap_or("incoming"),
                            edge.source,
                            edge.authored_kind.as_deref().unwrap_or("unknown")
                        ))?;
                        output::print_text_line(format_args!(
                            "  source field: {}",
                            edge.field_path
                        ))?;
                        for (key, value) in edge.attributes {
                            output::print_text_line(format_args!(
                                "  {}: {}",
                                key,
                                serde_json::to_string(&value).unwrap_or_default()
                            ))?;
                        }
                    }
                }
            }
            for id in backlinks_of(&bundle, Some(ontology), &args.concept)
                .into_iter()
                .filter(|id| !matched.contains(&id.0))
            {
                if json {
                    output::print_line(
                        &json!({"kind":"relationship","schema_version":1,"direction":"incoming","source":id.0,"target":target.0,"authored_kind":null,"inverse":null,"status":"resolved"}),
                    )?;
                } else {
                    output::print_text_line(format_args!("{} ← {}", target.0, id.0))?;
                }
            }
            return Ok(0);
        }
    }
    let ids = backlinks_of(&bundle, ontology.as_ref(), &args.concept);
    let concepts: Vec<&Concept> = ids.iter().filter_map(|id| bundle.get(&id.0)).collect();
    output::print_concepts(&concepts, json)?;
    Ok(0)
}

/// `okf links <concept> [bundle]` — direct normalized outbound concept links.
pub fn run_links(args: &IdArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let (_, ontology) = okf_core::bundle::settings::load_for(&root)?;
    let concept = load_concept(&root, &args.concept)?
        .ok_or_else(|| OkfError::Usage(format!("concept not found: {}", args.concept)))?;
    let links = outbound_links(&concept, ontology.as_ref());

    if json {
        for target in &links {
            output::print_line(&json!({
                "kind": "link",
                "source": concept.id.0,
                "target": target.0,
                "exists": concept_exists(&root, target),
            }))?;
        }
    } else if links.is_empty() {
        output::print_text_line(format_args!("(no links)"))?;
    } else {
        output::print_text_line(format_args!("TARGET\tSTATUS"))?;
        for target in &links {
            output::print_text_line(format_args!(
                "{}\t{}",
                target.0,
                if concept_exists(&root, target) {
                    "exists"
                } else {
                    "missing"
                }
            ))?;
        }
    }
    Ok(0)
}

/// `okf graph [bundle] [--root concept] [--direction ...] [--depth N] --format`.
pub fn run_graph(args: &GraphArgs, json_output: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let bundle = load_bundle(&root)?;
    let (_, ontology) = okf_core::bundle::settings::load_for(&root)?;
    let format = RenderFormat::parse(&args.format).ok_or_else(|| {
        OkfError::Usage(format!(
            "unknown graph format {:?}: expected mermaid, dot, or graphml",
            args.format
        ))
    })?;
    let direction = args
        .direction
        .as_deref()
        .and_then(GraphDirection::parse)
        .unwrap_or(GraphDirection::Outgoing);
    let graph = build_graph(&bundle, ontology.as_ref());
    let out = render_neighborhood(
        &graph,
        format,
        args.concept.as_deref(),
        direction,
        args.depth,
    );
    if json_output {
        output::print_line(&json!({
            "kind": "graph",
            "format": format.as_str(),
            "root": args.concept,
            "direction": direction.as_str(),
            "depth": args.depth,
            "content": out,
        }))?;
    } else {
        output::print_text(format_args!("{out}"))?;
    }
    Ok(0)
}

/// `okf resolve <link> [bundle] [--from <ctx>]`.
pub fn run_resolve(args: &ResolveArgs, json: bool) -> Result<i32> {
    let root = resolve_bundle(args.bundle.as_deref())?;
    let r = resolve_at(&root, args.from.as_deref(), &args.link);
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
    Ok(if r.exists { 0 } else { 1 })
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
    match val {
        serde_yaml::Value::Sequence(items) => items.iter().any(|item| yaml_scalar_eq(item, value)),
        other => yaml_scalar_eq(other, value),
    }
}

fn yaml_scalar_eq(v: &serde_yaml::Value, want: &str) -> bool {
    match v {
        serde_yaml::Value::String(s) => s == want,
        serde_yaml::Value::Bool(b) => (*b && want == "true") || (!*b && want == "false"),
        serde_yaml::Value::Number(n) => n.to_string() == want,
        serde_yaml::Value::Tagged(tagged) => yaml_scalar_eq(&tagged.value, want),
        _ => false,
    }
}

fn emit_expansion(
    args: &SearchArgs,
    json_output: bool,
    rules: &[String],
    page: &[(usize, okf_core::query::search::SearchHit<'_>)],
    bundles: &[okf_core::bundle::loader::Bundle],
    scope: &[(String, std::path::PathBuf, String)],
    settings: &[(
        okf_core::bundle::settings::EffectiveSettings,
        Option<okf_core::ontology::schema::Ontology>,
    )],
) -> Result<()> {
    use std::collections::BTreeSet;
    let context = okf_core::bundle::context::current_context(args.bundle.as_deref(), None)?;
    let fields = if args.target_field.is_empty() {
        vec!["title".into(), "type".into(), "description".into()]
    } else {
        args.target_field.clone()
    };
    let mut total_edges = 0usize;
    let mut targets = BTreeSet::new();
    let mut fetched_targets: std::collections::BTreeMap<String, Option<Concept>> =
        std::collections::BTreeMap::new();
    let mut bytes = 0usize;
    let mut emitted_edges = 0usize;
    let mut emitted_targets = 0usize;
    let mut reasons = BTreeSet::new();
    for (i, hit) in page {
        if reasons.contains("byte-limit") {
            break;
        }
        let Some(ontology) = &settings[*i].1 else {
            continue;
        };
        let mut selected = BTreeSet::new();
        for raw in rules {
            let (declaring, name) = raw
                .split_once(':')
                .map(|(b, r)| (Some(b), r))
                .unwrap_or((None, raw.as_str()));
            if declaring.is_none() && *i != 0 {
                continue;
            }
            if declaring.is_some_and(|b| b != scope[*i].0) {
                continue;
            }
            if !ontology
                .concepts
                .values()
                .any(|c| c.relationships.contains_key(name))
            {
                return Err(OkfError::Usage(format!(
                    "unknown relationship rule {raw:?}"
                )));
            }
            selected.insert(name.to_string());
        }
        let edges = okf_core::graph::relationships::relationships(hit.concept, ontology)
            .into_iter()
            .filter(|e| selected.contains(&e.rule))
            .collect::<Vec<_>>();
        total_edges += edges.len();
        if edges.len() > args.expansion_edges {
            reasons.insert("edge-limit");
        }
        for mut edge in edges.into_iter().take(args.expansion_edges) {
            let raw = &edge.raw_reference;
            let source_root = std::fs::canonicalize(&bundles[*i].root)
                .unwrap_or_else(|_| bundles[*i].root.clone());
            let path =
                okf_core::graph::catalog::ordinary_path(&source_root, &hit.concept.id.0, raw);
            let target_entry = context.catalog.as_ref().and_then(|catalog| {
                path.as_ref().and_then(|path| {
                    okf_core::graph::catalog::entry_for_path(&catalog.entries, path)
                })
            });
            let target_root = target_entry.map(|e| e.root.clone()).unwrap_or(source_root);
            let rel = path
                .as_ref()
                .and_then(|path| path.strip_prefix(&target_root).ok());
            let target_index = rel.and_then(|_| {
                scope.iter().position(|(_, root, _)| {
                    std::fs::canonicalize(root).unwrap_or(root.clone())
                        == std::fs::canonicalize(&target_root).unwrap_or(target_root.clone())
                })
            });
            let id = rel
                .filter(|p| {
                    p.extension().and_then(|s| s.to_str()) == Some("md")
                        && !matches!(
                            p.file_name().and_then(|s| s.to_str()),
                            Some("index.md" | "log.md")
                        )
                })
                .map(|p| {
                    okf_core::model::concept::ConceptId::from_relative(
                        p.to_string_lossy()
                            .strip_suffix(".md")
                            .unwrap_or(&p.to_string_lossy()),
                    )
                    .0
                });
            let target_key = target_index.and_then(|n| {
                id.as_ref()
                    .map(|id| format!("{}:{}@{}", scope[n].0, id, scope[n].2))
            });
            let mut fetch_limited = false;
            if let (Some(n), Some(id), Some(key)) = (target_index, id.as_ref(), target_key.as_ref())
            {
                if bundles[n].get(id).is_none() && !fetched_targets.contains_key(key) {
                    if fetched_targets.len() >= args.expansion_targets
                        || emitted_targets >= args.expansion_targets
                    {
                        reasons.insert("target-limit");
                        fetch_limited = true;
                    } else {
                        let concept = if let Some(revision) = scope[n].2.strip_prefix("git:") {
                            okf_core::graph::catalog::read_concept_at(
                                &bundles[n].root,
                                id,
                                revision,
                            )?
                        } else {
                            load_concept(&bundles[n].root, id)?
                        };
                        fetched_targets.insert(key.clone(), concept);
                    }
                }
            }
            let target = target_index.and_then(|n| {
                id.as_ref()
                    .and_then(|id| {
                        bundles[n].get(id).or_else(|| {
                            target_key
                                .as_ref()
                                .and_then(|key| fetched_targets.get(key))
                                .and_then(Option::as_ref)
                        })
                    })
                    .map(|c| (n, c))
            });
            if edge.status != "unknown-kind" && edge.status != "external" {
                edge.status = if target.is_some() {
                    "resolved"
                } else if target_entry.is_some_and(|e| !e.available) {
                    "unavailable"
                } else if target_index.is_none() {
                    "out-of-scope"
                } else if fetch_limited
                    && target_index.zip(id.as_ref()).is_some_and(|(n, id)| {
                        okf_core::bundle::loader::concept_exists(
                            &bundles[n].root,
                            &okf_core::model::concept::ConceptId(id.clone()),
                        )
                    })
                {
                    "resolved"
                } else {
                    "missing"
                }
                .into();
            }
            if let Some((n, c)) = target {
                if let Some(ct) = ontology
                    .concepts
                    .get(hit.concept.concept_type().unwrap_or(""))
                {
                    if let Some(mapped) = ct.relationships.get(&edge.rule) {
                        if let Some(reference) = ct.references.get(&mapped.reference) {
                            let restriction = edge
                                .authored_kind
                                .as_ref()
                                .and_then(|kind| mapped.kinds.get(kind))
                                .and_then(|k| k.target.as_ref())
                                .unwrap_or(&reference.target);
                            let allowed = restriction.types().iter().any(|expected| {
                                let (expected_bundle, expected_type) =
                                    expected.split_once(':').unwrap_or((&scope[*i].0, expected));
                                expected_bundle == scope[n].0
                                    && c.concept_type().is_some_and(|actual| {
                                        settings[n]
                                            .1
                                            .as_ref()
                                            .map(|ont| {
                                                okf_core::ontology::field_types::concept_is_a(
                                                    ont,
                                                    actual,
                                                    expected_type,
                                                )
                                            })
                                            .unwrap_or(actual == expected_type)
                                    })
                            });
                            if !allowed {
                                edge.status = "wrong-type".into();
                            }
                        }
                    }
                }
            }
            let identity = target.map(|(n, c)| format!("{}:{}@{}", scope[n].0, c.id.0, scope[n].2));
            let edge_record = json!({"kind":"relationship","schema_version":1,"primary":hit.concept.id.0,"bundle":scope[*i].0,"version":scope[*i].2,"edge":edge,"target_identity":identity});
            let payload = target
                .map(|(n, c)| {
                    okf_core::query::metadata::project(c, &fields, Some(&scope[n].0), &scope[n].2)
                })
                .transpose()?;
            let mut records = vec![edge_record];
            if fetch_limited {
                records[0]["target_payload_status"] = json!("target-limit");
                records[0]["target_identity"] = json!(target_key);
            }
            if let (Some(identity), Some(mut payload)) = (&identity, payload) {
                if !targets.contains(identity) {
                    if emitted_targets >= args.expansion_targets {
                        reasons.insert("target-limit");
                        records[0]["target_payload_status"] = json!("target-limit");
                    } else {
                        payload["kind"] = json!("related-concept");
                        payload["identity"] = json!(identity);
                        records.push(payload);
                    }
                }
            }
            let size = records
                .iter()
                .map(|r| serde_json::to_vec(r).unwrap_or_default().len() + 1)
                .sum::<usize>();
            if bytes + size > args.expansion_bytes {
                reasons.insert("byte-limit");
                break;
            }
            bytes += size;
            emitted_edges += 1;
            if records.len() > 1 {
                emitted_targets += 1;
                if let Some(identity) = identity {
                    targets.insert(identity);
                }
            }
            for record in records {
                if json_output {
                    output::print_line(&record)?;
                } else if record["kind"] == "relationship" {
                    let edge = &record["edge"];
                    output::print_text_line(format_args!(
                        "{}: {} → {} [{}]",
                        hit.concept.id.0,
                        edge["authored_kind"].as_str().unwrap_or("unknown"),
                        edge["raw_reference"].as_str().unwrap_or(""),
                        edge["status"].as_str().unwrap_or("unknown")
                    ))?;
                    if let Some(attrs) = edge["attributes"].as_object() {
                        for (key, value) in attrs {
                            output::print_text_line(format_args!("  {key}: {value}"))?;
                        }
                    }
                } else {
                    for cell in record["fields"].as_array().unwrap_or(&Vec::new()) {
                        if cell["present"] == true {
                            let values = if let Some(value) = cell.get("value") {
                                value.to_string()
                            } else {
                                cell["occurrences"]
                                    .as_array()
                                    .into_iter()
                                    .flatten()
                                    .map(|o| o["value"].to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            };
                            output::print_text_line(format_args!(
                                "  {}: {}",
                                cell["field"].as_str().unwrap_or(""),
                                values
                            ))?;
                        }
                    }
                }
            }
        }
    }
    let summary = json!({"kind":"expansion-summary","schema_version":1,"emitted_edges":emitted_edges,"emitted_targets":emitted_targets,"emitted_bytes":bytes,"bounds":{"edges_per_hit":args.expansion_edges,"targets":args.expansion_targets,"bytes":args.expansion_bytes},"truncated":!reasons.is_empty(),"reasons":reasons,"total_edges":if reasons.contains("byte-limit"){None}else{Some(total_edges)}});
    if json_output {
        output::print_line(&summary)?;
    } else if !reasons.is_empty() {
        output::print_text_line(format_args!("expansion truncated: {}", summary))?;
    }
    for reason in reasons {
        let bound = match reason {
            "edge-limit" => args.expansion_edges,
            "target-limit" => args.expansion_targets,
            _ => args.expansion_bytes,
        };
        query_warning(json_output, reason, Some(bound), None)?;
    }
    Ok(())
}
