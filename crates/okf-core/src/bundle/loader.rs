//! Walk a directory into concepts; honor reserved filenames (index.md/log.md).
use crate::error::{OkfError, Result};
use crate::model::concept::{Concept, ConceptId};
use crate::model::frontmatter::Frontmatter;
use crate::parse::{parse_concept, yaml::parse_frontmatter};
use std::io::BufRead;
use std::path::{Path, PathBuf};

use super::walk::walk_markdown;

/// Reserved filenames that are structural, not concepts.
const RESERVED: [&str; 2] = ["index.md", "log.md"];

/// A loaded bundle: its root and the concepts it contains (sorted by id).
#[derive(Debug, Clone)]
pub struct Bundle {
    pub root: PathBuf,
    pub concepts: Vec<Concept>,
}

impl Bundle {
    /// Find a concept by id. Accepts ids with or without a leading slash.
    pub fn get(&self, id: &str) -> Option<&Concept> {
        let target = ConceptId::from_relative(id);
        self.concepts
            .binary_search_by(|concept| concept.id.0.cmp(&target.0))
            .ok()
            .map(|index| &self.concepts[index])
    }
}

/// Load one concept directly instead of walking and parsing the entire bundle.
pub fn load_concept(root: &Path, id: &str) -> Result<Option<Concept>> {
    let id = ConceptId::parse(id)?;
    let path = root.join(format!("{}.md", id.0.trim_start_matches('/')));
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_file() => {}
        Ok(_) => return Ok(None),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(OkfError::Io(format!("{}: {e}", path.display()))),
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))?;
    parse_concept(id, &content).map(Some)
}

/// Whether an id maps to a regular, non-reserved concept document.
pub fn concept_exists(root: &Path, id: &ConceptId) -> bool {
    let rel = id.0.trim_start_matches('/');
    let Some(name) = Path::new(rel).file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if matches!(name, "index" | "log") {
        return false;
    }
    std::fs::symlink_metadata(root.join(format!("{rel}.md")))
        .is_ok_and(|metadata| metadata.file_type().is_file())
}

/// Walk `root` and load every `*.md` file except reserved
/// names into a [`Bundle`].
pub fn load_bundle(root: &Path) -> Result<Bundle> {
    load_bundle_with(root, true)
}

/// Load concept ids and frontmatter while leaving bodies unread. Commands which only inspect
/// metadata avoid allocating and parsing the rest of every Markdown document.
pub fn load_bundle_metadata(root: &Path) -> Result<Bundle> {
    load_bundle_with(root, false)
}

fn load_bundle_with(root: &Path, include_body: bool) -> Result<Bundle> {
    if !root.is_dir() {
        return Err(OkfError::Environment(format!(
            "bundle path is not a directory: {}",
            root.display()
        )));
    }

    let mut concepts = Vec::new();
    for rel in walk_markdown(root)? {
        let path = root.join(&rel);
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if RESERVED.contains(&file_name) {
            continue;
        }

        let rel_str = rel.to_string_lossy();
        let stem = rel_str
            .strip_suffix(".md")
            .ok_or_else(|| OkfError::Internal(format!("expected .md suffix: {rel_str}")))?;
        let id = ConceptId::from_relative(stem);

        let concept = if include_body {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))?;
            parse_concept(id, &content)?
        } else {
            load_metadata(id, &path)?
        };
        concepts.push(concept);
    }

    // Filesystem path ordering is not always concept-id ordering. For example,
    // `tui/a.md` sorts before `tui-rework.md` as a path because `tui` is a full
    // component, while `/tui-rework` sorts before `/tui/a` as a string because
    // `-` precedes `/`. Bundle::get uses binary search, so establish its stated
    // id-ordering invariant explicitly after loading.
    concepts.sort_by(|a, b| a.id.0.cmp(&b.id.0));

    Ok(Bundle {
        root: root.to_path_buf(),
        concepts,
    })
}

fn load_metadata(id: ConceptId, path: &Path) -> Result<Concept> {
    let file =
        std::fs::File::open(path).map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))?;
    let mut lines = std::io::BufReader::new(file).lines();
    let Some(first) = lines.next().transpose().map_err(OkfError::from)? else {
        return Ok(Concept {
            id,
            frontmatter: Frontmatter::new(),
            body: String::new(),
        });
    };
    if first.trim_end_matches('\r') != "---" {
        return Ok(Concept {
            id,
            frontmatter: Frontmatter::new(),
            body: String::new(),
        });
    }

    let mut yaml = String::new();
    let mut closed = false;
    for line in lines {
        let line = line.map_err(OkfError::from)?;
        if line.trim_end_matches('\r') == "---" {
            closed = true;
            break;
        }
        yaml.push_str(&line);
        yaml.push('\n');
    }
    let frontmatter = if closed {
        Frontmatter::from_map(parse_frontmatter(&yaml)?)
    } else {
        Frontmatter::new()
    };
    Ok(Concept {
        id,
        frontmatter,
        body: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_loader_preserves_frontmatter_without_body() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("note.md"),
            "---\r\ntype: Note\r\ntitle: Example\r\n---\r\nlarge body\r\n",
        )
        .unwrap();

        let full = load_bundle(root.path()).unwrap();
        let metadata = load_bundle_metadata(root.path()).unwrap();
        assert_eq!(
            metadata.concepts[0].frontmatter,
            full.concepts[0].frontmatter
        );
        assert!(metadata.concepts[0].body.is_empty());
        assert_eq!(metadata.get("note").unwrap().title(), Some("Example"));
    }

    #[test]
    fn direct_load_does_not_parse_unrelated_documents() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("good.md"), "---\ntype: Note\n---\nbody\n").unwrap();
        std::fs::write(root.path().join("bad.md"), "---\ntype: [\n---\n").unwrap();

        let concept = load_concept(root.path(), "good").unwrap().unwrap();
        assert_eq!(concept.concept_type(), Some("Note"));
        assert!(load_bundle(root.path()).is_err());
    }

    #[test]
    fn bundle_is_sorted_by_concept_id_when_file_and_directory_prefixes_collide() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("tui")).unwrap();
        std::fs::write(root.path().join("tui/a.md"), "---\ntype: Note\n---\n").unwrap();
        std::fs::write(root.path().join("tui-rework.md"), "---\ntype: Note\n---\n").unwrap();

        let bundle = load_bundle(root.path()).unwrap();
        let ids: Vec<&str> = bundle.concepts.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec!["/tui-rework", "/tui/a"]);
        assert!(bundle.get("/tui-rework").is_some());
        assert!(bundle.get("/tui/a").is_some());
    }
}
