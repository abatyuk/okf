//! Advisory health checks for fingerprinted sources. Standard kind-less provenance remains
//! valid without fingerprints. Local file/text sources are checked for readability even
//! before their first refresh; lint does not fetch URL sources or require a Git worktree.
use crate::check::lint::{Finding, RuleContext};
use crate::fingerprint::{Engine, Fingerprinter};
use crate::model::source::{parse_sources, SourceKind};
use crate::ports::fs::RealFs;
use crate::ports::git::RealGit;

pub const UNRECORDED_RULE: &str = "source-unrecorded";
pub const MISSING_RULE: &str = "source-missing";

pub fn run(ctx: &RuleContext) -> Vec<Finding> {
    let fs = RealFs;
    let git = RealGit;
    let engine = Engine::new(&ctx.bundle.root, &fs, &git, None);
    let mut findings = Vec::new();
    for concept in &ctx.bundle.concepts {
        let Some(value) = concept.frontmatter.get("sources") else {
            continue;
        };
        for source in parse_sources(value) {
            if source.kind.as_kind_str().trim().is_empty() {
                continue;
            }
            if source.fingerprint.is_empty() {
                findings.push(Finding {
                    rule: UNRECORDED_RULE.to_string(),
                    severity: ctx.config.source_unrecorded,
                    concept: Some(concept.id.0.clone()),
                    message: format!(
                        "source {} has no recorded fingerprint; run refresh after reviewing the source",
                        source.resource
                    ),
                });
            }
            if matches!(
                source.kind,
                SourceKind::File | SourceKind::LineRange | SourceKind::MarkdownHeading
            ) {
                if let Err(error) = engine.fingerprint(&source) {
                    findings.push(Finding {
                        rule: MISSING_RULE.to_string(),
                        severity: ctx.config.source_missing,
                        concept: Some(concept.id.0.clone()),
                        message: format!("cannot fingerprint source {}: {error}", source.resource),
                    });
                }
            }
        }
    }
    findings
}
