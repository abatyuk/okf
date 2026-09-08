//! Advisory lint rule engine.
//!
//! Every check here is an *opinion*, not conformance — broken links, missing `title` /
//! `description`, orphaned concepts, ontology violations. Each rule has a [`Severity`] that is
//! overridable through [`LintConfig`]. The engine runs every rule over a [`Bundle`] and
//! collects typed [`Finding`]s; it prints nothing. Whether the run should fail the CLI is a
//! pure function of the findings and a [`FailOn`] threshold ([`meets_threshold`]); core only
//! computes the boolean, the CLI maps it to exit code 1.
pub mod rules;

use std::str::FromStr;

use crate::bundle::loader::Bundle;
use crate::error::OkfError;
use crate::graph::build::{build_graph, LinkGraph};
use crate::ontology::schema::Ontology;

/// Severity of a lint finding, ordered `Info < Warn < Error` so a threshold is a simple
/// comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warn,
    Error,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warn => "warn",
            Severity::Error => "error",
        }
    }
}

/// A single advisory finding.
#[derive(Debug, Clone)]
pub struct Finding {
    pub rule: String,
    pub severity: Severity,
    pub concept: Option<String>,
    pub message: String,
}

/// The `--fail-on` threshold: the lowest severity that makes a lint run "fail" (CLI exit 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FailOn {
    /// Never fail, whatever the findings.
    Never,
    /// Fail on any finding at/above `info` (i.e. any finding at all).
    Info,
    /// Fail on any finding at/above `warn`.
    Warn,
    /// Fail on any finding at/above `error` (the default).
    #[default]
    Error,
    /// Fail if there is any finding at all (alias of `info`, kept for CLI vocabulary).
    Any,
}

impl FailOn {
    /// The minimum severity that trips the threshold, or `None` for [`FailOn::Never`].
    pub fn min_severity(self) -> Option<Severity> {
        match self {
            FailOn::Never => None,
            FailOn::Info | FailOn::Any => Some(Severity::Info),
            FailOn::Warn => Some(Severity::Warn),
            FailOn::Error => Some(Severity::Error),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            FailOn::Never => "never",
            FailOn::Info => "info",
            FailOn::Warn => "warn",
            FailOn::Error => "error",
            FailOn::Any => "any",
        }
    }
}

impl FromStr for FailOn {
    type Err = OkfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "never" => FailOn::Never,
            "info" => FailOn::Info,
            "warn" => FailOn::Warn,
            "error" => FailOn::Error,
            "any" => FailOn::Any,
            other => {
                return Err(OkfError::Usage(format!(
                    "invalid --fail-on {other:?}: expected never, info, warn, error, or any"
                )))
            }
        })
    }
}

/// Whether `findings` meet/exceed the `fail_on` threshold — the boolean that drives the CLI's
/// exit-1 path. Pure: no I/O, no printing.
pub fn meets_threshold(findings: &[Finding], fail_on: FailOn) -> bool {
    match fail_on.min_severity() {
        None => false,
        Some(min) => findings.iter().any(|f| f.severity >= min),
    }
}

/// Per-rule severities, overridable from a config file (parsing that file is out of scope —
/// this is just the struct + defaults). A rule fires with the severity carried here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LintConfig {
    pub broken_link: Severity,
    pub missing_title: Severity,
    pub missing_description: Severity,
    pub orphan: Severity,
    pub ontology_violation: Severity,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            broken_link: Severity::Error,
            missing_title: Severity::Warn,
            missing_description: Severity::Info,
            orphan: Severity::Info,
            ontology_violation: Severity::Warn,
        }
    }
}

/// Everything a rule needs: the bundle, its prebuilt link graph, the (optional) ontology, and
/// the per-rule severities. Built once by [`lint_bundle`] and shared across rules.
pub struct RuleContext<'a> {
    pub bundle: &'a Bundle,
    pub graph: &'a LinkGraph,
    pub ontology: Option<&'a Ontology>,
    pub config: &'a LintConfig,
}

/// Run every lint rule over a bundle and collect the findings. The ontology is optional —
/// when absent, the ontology-violation rule is skipped (the rest still run). Findings are
/// returned rule-by-rule in bundle order; the caller decides how to sort/render.
pub fn lint_bundle(
    bundle: &Bundle,
    ontology: Option<&Ontology>,
    config: &LintConfig,
) -> Vec<Finding> {
    let graph = build_graph(bundle, ontology);
    let ctx = RuleContext {
        bundle,
        graph: &graph,
        ontology,
        config,
    };

    let mut findings = Vec::new();
    findings.extend(rules::broken_link::run(&ctx));
    findings.extend(rules::missing_title::run(&ctx));
    findings.extend(rules::missing_description::run(&ctx));
    findings.extend(rules::orphan::run(&ctx));
    findings.extend(rules::ontology_violation::run(&ctx));
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(sev: Severity) -> Finding {
        Finding {
            rule: "x".to_string(),
            severity: sev,
            concept: None,
            message: String::new(),
        }
    }

    #[test]
    fn fail_on_parses_vocabulary() {
        assert_eq!("never".parse::<FailOn>().unwrap(), FailOn::Never);
        assert_eq!("info".parse::<FailOn>().unwrap(), FailOn::Info);
        assert_eq!("warn".parse::<FailOn>().unwrap(), FailOn::Warn);
        assert_eq!("error".parse::<FailOn>().unwrap(), FailOn::Error);
        assert_eq!("any".parse::<FailOn>().unwrap(), FailOn::Any);
        assert!("bogus".parse::<FailOn>().is_err());
    }

    #[test]
    fn default_fail_on_is_error() {
        assert_eq!(FailOn::default(), FailOn::Error);
    }

    #[test]
    fn threshold_error_ignores_lower_severities() {
        let f = vec![finding(Severity::Info), finding(Severity::Warn)];
        assert!(!meets_threshold(&f, FailOn::Error));
        assert!(meets_threshold(&f, FailOn::Warn));
        assert!(meets_threshold(&f, FailOn::Info));
        assert!(meets_threshold(&f, FailOn::Any));
        assert!(!meets_threshold(&f, FailOn::Never));
    }

    #[test]
    fn threshold_error_trips_on_error() {
        let f = vec![finding(Severity::Info), finding(Severity::Error)];
        assert!(meets_threshold(&f, FailOn::Error));
    }

    #[test]
    fn never_never_fails_and_empty_never_trips() {
        assert!(!meets_threshold(&[finding(Severity::Error)], FailOn::Never));
        assert!(!meets_threshold(&[], FailOn::Info));
    }
}
