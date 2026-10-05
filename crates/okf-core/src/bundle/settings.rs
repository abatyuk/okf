//! Optional local interpretation settings. Named settings never register or leak to other bundles.
use super::context::{current_context, Context};
use crate::{
    error::{OkfError, Result},
    ontology::{load, schema::Ontology},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuerySettings {
    pub scan_limit: Option<usize>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LintSettings {
    #[serde(default)]
    pub index_exclude: Vec<String>,
    pub finding_budget: Option<usize>,
    pub ontology_violation: Option<String>,
    pub index_coverage: Option<String>,
    pub source_unrecorded: Option<String>,
    pub broken_link: Option<String>,
    pub missing_title: Option<String>,
    pub missing_description: Option<String>,
    pub orphan: Option<String>,
    pub spec_v02: Option<String>,
    pub source_missing: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewSettings {
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub expand: Vec<String>,
    #[serde(default)]
    pub extended_output: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetSettings {
    #[serde(default = "default_fields")]
    pub fields: Vec<String>,
    #[serde(default = "threshold")]
    pub max_distinct_values: usize,
    #[serde(default)]
    pub include_high_cardinality: Vec<String>,
}
fn default_fields() -> Vec<String> {
    vec!["type".into(), "tags[]".into()]
}
fn threshold() -> usize {
    100
}
impl Default for FacetSettings {
    fn default() -> Self {
        Self {
            fields: default_fields(),
            max_distinct_values: 100,
            include_high_cardinality: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleSettings {
    pub ontology: Option<String>,
    #[serde(default)]
    pub query: QuerySettings,
    #[serde(default)]
    pub lint: LintSettings,
    #[serde(default)]
    pub views: BTreeMap<String, ViewSettings>,
    #[serde(default)]
    pub facets: FacetSettings,
}
#[derive(Debug, Clone, Serialize)]
pub struct EffectiveSettings {
    pub bundle: Option<String>,
    pub ontology_path: Option<PathBuf>,
    pub ontology_digest: Option<String>,
    pub settings: BundleSettings,
}
pub fn for_context(context: &Context, root: &Path) -> Result<EffectiveSettings> {
    let abs = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let id = context
        .catalog
        .as_ref()
        .and_then(|c| c.entries.values().find(|e| e.root == abs))
        .map(|e| e.id.clone());
    let table = context
        .config
        .settings
        .get("bundle_settings")
        .and_then(toml::Value::as_table);
    let key = id.as_deref().unwrap_or("default");
    let settings: BundleSettings = match table.and_then(|t| {
        if id.is_none() && context.catalog.is_some() {
            None
        } else {
            t.get(key)
        }
    }) {
        Some(v) => v
            .clone()
            .try_into()
            .map_err(|e| OkfError::Usage(format!("invalid bundle settings {key:?}: {e}")))?,
        None => BundleSettings::default(),
    };
    validate_settings(&settings)?;
    let base = context
        .config_path
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or(root);
    let ontology_path = if let Some(path) = &settings.ontology {
        Some(base.join(path))
    } else {
        load::find_ontology(root)
    };
    let ontology_digest = ontology_path
        .as_ref()
        .map(|p| std::fs::read(p).map(|bytes| crate::fingerprint::canonicalize::sha256_hex(&bytes)))
        .transpose()
        .map_err(|e| OkfError::Environment(format!("cannot read configured ontology: {e}")))?;
    Ok(EffectiveSettings {
        bundle: id,
        ontology_path,
        ontology_digest,
        settings,
    })
}
pub fn load_for(root: &Path) -> Result<(EffectiveSettings, Option<Ontology>)> {
    let context = current_context(Some(&root.to_string_lossy()), None)?;
    let effective = for_context(&context, root)?;
    let ontology = effective
        .ontology_path
        .as_ref()
        .map(|p| load::load_ontology(p))
        .transpose()?;
    Ok((effective, ontology))
}

enum HistoricalFile {
    External,
    Missing,
    Bytes(Vec<u8>),
}
fn historical(root: &Path, path: &Path, revision: &str) -> Result<HistoricalFile> {
    let run = |args: &[&str]| {
        std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .env("GIT_NO_LAZY_FETCH", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .args(args)
            .output()
    };
    let repo = run(&["rev-parse", "--show-toplevel"])?;
    if !repo.status.success() {
        return Err(OkfError::Environment(
            "historical settings require available local Git repository".into(),
        ));
    }
    let repo = PathBuf::from(String::from_utf8_lossy(&repo.stdout).trim());
    let abs = super::catalog::absolute(path)?;
    let Ok(rel) = abs.strip_prefix(&repo) else {
        return Ok(HistoricalFile::External);
    };
    if !run(&["rev-parse", "--verify", &format!("{revision}^{{commit}}")])?
        .status
        .success()
    {
        return Err(OkfError::Environment(format!(
            "historical settings revision unavailable: {revision}"
        )));
    }
    let inventory = run(&[
        "ls-tree",
        "-z",
        revision,
        "--",
        &format!(":(top){}", rel.to_string_lossy()),
    ])?;
    if !inventory.status.success() {
        return Err(OkfError::Environment(format!(
            "cannot inspect historical settings {}",
            path.display()
        )));
    }
    if inventory.stdout.is_empty() {
        return Ok(HistoricalFile::Missing);
    }
    let output = run(&["show", &format!("{revision}:{}", rel.to_string_lossy())])?;
    if !output.status.success() {
        return Err(OkfError::Environment(format!(
            "historical interpretation file unavailable: {} at {revision}",
            path.display()
        )));
    }
    Ok(HistoricalFile::Bytes(output.stdout))
}
fn validate_settings(settings: &BundleSettings) -> Result<()> {
    if settings.query.scan_limit == Some(0)
        || settings.lint.finding_budget == Some(0)
        || settings.facets.max_distinct_values == 0
    {
        return Err(OkfError::Usage(
            "configured query, lint and facet limits must be positive".into(),
        ));
    }
    for view in settings.views.values() {
        for field in &view.columns {
            if field.starts_with('$') {
                if !matches!(field.as_str(), "$id" | "$bundle" | "$version") {
                    return Err(OkfError::Usage(format!("unknown computed field {field:?}")));
                }
            } else {
                crate::query::selector::Selector::parse(field)?;
            }
        }
    }
    for field in settings
        .facets
        .fields
        .iter()
        .chain(&settings.facets.include_high_cardinality)
    {
        crate::query::selector::Selector::parse(field)?;
    }
    for glob in &settings.lint.index_exclude {
        globset::GlobBuilder::new(glob)
            .literal_separator(true)
            .build()
            .map_err(|e| OkfError::Usage(format!("invalid index exclusion {glob:?}: {e}")))?;
    }
    for (rule, severity) in [
        ("broken_link", &settings.lint.broken_link),
        ("missing_title", &settings.lint.missing_title),
        ("missing_description", &settings.lint.missing_description),
        ("orphan", &settings.lint.orphan),
        ("ontology_violation", &settings.lint.ontology_violation),
        ("spec_v02", &settings.lint.spec_v02),
        ("source_missing", &settings.lint.source_missing),
        ("source_unrecorded", &settings.lint.source_unrecorded),
        ("index_coverage", &settings.lint.index_coverage),
    ] {
        if let Some(value) = severity {
            if !matches!(value.as_str(), "off" | "info" | "warn" | "error") {
                return Err(OkfError::Usage(format!(
                    "invalid {rule} lint severity {value:?}; expected off, info, warn, or error"
                )));
            }
        }
    }
    Ok(())
}
/// Historical in-repository settings are resolved at the examined revision. External sidecars stay explicit local context.
pub fn for_context_at(
    context: &Context,
    root: &Path,
    revision: &str,
) -> Result<(EffectiveSettings, Option<Ontology>)> {
    let mut historical_context = context.clone();
    if let Some(path) = &context.config_path {
        match historical(root, path, revision)? {
            HistoricalFile::Bytes(bytes) => {
                let text =
                    String::from_utf8(bytes).map_err(|e| OkfError::Environment(e.to_string()))?;
                historical_context.config = toml::from_str(&text)
                    .map_err(|e| OkfError::Usage(format!("invalid historical config: {e}")))?;
                historical_context
                    .config
                    .validate(path.parent().unwrap_or(root))?;
            }
            HistoricalFile::Missing => {
                return Err(OkfError::Environment(format!(
                    "historical configuration unavailable: {} at {revision}",
                    path.display()
                )))
            }
            HistoricalFile::External => {}
        }
    }
    // Determine paths without consulting working-tree ontology existence or bytes.
    let abs = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let id = context
        .catalog
        .as_ref()
        .and_then(|c| c.entries.values().find(|e| e.root == abs))
        .map(|e| e.id.clone());
    let table = historical_context
        .config
        .settings
        .get("bundle_settings")
        .and_then(toml::Value::as_table);
    let settings: BundleSettings = match table.and_then(|t| {
        if id.is_none() && context.catalog.is_some() {
            None
        } else {
            t.get(id.as_deref().unwrap_or("default"))
        }
    }) {
        Some(v) => v
            .clone()
            .try_into()
            .map_err(|e| OkfError::Usage(format!("invalid historical settings: {e}")))?,
        None => BundleSettings::default(),
    };
    validate_settings(&settings)?;
    let configured = settings.ontology.is_some();
    let base = context
        .config_path
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or(root);
    let path = settings
        .ontology
        .as_ref()
        .map(|p| base.join(p))
        .unwrap_or_else(|| load::ontology_path(root));
    let bytes = match historical(root, &path, revision)? {
        HistoricalFile::Bytes(bytes) => Some(bytes),
        HistoricalFile::External => {
            if configured {
                Some(std::fs::read(&path)?)
            } else {
                None
            }
        }
        HistoricalFile::Missing => {
            if configured {
                return Err(OkfError::Environment(format!(
                    "configured historical ontology unavailable: {} at {revision}",
                    path.display()
                )));
            } else {
                None
            }
        }
    };
    let ontology = bytes
        .as_ref()
        .map(|b| {
            std::str::from_utf8(b)
                .map_err(|e| OkfError::Environment(e.to_string()))
                .and_then(load::parse_ontology)
                .map_err(|error| error.at_path(&path))
        })
        .transpose()?;
    Ok((
        EffectiveSettings {
            bundle: id,
            ontology_path: bytes.as_ref().map(|_| path),
            ontology_digest: bytes
                .as_ref()
                .map(|b| crate::fingerprint::canonicalize::sha256_hex(b)),
            settings,
        },
        ontology,
    ))
}
pub fn load_for_at(root: &Path, version: &str) -> Result<(EffectiveSettings, Option<Ontology>)> {
    let context = current_context(Some(&root.to_string_lossy()), None)?;
    if let Some(revision) = version.strip_prefix("git:") {
        for_context_at(&context, root, revision)
    } else {
        load_for(root)
    }
}
