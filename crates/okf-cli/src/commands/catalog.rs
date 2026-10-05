//! Catalog selection and scoped query dispatch; ordinary artifact operations stay confined.
use crate::cli::{ArtifactCmd, Cli, Command, ComputationCmd, OntFieldTypeCmd, OntologyCmd};
use crate::output;
use okf_core::bundle::{
    catalog::load_catalog,
    config::{find_config, load_config},
    context::{current_context, Context},
};
use okf_core::error::{OkfError, Result};
use okf_core::graph::catalog::{
    build_catalog_graph, resolve_reference, resolve_scope, resolve_scope_at, CatalogGraph, Edge,
    Node,
};
use serde_json::json;
use std::path::PathBuf;

pub fn bundle_slot(command: &mut Command) -> Option<&mut Option<String>> {
    Some(match command {
        Command::List(a) => &mut a.bundle,
        Command::Validate(a) => &mut a.bundle,
        Command::Search(a) => &mut a.bundle,
        Command::Show(a) => &mut a.bundle,
        Command::Browse(a) => &mut a.bundle,
        Command::Backlinks(a) | Command::Links(a) => &mut a.bundle,
        Command::Graph(a) => &mut a.bundle,
        Command::Resolve(a) => &mut a.bundle,
        Command::Scan(a) | Command::Stale(a) | Command::Stats(a) => &mut a.bundle,
        Command::Lint(a) => &mut a.bundle,
        Command::Affected(a) => &mut a.bundle,
        Command::Diff(a) => &mut a.bundle,
        Command::Doctor(a) => &mut a.bundle,
        Command::Init(a) => &mut a.bundle,
        Command::Add(a) => &mut a.bundle,
        Command::Edit(a) => &mut a.bundle,
        Command::Mv(a) => &mut a.bundle,
        Command::Rm(a) => &mut a.bundle,
        Command::Verify(a) => &mut a.bundle,
        Command::Refresh(a) => &mut a.bundle,
        Command::Docs(a) => &mut a.bundle,
        Command::Artifact(cmd) => match cmd {
            ArtifactCmd::List(a) => &mut a.bundle,
            ArtifactCmd::Resolve(a) => &mut a.bundle,
            ArtifactCmd::Show(a) => &mut a.bundle,
            ArtifactCmd::Put(a) => &mut a.bundle,
        },
        Command::Computation(ComputationCmd::Check(a)) => &mut a.bundle,
        Command::Ontology(cmd) => match cmd {
            OntologyCmd::List(a) => &mut a.bundle,
            OntologyCmd::Show(a) => &mut a.bundle,
            OntologyCmd::Add(a) | OntologyCmd::Update(a) => &mut a.bundle,
            OntologyCmd::Remove(a) => &mut a.bundle,
            OntologyCmd::Apply(a) => &mut a.bundle,
            OntologyCmd::FieldType(cmd) => match cmd {
                OntFieldTypeCmd::Add(a) | OntFieldTypeCmd::Update(a) => &mut a.bundle,
                OntFieldTypeCmd::Remove(a) => &mut a.bundle,
            },
        },
        _ => return None,
    })
}
fn print_edge(edge: &Edge, json_output: bool) -> Result<()> {
    if json_output {
        let mut value =
            serde_json::to_value(edge).map_err(|e| OkfError::Internal(e.to_string()))?;
        value["kind"] = json!("bundle-edge");
        output::print_line(&value)?;
    } else {
        output::print_text(format_args!("{}", edge_text(edge)))?;
    }
    Ok(())
}
fn edge_text(edge: &Edge) -> String {
    let mut text = format!(
        "{}\t{}\t{}\t{}\t{}\n",
        edge.source.key(),
        edge.resource,
        edge.target
            .as_ref()
            .map(Node::key)
            .unwrap_or_else(|| "?".into()),
        edge.status,
        edge.evidence
    );
    if let Some(snapshot) = &edge.snapshot {
        text.push_str(&format!(
            "  snapshot={} evidence={} requested={} candidate={}\n",
            snapshot.status,
            snapshot.evidence,
            snapshot
                .resolved
                .as_ref()
                .map(Node::key)
                .unwrap_or_else(|| "unresolved".into()),
            snapshot
                .candidate
                .as_ref()
                .map(Node::key)
                .unwrap_or_else(|| "none".into())
        ));
    }
    text
}
fn print_node(node: &Node, kind: &str, json_output: bool) -> Result<()> {
    if json_output {
        output::print_line(
            &json!({"kind":kind,"bundle":node.bundle,"id":node.id,"version":node.version}),
        )?;
    } else {
        output::print_text_line(format_args!("{}", node.key()))?;
    }
    Ok(())
}
fn print_scope(graph: &CatalogGraph, json_output: bool) -> Result<()> {
    if json_output {
        output::print_line(
            &json!({"kind":"scope","requested":graph.scope.requested,"examined":graph.scope.examined,"unavailable":graph.scope.unavailable,"snapshot_examined":graph.scope.snapshot_examined}),
        )?;
    } else {
        output::print_text_line(format_args!("{}", scope_text(graph)))?;
    }
    Ok(())
}
fn scope_text(graph: &CatalogGraph) -> String {
    format!(
        "scope: requested={} examined={} unavailable={}",
        graph.scope.requested.join(","),
        graph
            .scope
            .examined
            .iter()
            .map(|e| format!("{}@{}", e.id, e.version))
            .collect::<Vec<_>>()
            .join(","),
        graph.scope.unavailable.join(",")
    )
}
pub fn run_catalog(json_output: bool) -> Result<i32> {
    let cwd = std::env::current_dir()?;
    let path = find_config(&cwd)
        .ok_or_else(|| OkfError::Usage("catalog requires okf.toml with a catalog path".into()))?;
    let config = load_config(&path)?;
    let base = path.parent().unwrap();
    let catalog_path = config
        .catalog
        .ok_or_else(|| OkfError::Usage("no catalog configured".into()))?;
    let overrides = std::env::var("OKF_CATALOG_OVERRIDES")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| config.catalog_overrides.map(|s| base.join(s)));
    let catalog = load_catalog(&base.join(catalog_path), overrides.as_deref())?;
    for entry in catalog.entries.values() {
        if json_output {
            output::print_line(
                &json!({"kind":"bundle-registration","id":entry.id,"configured_root":entry.configured_root,"root":entry.root,"overridden":entry.overridden,"available":entry.available}),
            )?;
        } else {
            output::print_text_line(format_args!(
                "{}\t{}\t{}\t{}",
                entry.id,
                entry.root.display(),
                if entry.available {
                    "available"
                } else {
                    "unavailable"
                },
                if entry.overridden {
                    "overridden"
                } else {
                    "catalog"
                }
            ))?;
        }
        let context = Context {
            primary: okf_core::bundle::context::SelectedBundle {
                id: Some(entry.id.clone()),
                root: entry.root.clone(),
            },
            catalog: Some(catalog.clone()),
            config: load_config(&path)?,
            config_path: Some(path.clone()),
        };
        let effective = okf_core::bundle::settings::for_context(&context, &entry.root)?;
        if json_output {
            output::print_line(
                &json!({"kind":"effective-settings","bundle":entry.id,"settings":effective}),
            )?;
        } else {
            output::print_text_line(format_args!(
                "  ontology={} digest={}",
                effective
                    .ontology_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "none".into()),
                effective.ontology_digest.as_deref().unwrap_or("none")
            ))?;
        }
    }
    Ok(0)
}
/// Returns Some when a catalog-aware query handled the command; otherwise rewrites only
/// the selected root so existing authoring/query handlers retain their established contracts.
pub fn prepare(cli: &mut Cli) -> Result<Option<i32>> {
    if matches!(cli.command, Command::Catalog) {
        if cli.bundle_id.is_some()
            || !cli.scope_bundle.is_empty()
            || cli.catalog_scope
            || cli.revision.is_some()
        {
            return Err(OkfError::Usage(
                "catalog inspection does not take selectors or scope/revision flags".into(),
            ));
        }
        return run_catalog(cli.json).map(Some);
    }
    let scoped = !cli.scope_bundle.is_empty() || cli.catalog_scope;
    let revision = cli.revision.is_some();
    let supports_scope = matches!(
        cli.command,
        Command::Graph(_)
            | Command::Backlinks(_)
            | Command::Links(_)
            | Command::Resolve(_)
            | Command::Affected(_)
            | Command::Lint(_)
            | Command::Search(_)
            | Command::List(_)
    );
    if (scoped || revision) && !supports_scope {
        return Err(OkfError::Usage("scope and revision flags are supported by graph, backlinks, links, resolve, affected, lint, and search".into()));
    }
    if revision && matches!(cli.command, Command::Lint(_)) {
        return Err(OkfError::Usage("--revision is supported only by graph, backlinks, links, resolve, affected, search, and list".into()));
    }
    let Some(slot) = bundle_slot(&mut cli.command) else {
        if cli.bundle_id.is_some() {
            return Err(OkfError::Usage(
                "this command does not select an OKF bundle".into(),
            ));
        }
        return Ok(None);
    };
    let explicit = slot.clone();
    if cli.bundle_id.is_some() && explicit.is_some() {
        return Err(OkfError::Usage(
            "bundle path and --bundle-id are conflicting selectors".into(),
        ));
    }
    let context = current_context(explicit.as_deref(), cli.bundle_id.as_deref())?;
    if scoped && context.catalog.is_none() {
        if let Some(catalog_path) = &context.config.catalog {
            let base = context
                .config_path
                .as_ref()
                .and_then(|p| p.parent())
                .unwrap_or(std::path::Path::new("."));
            let overrides = std::env::var("OKF_CATALOG_OVERRIDES")
                .ok()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .or_else(|| {
                    context
                        .config
                        .catalog_overrides
                        .as_ref()
                        .map(|s| base.join(s))
                });
            load_catalog(&base.join(catalog_path), overrides.as_deref())?;
        }
        return Err(OkfError::Usage(
            "explicit bundle scope requires a valid catalog".into(),
        ));
    }
    if cli.bundle_id.is_some() || context.catalog.is_some() {
        *slot = Some(context.primary.root.to_string_lossy().into_owned());
    }
    // Catalog-aware graph queries use selected-only scope even without an explicit scope flag.
    if supports_scope && (context.catalog.is_some() || scoped || revision) {
        if let Command::Search(args) | Command::List(args) = &cli.command {
            let scope = resolve_scope_at(
                &context,
                &cli.scope_bundle,
                cli.catalog_scope,
                cli.revision.as_deref(),
            )?;
            let graph = CatalogGraph {
                scope,
                ..Default::default()
            };
            print_scope(&graph, cli.json)?;
            let roots = graph
                .scope
                .examined
                .iter()
                .map(|e| (e.id.clone(), e.root.clone(), e.version.clone()))
                .collect::<Vec<_>>();
            return super::query::run_search_in_scope(args, cli.json, &roots).map(Some);
        }
        let graph = build_catalog_graph(
            &context,
            &cli.scope_bundle,
            cli.catalog_scope,
            cli.revision.as_deref(),
        )?;
        if matches!(cli.command, Command::Graph(_)) && !cli.json {
            use std::io::Write;
            writeln!(std::io::stderr().lock(), "{}", scope_text(&graph))?;
        } else {
            print_scope(&graph, cli.json)?;
        }
        return run_scoped(cli, &context, &graph).map(Some);
    }
    Ok(None)
}
fn run_scoped(cli: &Cli, context: &Context, graph: &CatalogGraph) -> Result<i32> {
    match &cli.command {
        Command::Search(a) | Command::List(a) => {
            let scope = graph
                .scope
                .examined
                .iter()
                .map(|e| (e.id.clone(), e.root.clone(), e.version.clone()))
                .collect::<Vec<_>>();
            super::query::run_search_in_scope(a, cli.json, &scope)
        }
        Command::Links(a) => {
            let target = graph.target(context, &a.concept);
            if !graph.nodes.contains(&target) {
                return Err(OkfError::Usage(format!(
                    "concept not found in examined scope: {}",
                    a.concept
                )));
            }
            for edge in graph.edges.iter().filter(|e| e.source == target) {
                print_edge(edge, cli.json)?;
            }
            Ok(0)
        }
        Command::Backlinks(a) => {
            let target = graph.target(context, &a.concept);
            let nodes: std::collections::BTreeSet<_> = graph
                .edges
                .iter()
                .filter(|e| e.target.as_ref() == Some(&target))
                .map(|e| e.source.clone())
                .collect();
            if a.details {
                for edge in graph
                    .edges
                    .iter()
                    .filter(|e| e.target.as_ref() == Some(&target))
                {
                    print_edge(edge, cli.json)?;
                }
            } else {
                for node in nodes {
                    print_node(&node, "bundle-backlink", cli.json)?;
                }
            }
            Ok(0)
        }
        Command::Resolve(a) => {
            let edge = resolve_reference(
                context,
                graph,
                a.from.as_deref().unwrap_or("/context"),
                &a.link,
            );
            print_edge(&edge, cli.json)?;
            Ok(0)
        }
        Command::Affected(a) => {
            use std::io::{BufRead, IsTerminal};
            let mut raw = a.changed.clone();
            if !std::io::stdin().is_terminal() {
                for line in std::io::stdin().lock().lines() {
                    let line = line?;
                    if !line.trim().is_empty() {
                        raw.push(line.trim().into());
                    }
                }
            }
            if raw.is_empty() {
                return Err(OkfError::Usage(
                    "affected requires --changed targets or stdin".into(),
                ));
            }
            let changed = raw
                .iter()
                .map(|s| graph.target(context, s))
                .collect::<Vec<_>>();
            let nodes = graph.affected(&changed, a.transitive, a.depth);
            for node in &nodes {
                print_node(node, "bundle-affected", cli.json)?;
            }
            Ok(
                if !nodes.is_empty() && a.fail_on.as_deref().is_some_and(|s| s != "never") {
                    1
                } else {
                    0
                },
            )
        }
        Command::Graph(a) => {
            let selected = a.concept.as_ref().map(|s| {
                graph.neighborhood(
                    &graph.target(context, s),
                    a.direction.as_deref().unwrap_or("outgoing"),
                    a.depth,
                )
            });
            let edges = graph.edges.iter().filter(|e| {
                selected.as_ref().is_none_or(|n| {
                    n.contains(&e.source) && e.target.as_ref().is_none_or(|t| n.contains(t))
                })
            });
            if cli.json {
                for node in graph
                    .nodes
                    .iter()
                    .filter(|n| selected.as_ref().is_none_or(|s| s.contains(n)))
                {
                    print_node(node, "bundle-node", true)?;
                }
                for edge in edges {
                    print_edge(edge, true)?;
                }
            } else {
                let mut content = String::new();
                let mut rendered_nodes = graph.nodes.clone();
                for edge in &graph.edges {
                    if let Some(target) = &edge.target {
                        rendered_nodes.insert(target.clone());
                    }
                }
                let nodes: Vec<_> = rendered_nodes
                    .iter()
                    .filter(|n| selected.as_ref().is_none_or(|s| s.contains(n)))
                    .collect();
                let index = nodes
                    .iter()
                    .enumerate()
                    .map(|(i, n)| (n.key(), i))
                    .collect::<std::collections::BTreeMap<_, _>>();
                match a.format.as_str() {
                    "dot" => {
                        content.push_str("digraph okf {\n");
                        for (i, node) in nodes.iter().enumerate() {
                            content.push_str(&format!(
                                "  n{i} [label={}];\n",
                                serde_json::to_string(&node.key()).unwrap()
                            ));
                        }
                        for edge in edges {
                            if let (Some(i), Some(j)) = (
                                index.get(&edge.source.key()),
                                edge.target.as_ref().and_then(|n| index.get(&n.key())),
                            ) {
                                content.push_str(&format!(
                                    "  n{i} -> n{j} [label={}];\n",
                                    serde_json::to_string(&edge.status).unwrap()
                                ));
                            }
                        }
                        content.push_str("}\n");
                    }
                    "graphml" => {
                        content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\"><key id=\"label\" for=\"node\" attr.name=\"label\" attr.type=\"string\"/><graph edgedefault=\"directed\">\n");
                        for (i, node) in nodes.iter().enumerate() {
                            content.push_str(&format!(
                                "<node id=\"n{i}\"><data key=\"label\">{}</data></node>\n",
                                xml(&node.key())
                            ));
                        }
                        for (edge_id, edge) in edges.enumerate() {
                            if let (Some(i), Some(j)) = (
                                index.get(&edge.source.key()),
                                edge.target.as_ref().and_then(|n| index.get(&n.key())),
                            ) {
                                content.push_str(&format!(
                                    "<edge id=\"e{edge_id}\" source=\"n{i}\" target=\"n{j}\"/>\n"
                                ));
                            }
                        }
                        content.push_str("</graph></graphml>\n");
                    }
                    _ => {
                        content.push_str("graph TD\n");
                        for (i, node) in nodes.iter().enumerate() {
                            content.push_str(&format!(
                                "  n{i}[\"{}\"]\n",
                                node.key().replace('&', "&amp;").replace('"', "&quot;")
                            ));
                        }
                        for edge in edges {
                            if let (Some(i), Some(j)) = (
                                index.get(&edge.source.key()),
                                edge.target.as_ref().and_then(|n| index.get(&n.key())),
                            ) {
                                content.push_str(&format!("  n{i} -->|{}| n{j}\n", edge.status));
                            }
                        }
                    }
                }
                output::print_text(format_args!("{content}"))?;
                // Keep unresolved references and snapshot candidates visible in human output.
                for edge in graph.edges.iter().filter(|e| {
                    (e.status != "resolved" || e.snapshot.is_some())
                        && selected
                            .as_ref()
                            .is_none_or(|nodes| nodes.contains(&e.source))
                }) {
                    use std::io::Write;
                    write!(std::io::stderr().lock(), "{}", edge_text(edge))?;
                }
            }
            Ok(0)
        }
        Command::Lint(a) => run_lint_scoped(a, cli.json, graph),
        _ => Err(OkfError::Internal("unsupported scoped dispatch".into())),
    }
}
fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn run_lint_scoped(
    args: &crate::cli::LintArgs,
    json_output: bool,
    graph: &CatalogGraph,
) -> Result<i32> {
    use okf_core::check::lint::{lint_bundle, meets_threshold, FailOn, Severity};
    use std::str::FromStr;
    let fail_on = FailOn::from_str(args.fail_on.as_deref().unwrap_or("error"))?;
    let mut failed = false;
    let mut target_types = std::collections::BTreeMap::new();
    let mut target_ontologies = std::collections::BTreeMap::new();
    for examined in &graph.scope.examined {
        if let Some(ontology) = okf_core::bundle::settings::load_for(&examined.root)?.1 {
            target_ontologies.insert(examined.id.clone(), ontology);
        }
        for concept in okf_core::bundle::loader::load_bundle(&examined.root)?.concepts {
            if let Some(type_name) = concept.concept_type() {
                target_types.insert(
                    Node {
                        bundle: examined.id.clone(),
                        id: concept.id.0.clone(),
                        version: examined.version.clone(),
                    },
                    format!("{}:{type_name}", examined.id),
                );
            }
        }
    }
    for examined in &graph.scope.examined {
        let bundle = okf_core::bundle::loader::load_bundle(&examined.root)?;
        let (effective, ontology) = okf_core::bundle::settings::load_for(&examined.root)?;
        let config = super::check::lint_config(&effective.settings)?;
        let mut findings = lint_bundle(&bundle, None, &config);
        if let Some(ontology) = &ontology {
            let mut qualified = ontology.clone();
            for ct in qualified.concepts.values_mut() {
                for rule in ct.references.values_mut() {
                    rule.target = okf_core::ontology::schema::Target::Many(
                        rule.target
                            .types()
                            .iter()
                            .map(|target| {
                                if target.contains(':') {
                                    target.to_string()
                                } else {
                                    format!("{}:{target}", examined.id)
                                }
                            })
                            .collect(),
                    );
                }
            }
            for concept in &bundle.concepts {
                let resolver = |resource: &str| {
                    graph
                        .edges
                        .iter()
                        .find(|edge| {
                            edge.source.bundle == examined.id
                                && edge.source.id == concept.id.0
                                && edge.resource == resource
                                && edge.status == "resolved"
                        })
                        .and_then(|edge| edge.target.as_ref())
                        .and_then(|node| target_types.get(node))
                        .cloned()
                };
                for violation in okf_core::ontology::field_types::check_concept_with_budget(
                    &qualified,
                    &concept.frontmatter,
                    resolver,
                    config.finding_budget,
                ) {
                    use okf_core::ontology::field_types::ViolationKind;
                    let (code, field_path) = match &violation.kind {
                        ViolationKind::Metadata { code, path } => {
                            (Some(code.clone()), Some(path.clone()))
                        }
                        ViolationKind::WrongReferenceTarget(rule) => {
                            let edge = graph.edges.iter().find(|edge| {
                                edge.source.bundle == examined.id
                                    && edge.source.id == concept.id.0
                                    && edge.reference_rule.as_ref() == Some(rule)
                                    && violation.message.contains(&format!("{:?}", edge.resource))
                            });
                            (
                                Some("metadata-reference-target-type".into()),
                                edge.map(|e| e.location.clone()),
                            )
                        }
                        _ => (None, None),
                    };
                    if !matches!(violation.kind, ViolationKind::WrongReferenceTarget(_))
                        && !matches!(
                            code.as_deref(),
                            Some("metadata-reference-missing" | "metadata-reference-target")
                        )
                    {
                        findings.push(okf_core::check::lint::Finding {
                            rule: "ontology-violation".into(),
                            code,
                            field_path,
                            severity: config.ontology_violation,
                            concept: Some(concept.id.0.clone()),
                            message: violation.message,
                        });
                    }
                }
                if let Some(ct) = concept
                    .concept_type()
                    .and_then(|name| ontology.concepts.get(name))
                {
                    for edge in graph.edges.iter().filter(|edge| {
                        edge.source.bundle == examined.id
                            && edge.source.id == concept.id.0
                            && edge.reference_rule.is_some()
                            && edge.status == "resolved"
                    }) {
                        let Some(target) = &edge.target else {
                            continue;
                        };
                        let Some(actual) = target_types.get(target).and_then(|qualified| {
                            qualified.strip_prefix(&format!("{}:", target.bundle))
                        }) else {
                            continue;
                        };
                        let Some(reference) = edge
                            .reference_rule
                            .as_ref()
                            .and_then(|key| ct.references.get(key))
                        else {
                            continue;
                        };
                        let allows = |expected: &okf_core::ontology::schema::Target| {
                            expected.types().iter().any(|expected| {
                                let (namespace, name) =
                                    expected.split_once(':').unwrap_or((&examined.id, expected));
                                namespace == target.bundle
                                    && (actual == name
                                        || target_ontologies.get(&target.bundle).is_some_and(
                                            |ontology| {
                                                okf_core::ontology::field_types::concept_is_a(
                                                    ontology, actual, name,
                                                )
                                            },
                                        ))
                            })
                        };
                        let kind_target = edge.relationship.as_ref().and_then(|relationship| {
                            ct.relationships
                                .get(&relationship.rule)
                                .and_then(|rule| {
                                    relationship
                                        .authored_kind
                                        .as_ref()
                                        .and_then(|kind| rule.kinds.get(kind))
                                })
                                .and_then(|kind| kind.target.as_ref())
                        });
                        if !allows(&reference.target)
                            || kind_target.is_some_and(|expected| !allows(expected))
                        {
                            findings.push(okf_core::check::lint::Finding{rule:"ontology-violation".into(),code:Some("metadata-reference-target-type".into()),field_path:Some(edge.location.clone()),severity:config.ontology_violation,concept:Some(concept.id.0.clone()),message:format!("{}: reference {:?} targets {}:{actual}, outside its declared target namespace/types",edge.location,edge.resource,target.bundle)});
                        }
                    }
                }
                let selected = findings
                    .iter()
                    .enumerate()
                    .filter(|(_, finding)| {
                        finding.rule == "ontology-violation"
                            && finding.concept.as_ref() == Some(&concept.id.0)
                    })
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>();
                if selected.len() > config.finding_budget {
                    let remove = selected
                        .into_iter()
                        .skip(config.finding_budget)
                        .collect::<std::collections::BTreeSet<_>>();
                    let mut index = 0usize;
                    findings.retain(|_| {
                        let keep = !remove.contains(&index);
                        index += 1;
                        keep
                    });
                    if !findings.iter().any(|finding| {
                        finding.concept.as_ref() == Some(&concept.id.0)
                            && finding.code.as_deref() == Some("incomplete-check")
                    }) {
                        findings.push(okf_core::check::lint::Finding {
                            rule: "ontology-violation".into(),
                            code: Some("incomplete-check".into()),
                            field_path: None,
                            severity: config.ontology_violation,
                            concept: Some(concept.id.0.clone()),
                            message: "ontology finding budget exhausted; check incomplete".into(),
                        });
                    }
                }
            }
        }
        // The qualified resolver owns all broken-reference diagnostics; the single-root
        // graph would normalize above-root references into unrelated local IDs.
        findings.retain(|f| {
            f.rule != "broken-link" && f.code.as_deref() != Some("metadata-reference-missing")
        });
        failed |= meets_threshold(&findings, fail_on);
        for finding in findings {
            if json_output {
                output::print_line(
                    &json!({"kind":"finding","rule":finding.rule,"code":finding.code,"field_path":finding.field_path,"severity":finding.severity.as_str(),"concept":finding.concept,"bundle":examined.id,"message":finding.message}),
                )?;
            } else {
                output::print_text_line(format_args!(
                    "{}\t{}\t{}:{}\t{}",
                    finding.severity.as_str(),
                    finding.rule,
                    examined.id,
                    finding.concept.as_deref().unwrap_or("-"),
                    finding.message
                ))?;
            }
        }
        let index_severity = super::check::lint_severity(
            effective.settings.lint.index_coverage.as_deref(),
            Severity::Warn,
        )?;
        for finding in okf_core::check::lint::rules::index_coverage::check_indexes(
            &bundle,
            &effective.settings.lint.index_exclude,
        )? {
            failed |= fail_on
                .min_severity()
                .is_some_and(|threshold| index_severity >= threshold);
            if json_output {
                let mut value = serde_json::to_value(&finding)
                    .map_err(|e| OkfError::Internal(e.to_string()))?;
                value["kind"] = json!("finding");
                value["rule"] = json!("index-coverage");
                value["severity"] = json!(index_severity.as_str());
                value["bundle"] = json!(examined.id);
                output::print_line(&value)?;
            } else {
                output::print_text_line(format_args!(
                    "{}\tindex-coverage\t{}:{}\t{}",
                    index_severity.as_str(),
                    examined.id,
                    finding.directory,
                    finding.message
                ))?;
            }
        }
    }
    let threshold = args.fail_on.as_deref().unwrap_or("error");
    for edge in &graph.edges {
        let issue = if !matches!(edge.status.as_str(), "resolved" | "out-of-scope") {
            Some(edge.status.as_str())
        } else if edge
            .snapshot
            .as_ref()
            .is_some_and(|s| s.status != "matched")
        {
            Some("snapshot-expectation")
        } else if edge.fingerprint_status.as_deref() == Some("changed") {
            Some("changed-source-fingerprint")
        } else {
            None
        };
        if let Some(issue) = issue {
            let severity = if edge.status == "missing-target"
                && edge
                    .target
                    .as_ref()
                    .is_some_and(|target| target.bundle == edge.source.bundle)
            {
                "error"
            } else {
                "warn"
            };
            let rule = if severity == "error" && edge.reference_rule.is_none() {
                "broken-link"
            } else {
                "cross-bundle-reference"
            };
            if json_output {
                output::print_line(
                    &json!({"kind":"finding","rule":rule,"code":issue,"severity":severity,"concept":edge.source.id,"bundle":edge.source.bundle,"field_path":edge.location,"message":edge.evidence,"resolution":edge}),
                )?;
            } else {
                output::print_text_line(format_args!(
                    "{severity}\t{issue}\t{}\t{}",
                    edge.source.key(),
                    edge.resource
                ))?;
            }
            failed |= matches!(threshold, "info" | "warn" | "any")
                || (threshold == "error" && severity == "error");
        }
    }
    for unavailable in &graph.scope.unavailable {
        if json_output {
            output::print_line(
                &json!({"kind":"finding","rule":"unavailable-bundle","severity":"warn","bundle":unavailable,"message":"requested scope member unavailable"}),
            )?;
        } else {
            output::print_text_line(format_args!("warn\tunavailable-bundle\t{unavailable}"))?;
        }
        failed |= matches!(threshold, "info" | "warn" | "any");
    }
    Ok(if failed { 1 } else { 0 })
}
