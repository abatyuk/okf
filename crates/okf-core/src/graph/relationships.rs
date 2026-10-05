//! Authored semantic edge occurrences; never inferred from arbitrary path-looking strings.
use crate::{
    error::{OkfError, Result},
    model::{
        concept::Concept,
        link::{classify, resolve_link, LinkKind},
    },
    ontology::schema::Ontology,
    query::selector::{Occurrence, Selector},
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub source: String,
    pub rule: String,
    pub field_path: String,
    pub raw_reference: String,
    pub target: String,
    pub authored_kind: Option<String>,
    pub inverse: Option<String>,
    pub attributes: IndexMap<String, Value>,
    pub status: String,
}
fn literal_selector(key: &str) -> Result<Selector> {
    Selector::parse(&format!("[{}]", serde_json::to_string(key).unwrap()))
}
pub fn reference_selector(key: &str, selector: Option<&str>) -> Result<Selector> {
    match selector {
        Some(s) => Selector::parse(s),
        None => literal_selector(key),
    }
}
pub fn validate_relationships(ontology: &Ontology) -> Result<()> {
    for (name, ct) in &ontology.concepts {
        for (key, reference) in &ct.references {
            reference_selector(key, reference.selector.as_deref())?;
        }
        for (name_rule, rule) in &ct.relationships {
            let error = |why: &str| {
                OkfError::Usage(format!(
                    "concept {name:?} relationship {name_rule:?}: {why}"
                ))
            };
            let reference = ct
                .references
                .get(&rule.reference)
                .ok_or_else(|| error("unknown reference rule"))?;
            if rule.kind.is_some() == rule.kind_selector.is_some() {
                return Err(error("set exactly one of kind or kind_selector"));
            }
            if rule.kind.is_some()
                && (rule.inverse.as_deref() == Some("") || !rule.kinds.is_empty())
            {
                return Err(error(
                    "fixed kind permits optional nonempty inverse and no kinds map",
                ));
            }
            if rule.kind_selector.is_some() && (rule.kinds.is_empty() || rule.inverse.is_some()) {
                return Err(error("dynamic kind requires kinds and no fixed inverse"));
            }
            if rule.kinds.values().any(|k| k.inverse.is_empty()) {
                return Err(error("inverse names must be nonempty"));
            }
            for mapped in rule.kinds.values() {
                if let Some(targets) = &mapped.target {
                    if targets.types().iter().any(|t| {
                        !reference.target.types().iter().any(|parent| {
                            crate::ontology::field_types::concept_is_a(ontology, t, parent)
                        })
                    }) {
                        return Err(error(
                            "kind-specific targets must narrow reference target types",
                        ));
                    }
                }
            }
            if let Some(kind_selector) = &rule.kind_selector {
                let selector = Selector::parse(kind_selector)?;
                let mut resolved = None;
                for step in &selector.steps {
                    resolved = match (step, resolved) {
                        (crate::query::selector::Step::Key(key), None) => ct
                            .fields
                            .get(key)
                            .map(|f| crate::ontology::field_types::resolve_field(ontology, f))
                            .transpose()?
                            .map(|f| f.ty),
                        (crate::query::selector::Step::Key(key), Some(ty)) => {
                            ty.fields.get(key).map(|f| f.ty.clone())
                        }
                        (crate::query::selector::Step::Each, Some(ty)) => ty.item.map(|i| *i),
                        _ => None,
                    };
                }
                if let Some(ty) = resolved {
                    if ty.base != crate::ontology::schema::FieldType::Enum {
                        return Err(error("dynamic kind selector requires enum field"));
                    }
                    if ty.values.iter().any(|v| !rule.kinds.contains_key(v))
                        || rule.kinds.keys().any(|v| !ty.values.contains(v))
                    {
                        return Err(error(
                            "dynamic kinds must agree with declared enum vocabulary",
                        ));
                    }
                }
            }
            let target = reference_selector(&rule.reference, reference.selector.as_deref())?;
            for selector in rule.kind_selector.iter().chain(rule.attributes.values()) {
                let s = Selector::parse(selector)?;
                if s.list_shape() != target.list_shape() {
                    return Err(error(
                        "selector list anchors do not match target occurrence",
                    ));
                }
            }
        }
    }
    Ok(())
}
fn paired<'a>(selected: &'a [Occurrence<'a>], indices: &[usize]) -> Option<&'a Value> {
    selected
        .iter()
        .find(|o| o.indices == indices)
        .map(|o| o.value)
}
pub fn relationships(concept: &Concept, ontology: &Ontology) -> Vec<Relationship> {
    let Some(ct) = concept
        .concept_type()
        .and_then(|t| ontology.concepts.get(t))
    else {
        return vec![];
    };
    let root = serde_yaml::to_value(&concept.frontmatter.map).unwrap_or_default();
    let mut out = Vec::new();
    for (name, rule) in &ct.relationships {
        let Some(reference) = ct.references.get(&rule.reference) else {
            continue;
        };
        let Ok(target_selector) =
            reference_selector(&rule.reference, reference.selector.as_deref())
        else {
            continue;
        };
        let selected = target_selector.select(&root);
        let kinds = rule
            .kind_selector
            .as_deref()
            .and_then(|s| Selector::parse(s).ok())
            .map(|s| s.select(&root));
        let attrs: Vec<_> = rule
            .attributes
            .iter()
            .filter_map(|(k, s)| Selector::parse(s).ok().map(|s| (k, s.select(&root))))
            .collect();
        for occurrence in selected.leaves {
            let targets: Vec<(&Value, String, Vec<usize>)> = if reference.selector.is_none() {
                if let Some(seq) = occurrence.value.as_sequence() {
                    seq.iter()
                        .enumerate()
                        .map(|(i, v)| (v, format!("{}[{i}]", occurrence.path), vec![i]))
                        .collect()
                } else {
                    vec![(
                        occurrence.value,
                        occurrence.path.clone(),
                        occurrence.indices.clone(),
                    )]
                }
            } else {
                vec![(
                    occurrence.value,
                    occurrence.path.clone(),
                    occurrence.indices.clone(),
                )]
            };
            for (value, path, indices) in targets {
                let Some(raw) = value.as_str().filter(|s| !s.trim().is_empty()) else {
                    continue;
                };
                let kind = rule.kind.clone().or_else(|| {
                    kinds
                        .as_ref()
                        .and_then(|s| paired(&s.leaves, &indices))
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                });
                let inverse = rule.inverse.clone().or_else(|| {
                    kind.as_ref()
                        .and_then(|k| rule.kinds.get(k))
                        .map(|k| k.inverse.clone())
                });
                let attributes = attrs
                    .iter()
                    .filter_map(|(name, s)| {
                        paired(&s.leaves, &indices).map(|v| ((*name).clone(), v.clone()))
                    })
                    .collect();
                out.push(Relationship {
                    source: concept.id.0.clone(),
                    rule: name.clone(),
                    field_path: path,
                    raw_reference: raw.into(),
                    target: resolve_link(&concept.id, raw).0,
                    authored_kind: kind,
                    inverse: inverse.clone(),
                    attributes,
                    status: if classify(raw) == LinkKind::External {
                        "external"
                    } else if rule.kind_selector.is_some() && inverse.is_none() {
                        "unknown-kind"
                    } else {
                        "unresolved"
                    }
                    .into(),
                });
            }
        }
    }
    out
}
