//! Resolve a link/concept-id to a concrete file path (bundle-relative).
use crate::bundle::loader::Bundle;
use crate::model::concept::ConceptId;
use crate::model::link::resolve_link;
use std::path::PathBuf;

/// The result of resolving a link/id against a bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// Canonical concept id (leading slash, `.md`/fragment stripped, `.`/`..` collapsed).
    pub id: ConceptId,
    /// Bundle-relative file path, e.g. `tables/customers.md`.
    pub path: PathBuf,
    /// Whether a concept with this id is actually loaded in the bundle (a broken link
    /// resolves fine but does not exist).
    pub exists: bool,
}

impl Resolved {
    /// Absolute path within a bundle root.
    pub fn abs_path(&self, bundle: &Bundle) -> PathBuf {
        bundle.root.join(&self.path)
    }
}

/// Resolve `link` (bundle-relative `/…`, relative `./…`/`../…`, or bare id) to a concrete
/// bundle-relative path. Relative links resolve against `from` (the containing concept id);
/// pass `None` to resolve from the bundle root.
pub fn resolve(bundle: &Bundle, from: Option<&str>, link: &str) -> Resolved {
    let from_id = from
        .map(ConceptId::from_relative)
        .unwrap_or_else(|| ConceptId("/".to_string()));
    let id = resolve_link(&from_id, link);
    let rel = id.0.trim_start_matches('/');
    let path = PathBuf::from(format!("{rel}.md"));
    // Structural resources such as index.md/log.md are deliberately not loaded as concepts,
    // but `resolve` should still report their physical existence truthfully.
    let exists = bundle.get(id.0.as_str()).is_some() || bundle.root.join(&path).is_file();
    Resolved { id, path, exists }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::concept::Concept;
    use crate::model::frontmatter::Frontmatter;

    fn bundle() -> Bundle {
        Bundle {
            root: PathBuf::from("/bundle"),
            concepts: vec![Concept {
                id: ConceptId::from_relative("tables/customers"),
                frontmatter: Frontmatter::new(),
                body: String::new(),
            }],
        }
    }

    #[test]
    fn resolves_id_to_path_and_existence() {
        let b = bundle();
        let r = resolve(&b, None, "/tables/customers.md");
        assert_eq!(r.id.0, "/tables/customers");
        assert_eq!(r.path, PathBuf::from("tables/customers.md"));
        assert!(r.exists);
        assert_eq!(r.abs_path(&b), PathBuf::from("/bundle/tables/customers.md"));
    }

    #[test]
    fn broken_link_resolves_but_does_not_exist() {
        let r = resolve(&bundle(), None, "tables/ghost");
        assert_eq!(r.path, PathBuf::from("tables/ghost.md"));
        assert!(!r.exists);
    }

    #[test]
    fn relative_link_uses_from_context() {
        let r = resolve(&bundle(), Some("policies/travel"), "../tables/customers");
        assert_eq!(r.id.0, "/tables/customers");
        assert!(r.exists);
    }
}
