//! Re-record fingerprints after a change is acknowledged (the "it's in sync again" op).
//!
//! `refresh` recomputes the current fingerprint for each of a concept's `sources[]` via
//! [`fingerprint::Engine`] and rewrites only the fingerprint value — `resource`, `kind` and every
//! `extra` key (`id`, `author`, `usage_count`, …) are preserved via [`Source`]'s round-trip;
//! only `fingerprint` is replaced. The operation time is returned as runtime output, not written
//! into standard source metadata. Because `okf stale` derives drift by comparing recorded vs. recomputed
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
    /// Runtime observation time, not standard `sources[].last_modified` metadata.
    pub refreshed_at: String,
}

/// Recompute and rewrite the `sources[]` fingerprints of the concept `id`, and update its
/// `last_modified`. `engine`'s `root` is the base that source resource paths resolve against.
pub fn refresh(root: &Path, id: &str, engine: &Engine, clock: &dyn Clock) -> Result<RefreshResult> {
    let cid = ConceptId::parse(id)?;
    let mut concept = load_concept(root, &cid)?;

    let mut updated = Vec::new();
    let mut unchanged = Vec::new();
    let mut skipped = Vec::new();

    if let Some(sources_val) = concept.frontmatter.map.get("sources").cloned() {
        if let Value::Sequence(entries) = sources_val {
            let mut rebuilt = Vec::with_capacity(entries.len());
            for (index, original) in entries.into_iter().enumerate() {
                let Some(source) = Source::from_value(&original) else {
                    skipped.push(SkippedSource {
                        resource: format!("sources[{index}]"),
                        reason: "malformed source: expected a mapping with resource".to_string(),
                    });
                    rebuilt.push(original);
                    continue;
                };
                if source.resource.trim().is_empty() {
                    skipped.push(SkippedSource {
                        resource: format!("sources[{index}]"),
                        reason: "malformed source: resource must be a non-empty string".to_string(),
                    });
                    rebuilt.push(original);
                    continue;
                }
                if source.kind.as_kind_str().trim().is_empty() {
                    skipped.push(SkippedSource {
                        resource: source.resource.clone(),
                        reason:
                            "standard source has no fingerprint `kind` extension; left unchanged"
                                .to_string(),
                    });
                    rebuilt.push(original);
                    continue;
                }
                match engine.fingerprint(&source) {
                    Ok(fp) => {
                        if fp != source.fingerprint {
                            updated.push(source.resource.clone());
                            let mut updated_entry = original.clone();
                            if let Some(map) = updated_entry.as_mapping_mut() {
                                map.insert(Value::String("fingerprint".to_string()), fp.to_value());
                            }
                            rebuilt.push(updated_entry);
                        } else {
                            unchanged.push(source.resource.clone());
                            rebuilt.push(original);
                        }
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

    let refreshed_at = clock.now_rfc3339();

    let path = save_concept(root, &concept)?;
    Ok(RefreshResult {
        id: cid,
        path,
        updated,
        unchanged,
        skipped,
        refreshed_at,
    })
}
