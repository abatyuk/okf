//! Re-record fingerprints after a change is acknowledged (the "it's in sync again" op).
//!
//! `refresh` recomputes the current fingerprint for each of a concept's `sources[]` via
//! [`fingerprint::Engine`] and rewrites the entries losslessly — `resource`, `kind` and every
//! `extra` key (`id`, `author`, `usage_count`, …) are preserved via [`Source`]'s round-trip;
//! only `fingerprint` is replaced. It also stamps the concept's top-level `last_modified` with
//! `clock`'s "now". Because `okf stale` derives drift by comparing recorded vs. recomputed
//! fingerprints, refreshing them is exactly what clears a concept's stale status.
//!
//! A source whose fingerprint cannot be recomputed (unknown kind, missing artifact, `url`
//! with no network port) is left untouched and reported in [`RefreshResult::skipped`] rather
//! than failing the whole operation.
use std::path::{Path, PathBuf};

use serde_yaml::Value;

use crate::error::Result;
use crate::fingerprint::{Engine, Fingerprinter};
use crate::model::concept::ConceptId;
use crate::model::source::Source;
use crate::ports::clock::Clock;

use super::edit::{load_concept, save_concept};

/// A source whose fingerprint could not be recomputed.
#[derive(Debug, Clone)]
pub struct SkippedSource {
    pub resource: String,
    pub reason: String,
}

/// What `refresh` recomputed.
#[derive(Debug, Clone)]
pub struct RefreshResult {
    pub id: ConceptId,
    pub path: PathBuf,
    /// Resources whose recorded fingerprint changed.
    pub updated: Vec<String>,
    /// Resources whose fingerprint was recomputed and matched (already in sync).
    pub unchanged: Vec<String>,
    /// Resources that could not be fingerprinted (left as-is).
    pub skipped: Vec<SkippedSource>,
    pub last_modified: String,
}

/// Recompute and rewrite the `sources[]` fingerprints of the concept `id`, and update its
/// `last_modified`. `engine`'s `root` is the base that source resource paths resolve against.
pub fn refresh(root: &Path, id: &str, engine: &Engine, clock: &dyn Clock) -> Result<RefreshResult> {
    let cid = ConceptId::from_relative(id);
    let mut concept = load_concept(root, &cid)?;

    let mut updated = Vec::new();
    let mut unchanged = Vec::new();
    let mut skipped = Vec::new();

    if let Some(sources_val) = concept.frontmatter.map.get("sources").cloned() {
        if let Value::Sequence(entries) = sources_val {
            let mut rebuilt = Vec::with_capacity(entries.len());
            for (index, original) in entries.into_iter().enumerate() {
                let Some(mut source) = Source::from_value(&original) else {
                    skipped.push(SkippedSource {
                        resource: format!("sources[{index}]"),
                        reason: "malformed source: expected a mapping with resource and kind"
                            .to_string(),
                    });
                    rebuilt.push(original);
                    continue;
                };
                if source.resource.trim().is_empty() || source.kind.as_kind_str().trim().is_empty()
                {
                    skipped.push(SkippedSource {
                        resource: if source.resource.is_empty() {
                            format!("sources[{index}]")
                        } else {
                            source.resource.clone()
                        },
                        reason: "malformed source: resource and kind must be non-empty strings"
                            .to_string(),
                    });
                    rebuilt.push(original);
                    continue;
                }
                match engine.fingerprint(&source) {
                    Ok(fp) => {
                        if fp != source.fingerprint {
                            updated.push(source.resource.clone());
                            source.fingerprint = fp;
                        } else {
                            unchanged.push(source.resource.clone());
                        }
                        rebuilt.push(source.to_value());
                    }
                    Err(e) => {
                        skipped.push(SkippedSource {
                            resource: source.resource.clone(),
                            reason: e.to_string(),
                        });
                        rebuilt.push(original);
                    }
                }
            }
            concept
                .frontmatter
                .map
                .insert("sources".to_string(), Value::Sequence(rebuilt));
        } else {
            skipped.push(SkippedSource {
                resource: "sources".to_string(),
                reason: "malformed sources field: expected a sequence".to_string(),
            });
        }
    }

    let last_modified = clock.now_rfc3339();
    // Update in place (position preserved) or append.
    concept.frontmatter.map.insert(
        "last_modified".to_string(),
        Value::String(last_modified.clone()),
    );

    let path = save_concept(root, &concept)?;
    Ok(RefreshResult {
        id: cid,
        path,
        updated,
        unchanged,
        skipped,
        last_modified,
    })
}
