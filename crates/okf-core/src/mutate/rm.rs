//! Remove a concept; guard dangling backlinks unless `force`.
//!
//! Because a concept id *is* its file path, deleting a concept that others link to leaves
//! dangling references. `rm` therefore refuses (a `Usage` error naming the referrers) when
//! backlinks exist and `force` is false. With `force`, it removes the file anyway and reports
//! the now-dangling referrers in [`RmResult::dangling_referrers`] so the caller can follow up
//! (e.g. with `okf mv` beforehand, or `okf lint` after).
use std::path::{Path, PathBuf};

use crate::bundle::loader::load_bundle;
use crate::error::{OkfError, Result};
use crate::graph::backlinks::backlinks_of;
use crate::model::concept::ConceptId;
use crate::ontology::load::try_load;

use super::edit::id_to_path;

/// What `rm` did.
#[derive(Debug, Clone)]
pub struct RmResult {
    pub id: ConceptId,
    pub path: PathBuf,
    /// Whether the file was actually removed.
    pub removed: bool,
    /// Referrers that now dangle (populated on a forced removal).
    pub dangling_referrers: Vec<ConceptId>,
}

/// Remove the concept `id` from the bundle at `root`. Refuses (error) if backlinks exist and
/// `force` is false.
pub fn rm(root: &Path, id: &str, force: bool) -> Result<RmResult> {
    let cid = ConceptId::from_relative(id);
    let path = id_to_path(root, &cid);
    if !path.exists() {
        return Err(OkfError::Usage(format!(
            "cannot remove: no concept at {}",
            cid
        )));
    }

    let bundle = load_bundle(root)?;
    let ontology = try_load(root)?;
    let referrers = backlinks_of(&bundle, ontology.as_ref(), &cid.0);

    if !referrers.is_empty() && !force {
        let names: Vec<&str> = referrers.iter().map(|c| c.0.as_str()).collect();
        return Err(OkfError::Usage(format!(
            "refusing to remove {}: {} referrer(s) would dangle: {} — pass force to override",
            cid,
            referrers.len(),
            names.join(", ")
        )));
    }

    std::fs::remove_file(&path).map_err(|e| OkfError::Io(format!("{}: {e}", path.display())))?;

    Ok(RmResult {
        id: cid,
        path,
        removed: true,
        dangling_referrers: referrers,
    })
}
