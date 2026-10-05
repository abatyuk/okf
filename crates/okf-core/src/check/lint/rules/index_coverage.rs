//! Read-only immediate-child navigation coverage using real Markdown links.
use crate::{
    bundle::loader::Bundle,
    error::{OkfError, Result},
    model::link::{classify, LinkKind},
    parse::markdown::split_frontmatter,
};
use pulldown_cmark::{Event, Parser, Tag};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexFinding {
    pub code: String,
    pub directory: String,
    pub missing_child: String,
    pub message: String,
}
fn glob(pattern: &str) -> Result<globset::GlobMatcher> {
    globset::GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map(|g| g.compile_matcher())
        .map_err(|e| OkfError::Usage(format!("invalid index exclusion {pattern:?}: {e}")))
}
fn excluded(path: &str, globs: &[globset::GlobMatcher]) -> bool {
    let mut part = path;
    loop {
        if globs
            .iter()
            .any(|g| g.is_match(part) || g.is_match(format!("{part}/")))
        {
            return true;
        }
        let Some((parent, _)) = part.rsplit_once('/') else {
            return false;
        };
        part = parent;
    }
}
fn decode(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            if i + 2 >= b.len() || !b[i + 1].is_ascii_hexdigit() || !b[i + 2].is_ascii_hexdigit() {
                return None;
            }
            out.push(u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).ok()?, 16).ok()?);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}
pub fn check_indexes(bundle: &Bundle, exclusions: &[String]) -> Result<Vec<IndexFinding>> {
    let globs = exclusions
        .iter()
        .map(|s| glob(s))
        .collect::<Result<Vec<_>>>()?;
    let mut inventory: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for concept in &bundle.concepts {
        let rel = format!("{}.md", concept.id.0.trim_start_matches('/'));
        if excluded(&rel, &globs) {
            continue;
        }
        let (dir, name) = rel.rsplit_once('/').unwrap_or(("", &rel));
        inventory.entry(dir.into()).or_default().insert(name.into());
        let mut child = dir;
        while !child.is_empty() {
            let (parent, name) = child.rsplit_once('/').unwrap_or(("", child));
            inventory
                .entry(parent.into())
                .or_default()
                .insert(format!("{name}/"));
            child = parent;
        }
    }
    let mut findings = Vec::new();
    for (directory, children) in inventory {
        let file = bundle.root.join(&directory).join("index.md");
        let regular_index = match std::fs::symlink_metadata(&file) {
            Ok(metadata) => metadata.file_type().is_file(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(OkfError::Io(format!("{}: {error}", file.display()))),
        };
        if !regular_index {
            findings.push(IndexFinding {
                code: "index-missing".into(),
                directory: directory.clone(),
                missing_child: String::new(),
                message: "eligible directory has no index.md".into(),
            });
            continue;
        }
        let text = std::fs::read_to_string(&file)
            .map_err(|e| OkfError::Io(format!("{}: {e}", file.display())))?;
        let (_, body) = split_frontmatter(&text);
        let mut covered = BTreeSet::new();
        for event in Parser::new(&body) {
            if let Event::Start(Tag::Link { dest_url, .. }) = event {
                if classify(&dest_url) == LinkKind::External {
                    continue;
                }
                let raw = dest_url.split(['#', '?']).next().unwrap_or_default();
                let Some(raw) = decode(raw) else {
                    continue;
                };
                let joined = if raw.starts_with('/') {
                    raw.trim_start_matches('/').to_owned()
                } else {
                    format!("{directory}/{raw}")
                };
                let mut parts = Vec::new();
                let mut above = false;
                for part in joined.split('/') {
                    match part {
                        "" | "." => {}
                        ".." => {
                            if parts.pop().is_none() {
                                above = true;
                            }
                        }
                        _ => parts.push(part),
                    }
                }
                if above {
                    continue;
                }
                let resolved = parts.join("/");
                let prefix = if directory.is_empty() {
                    String::new()
                } else {
                    format!("{directory}/")
                };
                let Some(child) = resolved.strip_prefix(&prefix) else {
                    continue;
                };
                if children.contains(child) {
                    covered.insert(child.to_owned());
                }
                if let Some(stem) = child.strip_suffix("/index.md") {
                    if !stem.contains('/') {
                        covered.insert(format!("{stem}/"));
                    }
                }
                if !child.contains('/') {
                    if children.contains(&format!("{child}.md")) {
                        covered.insert(format!("{child}.md"));
                    }
                    if children.contains(&format!("{child}/")) {
                        covered.insert(format!("{child}/"));
                    }
                }
            }
        }
        for child in children.difference(&covered) {
            findings.push(IndexFinding {
                code: "index-missing-entry".into(),
                directory: directory.clone(),
                missing_child: child.clone(),
                message: format!("index omits immediate child {child}"),
            });
        }
    }
    Ok(findings)
}
