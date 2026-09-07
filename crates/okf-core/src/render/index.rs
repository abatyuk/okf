//! Generate `index.md` files for progressive disclosure.
//!
//! For every directory in the bundle we emit an `index.md` that lists the concepts living
//! directly in that directory (grouped by their OKF `type`, one `#` section per type) plus
//! links to immediate subdirectories, following the OKF spec's index format:
//!
//! ```markdown
//! # Section / Group Heading
//!
//! * [Title](relative-url) - short description of item
//!
//! # Subdirectories
//!
//! * [subdir/](subdir/)
//! ```
//!
//! Output is deterministic (types and entries are sorted) so re-running yields no diff, and
//! [`write_indexes`] preserves any pre-existing `index.md` frontmatter (e.g. the bundle-root
//! `okf_version`) so the write is lossless and idempotent. Concept files and `log.md` are
//! never touched.
use crate::bundle::loader::Bundle;
use crate::error::Result;
use crate::model::concept::Concept;
use crate::parse::markdown::split_frontmatter;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// A generated `index.md` for one directory of the bundle.
#[derive(Debug, Clone)]
pub struct IndexFile {
    /// Directory this index belongs to, relative to the bundle root (empty for the root).
    pub dir: PathBuf,
    /// The generated markdown body (no frontmatter). Ends with a single trailing newline.
    pub content: String,
}

/// Last path segment of a bundle id (`/tables/customers` → `customers`).
fn stem_of(id: &str) -> &str {
    id.trim_start_matches('/').rsplit('/').next().unwrap_or(id)
}

/// Generate `index.md` content for every directory in the bundle, sorted by directory path
/// (root first). Pure — reads no filesystem, so the content is stable across runs.
pub fn generate_indexes(bundle: &Bundle) -> Vec<IndexFile> {
    // dir path -> concepts living directly in it. Every ancestor dir is present as a key
    // (even with no direct concepts) so we still emit an index that links onward.
    let mut direct: BTreeMap<String, Vec<&Concept>> = BTreeMap::new();
    // dir path -> immediate subdirectory names.
    let mut subdirs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    // The root always gets an index, even for an empty bundle.
    direct.entry(String::new()).or_default();

    for c in &bundle.concepts {
        let rel = c.id.0.trim_start_matches('/');
        let segs: Vec<&str> = rel.split('/').collect();
        let dir_segs = &segs[..segs.len().saturating_sub(1)];
        let dir = dir_segs.join("/");
        direct.entry(dir).or_default().push(c);

        // Register the ancestor chain and each parent→child subdirectory link.
        let mut parent = String::new();
        for seg in dir_segs {
            subdirs
                .entry(parent.clone())
                .or_default()
                .insert((*seg).to_string());
            let child_dir = if parent.is_empty() {
                (*seg).to_string()
            } else {
                format!("{parent}/{seg}")
            };
            direct.entry(child_dir.clone()).or_default();
            parent = child_dir;
        }
    }

    let empty = BTreeSet::new();
    direct
        .iter()
        .map(|(dir, concepts)| IndexFile {
            dir: PathBuf::from(dir),
            content: render_dir_index(concepts, subdirs.get(dir).unwrap_or(&empty)),
        })
        .collect()
}

/// Render one directory's index: concepts grouped by `type`, then a `Subdirectories` section.
fn render_dir_index(concepts: &[&Concept], child_dirs: &BTreeSet<String>) -> String {
    // Group by type (empty/missing type → "Other"), types sorted alphabetically.
    let mut by_type: BTreeMap<&str, Vec<&Concept>> = BTreeMap::new();
    for c in concepts {
        let ty = c.concept_type().filter(|s| !s.is_empty()).unwrap_or("Other");
        by_type.entry(ty).or_default().push(c);
    }

    let mut out = String::new();
    for (ty, mut cs) in by_type {
        // Stable within a section: by title (falling back to stem), then by id.
        cs.sort_by(|a, b| {
            let ta = a.title().unwrap_or_else(|| stem_of(a.id.as_str()));
            let tb = b.title().unwrap_or_else(|| stem_of(b.id.as_str()));
            ta.cmp(tb).then_with(|| a.id.0.cmp(&b.id.0))
        });
        out.push_str(&format!("# {ty}\n\n"));
        for c in cs {
            let stem = stem_of(c.id.as_str());
            let title = c.title().unwrap_or(stem);
            let url = format!("{stem}.md");
            match c.description() {
                Some(d) => out.push_str(&format!("* [{title}]({url}) - {d}\n")),
                None => out.push_str(&format!("* [{title}]({url})\n")),
            }
        }
        out.push('\n');
    }

    if !child_dirs.is_empty() {
        out.push_str("# Subdirectories\n\n");
        for name in child_dirs {
            out.push_str(&format!("* [{name}/]({name}/)\n"));
        }
        out.push('\n');
    }

    // Normalize to exactly one trailing newline.
    let mut content = out.trim_end().to_string();
    content.push('\n');
    content
}

/// Write a generated `index.md` into every directory of the bundle (backs
/// `okf docs --format index`). Idempotent: content is deterministic and any existing
/// `index.md` frontmatter is preserved verbatim, so re-running produces no diff. Only
/// `index.md` files are written — concept documents and `log.md` are never touched.
///
/// Returns the paths written, in directory order.
pub fn write_indexes(bundle: &Bundle) -> Result<Vec<PathBuf>> {
    let mut written = Vec::new();
    for idx in generate_indexes(bundle) {
        let target = if idx.dir.as_os_str().is_empty() {
            bundle.root.join("index.md")
        } else {
            bundle.root.join(&idx.dir).join("index.md")
        };

        // Preserve any pre-existing frontmatter (e.g. root `okf_version`) losslessly.
        let final_content = match std::fs::read_to_string(&target) {
            Ok(existing) => match split_frontmatter(&existing).0 {
                Some(fm) => format!("---\n{fm}---\n{}", idx.content),
                None => idx.content.clone(),
            },
            Err(_) => idx.content.clone(),
        };

        std::fs::write(&target, final_content)
            .map_err(|e| crate::error::OkfError::Io(format!("{}: {e}", target.display())))?;
        written.push(target);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::loader::load_bundle;
    use std::path::{Path, PathBuf};

    fn sample_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample-bundle")
    }

    fn index_for<'a>(files: &'a [IndexFile], dir: &str) -> &'a IndexFile {
        files
            .iter()
            .find(|f| f.dir == PathBuf::from(dir))
            .unwrap_or_else(|| panic!("no index generated for {dir:?}"))
    }

    #[test]
    fn generates_one_index_per_directory() {
        let bundle = load_bundle(&sample_root()).unwrap();
        let files = generate_indexes(&bundle);
        let dirs: Vec<String> = files
            .iter()
            .map(|f| f.dir.to_string_lossy().into_owned())
            .collect();
        assert_eq!(dirs, vec!["", "metrics", "notes", "policies", "tables"]);
    }

    #[test]
    fn root_index_lists_sorted_subdirectories() {
        let bundle = load_bundle(&sample_root()).unwrap();
        let files = generate_indexes(&bundle);
        assert_eq!(
            index_for(&files, "").content,
            "# Subdirectories\n\n\
             * [metrics/](metrics/)\n\
             * [notes/](notes/)\n\
             * [policies/](policies/)\n\
             * [tables/](tables/)\n"
        );
    }

    #[test]
    fn leaf_index_groups_by_type_with_description() {
        let bundle = load_bundle(&sample_root()).unwrap();
        let files = generate_indexes(&bundle);

        // Policy has a description; Metric does not.
        assert_eq!(
            index_for(&files, "policies").content,
            "# Policy\n\n\
             * [Travel and expense policy](travel.md) - Rules and reimbursement rates for business travel.\n"
        );
        assert_eq!(
            index_for(&files, "metrics").content,
            "# Metric\n\n* [Revenue](revenue.md)\n"
        );
        assert_eq!(
            index_for(&files, "tables").content,
            "# BigQuery Table\n\n\
             * [Customers](customers.md) - The canonical customers dimension table.\n"
        );
    }

    // --- write path: copies the fixture into a temp dir so the committed fixture is untouched.

    fn copy_tree(src: &Path, dst: &Path) {
        std::fs::create_dir_all(dst).unwrap();
        for entry in std::fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let from = entry.path();
            let to = dst.join(entry.file_name());
            if from.is_dir() {
                copy_tree(&from, &to);
            } else {
                std::fs::copy(&from, &to).unwrap();
            }
        }
    }

    fn temp_copy() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("okf-index-test-{nanos}"));
        copy_tree(&sample_root(), &dir);
        dir
    }

    #[test]
    fn write_is_idempotent_and_preserves_root_frontmatter() {
        let root = temp_copy();

        // Give the root index frontmatter that must survive regeneration (spec allows
        // `okf_version` on the root index only).
        std::fs::write(root.join("index.md"), "---\nokf_version: \"0.2\"\n---\nold body\n").unwrap();

        let bundle = load_bundle(&root).unwrap();
        let written = write_indexes(&bundle).unwrap();
        assert_eq!(written.len(), 5);

        // Root frontmatter preserved, body replaced with the generated listing.
        let root_index = std::fs::read_to_string(root.join("index.md")).unwrap();
        assert!(root_index.starts_with("---\nokf_version: \"0.2\"\n---\n"));
        assert!(root_index.contains("# Subdirectories"));
        assert!(!root_index.contains("old body"));

        // A leaf index was written verbatim (no frontmatter).
        assert_eq!(
            std::fs::read_to_string(root.join("tables/index.md")).unwrap(),
            "# BigQuery Table\n\n* [Customers](customers.md) - The canonical customers dimension table.\n"
        );

        // Snapshot every index, re-run, and assert byte-identical output (no diff).
        let before: Vec<String> = written
            .iter()
            .map(|p| std::fs::read_to_string(p).unwrap())
            .collect();
        let bundle2 = load_bundle(&root).unwrap();
        write_indexes(&bundle2).unwrap();
        let after: Vec<String> = written
            .iter()
            .map(|p| std::fs::read_to_string(p).unwrap())
            .collect();
        assert_eq!(before, after, "re-running write_indexes must not change any file");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn write_does_not_touch_concept_files() {
        let root = temp_copy();
        let before = std::fs::read_to_string(root.join("tables/customers.md")).unwrap();
        let bundle = load_bundle(&root).unwrap();
        write_indexes(&bundle).unwrap();
        let after = std::fs::read_to_string(root.join("tables/customers.md")).unwrap();
        assert_eq!(before, after, "concept files must be left untouched");
        let _ = std::fs::remove_dir_all(&root);
    }
}
