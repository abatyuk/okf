//! Concept-level diff of the working-tree bundle vs a git ref (via the git CLI).
//!
//! Compares the concept set at `rev` (enumerated with `git ls-tree`, read with `git show`)
//! against the currently-loaded bundle, reporting which concept ids were added / removed /
//! modified. No printing — the CLI wave renders [`DiffResult`].
//!
//! Note (v1 limitation): `ls-tree` paths are treated as bundle-relative, i.e. the bundle root
//! is assumed to be the git repo root. A sub-directory bundle is a later refinement.
use crate::bundle::loader::Bundle;
use crate::error::Result;
use crate::model::concept::{Concept, ConceptId};
use crate::parse::parse_concept;
use crate::ports::git::Git;
use std::collections::BTreeMap;
use std::path::Path;

/// Reserved filenames that are structural, not concepts (mirrors `bundle::loader`).
const RESERVED: [&str; 2] = ["index.md", "log.md"];

/// The concept-level delta between a git ref and the current bundle. Ids are sorted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiffResult {
    /// In the current bundle, not at the ref.
    pub added: Vec<ConceptId>,
    /// At the ref, not in the current bundle.
    pub removed: Vec<ConceptId>,
    /// Present in both, but frontmatter or body differs.
    pub modified: Vec<ConceptId>,
}

impl DiffResult {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.modified.is_empty()
    }
}

/// Compute the concept-level diff of `bundle` against git ref `rev`.
pub fn diff(bundle: &Bundle, git: &dyn Git, rev: &str) -> Result<DiffResult> {
    // Concepts at the ref, keyed by id.
    let mut at_ref: BTreeMap<String, Concept> = BTreeMap::new();
    for path in git.ls_tree(rev)? {
        let Some(id) = concept_id_for(&path) else {
            continue;
        };
        let bytes = git.show(rev, Path::new(&path))?;
        let content = String::from_utf8_lossy(&bytes);
        let concept = parse_concept(id.clone(), &content)?;
        at_ref.insert(id.0, concept);
    }

    // Current concepts, keyed by id.
    let current: BTreeMap<&str, &Concept> =
        bundle.concepts.iter().map(|c| (c.id.0.as_str(), c)).collect();

    let mut result = DiffResult::default();

    for (id, cur) in &current {
        match at_ref.get(*id) {
            None => result.added.push(cur.id.clone()),
            Some(old) => {
                if old.frontmatter != cur.frontmatter || old.body != cur.body {
                    result.modified.push(cur.id.clone());
                }
            }
        }
    }
    for (id, old) in &at_ref {
        if !current.contains_key(id.as_str()) {
            result.removed.push(old.id.clone());
        }
    }

    result.added.sort_by(|a, b| a.0.cmp(&b.0));
    result.removed.sort_by(|a, b| a.0.cmp(&b.0));
    result.modified.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

/// Map a tracked path to a concept id, or `None` for non-concepts (non-`.md`, reserved).
fn concept_id_for(path: &str) -> Option<ConceptId> {
    let stem = path.strip_suffix(".md")?;
    let base = path.rsplit('/').next().unwrap_or(path);
    if RESERVED.contains(&base) {
        return None;
    }
    Some(ConceptId::from_relative(stem))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::frontmatter::Frontmatter;
    use crate::ports::git::FakeGit;
    use std::path::PathBuf;

    fn concept(id: &str, body: &str) -> Concept {
        parse_concept(
            ConceptId::from_relative(id),
            &format!("---\ntype: T\n---\n{body}"),
        )
        .unwrap()
    }

    fn bundle(concepts: Vec<Concept>) -> Bundle {
        Bundle {
            root: PathBuf::from("."),
            concepts,
        }
    }

    #[test]
    fn detects_added_removed_modified() {
        // Current bundle: keep (modified), plus new.
        let bundle = bundle(vec![
            concept("tables/customers", "changed body\n"),
            concept("tables/new", "brand new\n"),
        ]);

        let git = FakeGit::new()
            .with_tree(
                "HEAD",
                vec![
                    "tables/customers.md".to_string(),
                    "tables/gone.md".to_string(),
                    "index.md".to_string(), // reserved → ignored
                    "README.txt".to_string(), // non-md → ignored
                ],
            )
            .with_show("HEAD", "tables/customers.md", b"---\ntype: T\n---\noriginal body\n".to_vec())
            .with_show("HEAD", "tables/gone.md", b"---\ntype: T\n---\ngone\n".to_vec());

        let d = diff(&bundle, &git, "HEAD").unwrap();
        assert_eq!(ids(&d.added), vec!["/tables/new"]);
        assert_eq!(ids(&d.removed), vec!["/tables/gone"]);
        assert_eq!(ids(&d.modified), vec!["/tables/customers"]);
    }

    #[test]
    fn identical_content_is_not_modified() {
        let bundle = bundle(vec![concept("a", "same\n")]);
        let git = FakeGit::new()
            .with_tree("HEAD", vec!["a.md".to_string()])
            .with_show("HEAD", "a.md", b"---\ntype: T\n---\nsame\n".to_vec());
        let d = diff(&bundle, &git, "HEAD").unwrap();
        assert!(d.is_empty(), "unchanged concept should not appear");
    }

    fn ids(v: &[ConceptId]) -> Vec<&str> {
        v.iter().map(|i| i.0.as_str()).collect()
    }

    #[test]
    fn frontmatter_only_change_is_modified() {
        let bundle = bundle(vec![{
            let mut c = concept("a", "body\n");
            c.frontmatter = Frontmatter::from_map(
                [("type".to_string(), serde_yaml::Value::String("U".to_string()))]
                    .into_iter()
                    .collect(),
            );
            c
        }]);
        let git = FakeGit::new()
            .with_tree("HEAD", vec!["a.md".to_string()])
            .with_show("HEAD", "a.md", b"---\ntype: T\n---\nbody\n".to_vec());
        let d = diff(&bundle, &git, "HEAD").unwrap();
        assert_eq!(ids(&d.modified), vec!["/a"]);
    }
}
