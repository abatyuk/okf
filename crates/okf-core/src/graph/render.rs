//! Graph emitters: Mermaid, GraphML, DOT.
//!
//! All three walk the same deterministically-collected node/edge sets (sorted via
//! `BTreeSet`), so output is byte-stable across runs. A `root` limits output to the subgraph
//! forward-reachable from that concept; otherwise the whole graph is emitted. Broken-link
//! targets appear as nodes so breakage is visible in the rendered artifact.
use crate::graph::build::LinkGraph;
use crate::model::concept::ConceptId;
use std::collections::{BTreeSet, HashMap, HashSet};

/// Output format for [`render`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderFormat {
    Mermaid,
    Graphml,
    Dot,
}

impl RenderFormat {
    /// Parse a format name (`mermaid` / `graphml` / `dot`), case-insensitive.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "mermaid" => Some(RenderFormat::Mermaid),
            "graphml" => Some(RenderFormat::Graphml),
            "dot" => Some(RenderFormat::Dot),
            _ => None,
        }
    }
}

/// Render the graph (or the subgraph forward-reachable from `root`) in `format`.
pub fn render(graph: &LinkGraph, format: RenderFormat, root: Option<&str>) -> String {
    let (nodes, edges) = collect(graph, root);
    match format {
        RenderFormat::Mermaid => mermaid(&nodes, &edges),
        RenderFormat::Graphml => graphml(&nodes, &edges),
        RenderFormat::Dot => dot(&nodes, &edges),
    }
}

/// Deterministic (sorted) node and edge sets. With `root`, restrict to nodes/edges reachable
/// forward from it; otherwise take the whole graph.
fn collect(graph: &LinkGraph, root: Option<&str>) -> (Vec<String>, Vec<(String, String)>) {
    let mut nodes: BTreeSet<String> = BTreeSet::new();
    let mut edges: BTreeSet<(String, String)> = BTreeSet::new();

    match root {
        Some(r) => {
            let start = ConceptId::from_relative(r).0;
            let mut visited: HashSet<String> = HashSet::new();
            let mut stack = vec![start];
            while let Some(node) = stack.pop() {
                if !visited.insert(node.clone()) {
                    continue;
                }
                nodes.insert(node.clone());
                for target in graph.outbound(&node) {
                    edges.insert((node.clone(), target.0.clone()));
                    nodes.insert(target.0.clone());
                    stack.push(target.0.clone());
                }
            }
        }
        None => {
            for (src, outs) in &graph.forward {
                nodes.insert(src.clone());
                for target in outs {
                    nodes.insert(target.0.clone());
                    edges.insert((src.clone(), target.0.clone()));
                }
            }
        }
    }

    (nodes.into_iter().collect(), edges.into_iter().collect())
}

/// Stable `id → n{index}` map over the sorted node list.
fn index_map(nodes: &[String]) -> HashMap<&str, String> {
    nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), format!("n{i}")))
        .collect()
}

fn mermaid(nodes: &[String], edges: &[(String, String)]) -> String {
    let idx = index_map(nodes);
    let mut out = String::from("graph LR\n");
    for node in nodes {
        let label = node.replace('"', "&quot;");
        out.push_str(&format!("    {}[\"{}\"]\n", idx[node.as_str()], label));
    }
    for (src, dst) in edges {
        out.push_str(&format!(
            "    {} --> {}\n",
            idx[src.as_str()], idx[dst.as_str()]
        ));
    }
    out
}

fn dot(nodes: &[String], edges: &[(String, String)]) -> String {
    let idx = index_map(nodes);
    let mut out = String::from("digraph okf {\n");
    for node in nodes {
        let label = node.replace('\\', "\\\\").replace('"', "\\\"");
        out.push_str(&format!("    {} [label=\"{}\"];\n", idx[node.as_str()], label));
    }
    for (src, dst) in edges {
        out.push_str(&format!(
            "    {} -> {};\n",
            idx[src.as_str()], idx[dst.as_str()]
        ));
    }
    out.push_str("}\n");
    out
}

fn graphml(nodes: &[String], edges: &[(String, String)]) -> String {
    let idx = index_map(nodes);
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">\n");
    out.push_str("  <key id=\"label\" for=\"node\" attr.name=\"label\" attr.type=\"string\"/>\n");
    out.push_str("  <graph id=\"okf\" edgedefault=\"directed\">\n");
    for node in nodes {
        out.push_str(&format!(
            "    <node id=\"{}\"><data key=\"label\">{}</data></node>\n",
            idx[node.as_str()],
            xml_escape(node)
        ));
    }
    for (i, (src, dst)) in edges.iter().enumerate() {
        out.push_str(&format!(
            "    <edge id=\"e{}\" source=\"{}\" target=\"{}\"/>\n",
            i,
            idx[src.as_str()],
            idx[dst.as_str()]
        ));
    }
    out.push_str("  </graph>\n");
    out.push_str("</graphml>\n");
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::loader::Bundle;
    use crate::graph::build::build_graph;
    use crate::model::concept::Concept;
    use crate::model::frontmatter::Frontmatter;
    use indexmap::IndexMap;
    use serde_yaml::Value;

    fn concept(id: &str, yaml: &str) -> Concept {
        let map: IndexMap<String, Value> = serde_yaml::from_str(yaml).unwrap();
        Concept {
            id: ConceptId::from_relative(id),
            frontmatter: Frontmatter::from_map(map),
            body: String::new(),
        }
    }

    fn graph() -> LinkGraph {
        // a → b, b → c
        let bundle = Bundle {
            root: std::path::PathBuf::from("."),
            concepts: vec![
                concept("a", "refs:\n- /b"),
                concept("b", "refs:\n- /c"),
                concept("c", "type: X"),
            ],
        };
        build_graph(&bundle)
    }

    #[test]
    fn mermaid_is_stable() {
        let out = render(&graph(), RenderFormat::Mermaid, None);
        assert_eq!(
            out,
            "graph LR\n    n0[\"/a\"]\n    n1[\"/b\"]\n    n2[\"/c\"]\n    n0 --> n1\n    n1 --> n2\n"
        );
    }

    #[test]
    fn dot_is_stable() {
        let out = render(&graph(), RenderFormat::Dot, None);
        assert_eq!(
            out,
            "digraph okf {\n    n0 [label=\"/a\"];\n    n1 [label=\"/b\"];\n    n2 [label=\"/c\"];\n    n0 -> n1;\n    n1 -> n2;\n}\n"
        );
    }

    #[test]
    fn graphml_is_stable() {
        let out = render(&graph(), RenderFormat::Graphml, None);
        assert!(out.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml"));
        assert!(out.contains("<node id=\"n0\"><data key=\"label\">/a</data></node>"));
        assert!(out.contains("<edge id=\"e0\" source=\"n0\" target=\"n1\"/>"));
    }

    #[test]
    fn subtree_limits_to_reachable() {
        // reachable from /b is {b, c} with edge b→c only.
        let out = render(&graph(), RenderFormat::Mermaid, Some("b"));
        assert_eq!(
            out,
            "graph LR\n    n0[\"/b\"]\n    n1[\"/c\"]\n    n0 --> n1\n"
        );
    }
}
