//! Typed selector filters, projections and document-count facets over YAML metadata.
use crate::{
    error::{OkfError, Result},
    model::concept::Concept,
    query::selector::Selector,
};
use serde_json::{json, Value};
use serde_yaml::Value as Yaml;
use std::collections::BTreeMap;
#[derive(Debug, Clone)]
pub enum Operator {
    Eq(Value),
    In(Vec<Value>),
    NotIn(Vec<Value>),
}
#[derive(Debug, Clone)]
pub struct Condition {
    pub selector: Selector,
    pub operator: Operator,
    pub source: String,
}
fn scalar(v: &Value) -> bool {
    matches!(v, Value::String(_) | Value::Bool(_) | Value::Number(_))
}
impl Condition {
    pub fn parse(raw: &str) -> Result<Self> {
        // Operators occur after a complete selector; bracket strings may contain operator text.
        let mut quoted = false;
        let mut escaped = false;
        let mut split = None;
        for (i, c) in raw.char_indices() {
            if c == '"' && !escaped {
                quoted = !quoted;
            }
            if !quoted {
                if raw[i..].starts_with(" not in ") {
                    split = Some((i, 8, "not in"));
                    break;
                }
                if raw[i..].starts_with(" in ") {
                    split = Some((i, 4, "in"));
                    break;
                }
                if c == '=' {
                    split = Some((i, 1, "="));
                    break;
                }
            }
            if c == '\\' {
                escaped = !escaped
            } else {
                escaped = false
            }
        }
        let (at,n,op)=split.ok_or_else(||OkfError::Usage(format!("invalid facet filter {raw:?}: expected selector=JSON-scalar or selector [not] in [scalars]")))?;
        let selector = Selector::parse(raw[..at].trim())?;
        let operand: Value = serde_json::from_str(raw[at + n..].trim())
            .map_err(|e| OkfError::Usage(format!("invalid filter operand: {e}")))?;
        let operator = if op == "=" {
            if !scalar(&operand) {
                return Err(OkfError::Usage(
                    "equality requires nonnull JSON scalar".into(),
                ));
            }
            Operator::Eq(operand)
        } else {
            let values = operand
                .as_array()
                .filter(|v| !v.is_empty() && v.iter().all(scalar))
                .ok_or_else(|| {
                    OkfError::Usage(
                        "membership requires nonempty JSON array of nonnull scalars".into(),
                    )
                })?
                .clone();
            if op == "in" {
                Operator::In(values)
            } else {
                Operator::NotIn(values)
            }
        };
        Ok(Self {
            selector,
            operator,
            source: raw.into(),
        })
    }
    pub fn matches(&self, concept: &Concept) -> Result<bool> {
        let root = serde_yaml::to_value(&concept.frontmatter.map).unwrap_or_default();
        let selected = self.selector.select(&root);
        let mut values = Vec::new();
        let mut empty = selected.empty_lists > 0;
        for occurrence in selected.leaves {
            match occurrence.value {
                Yaml::Null => {}
                Yaml::Sequence(items) => {
                    if items.is_empty() {
                        empty = true;
                    }
                    for item in items {
                        if item.is_null() {
                            continue;
                        }
                        let v = serde_json::to_value(item).unwrap_or(Value::Null);
                        if !scalar(&v) {
                            return Err(OkfError::Usage(format!(
                                "filter {} selected nonscalar value at {}",
                                self.source, occurrence.path
                            )));
                        }
                        values.push(v);
                    }
                }
                Yaml::Mapping(_) => {
                    return Err(OkfError::Usage(format!(
                        "filter {} selected object at {}",
                        self.source, occurrence.path
                    )))
                }
                other => {
                    let v = serde_json::to_value(other).unwrap_or(Value::Null);
                    if !scalar(&v) {
                        return Err(OkfError::Usage("unsupported filter value".into()));
                    }
                    values.push(v);
                }
            }
        }
        let operands = match &self.operator {
            Operator::Eq(v) => std::slice::from_ref(v),
            Operator::In(v) | Operator::NotIn(v) => v.as_slice(),
        };
        // Typed comparisons never coerce strings/numbers/booleans. Heterogeneous membership operands are permitted.
        if values.iter().any(|v| {
            !operands
                .iter()
                .any(|o| std::mem::discriminant(v) == std::mem::discriminant(o))
        }) {
            return Err(OkfError::Usage(format!(
                "filter {} has incompatible scalar operand type",
                self.source
            )));
        }
        if values.is_empty() && !empty {
            return Ok(false);
        }
        let found = values.iter().any(|v| operands.contains(v));
        Ok(match self.operator {
            Operator::Eq(_) | Operator::In(_) => found,
            Operator::NotIn(_) => !found,
        })
    }
}
/// Missing is represented separately from authored null; computed identity never overwrites metadata.
pub fn project(
    concept: &Concept,
    fields: &[String],
    bundle: Option<&str>,
    version: &str,
) -> Result<Value> {
    let root = serde_yaml::to_value(&concept.frontmatter.map).unwrap_or_default();
    let mut cells = Vec::new();
    for field in fields {
        let cell = if field.starts_with('$') {
            let value = match field.as_str() {
                "$id" => json!(concept.id.0),
                "$bundle" => json!(bundle),
                "$version" => json!(version),
                _ => return Err(OkfError::Usage(format!("unknown computed field {field:?}"))),
            };
            json!({"field":field,"present":true,"value":value})
        } else {
            let selection = Selector::parse(field)?.select(&root);
            let values: Vec<_> = selection
                .leaves
                .iter()
                .map(|o| json!({"path":o.path,"value":o.value}))
                .collect();
            json!({"field":field,"present":!values.is_empty()||selection.empty_lists>0,"occurrences":values,"empty_lists":selection.empty_lists})
        };
        cells.push(cell);
    }
    Ok(
        json!({"kind":"projection","schema_version":1,"id":concept.id.0,"bundle":bundle,"version":version,"fields":cells}),
    )
}
pub fn facet(
    concepts: &[&Concept],
    field: &str,
    threshold: usize,
    include: bool,
    complete: bool,
) -> Result<Value> {
    let selector = Selector::parse(field)?;
    let mut counts: BTreeMap<String, (Value, usize)> = BTreeMap::new();
    let mut invalid = Vec::new();
    for concept in concepts {
        let root = serde_yaml::to_value(&concept.frontmatter.map).unwrap_or_default();
        let selected = selector.select(&root);
        let mut seen = BTreeMap::new();
        for o in selected.leaves {
            if o.value.is_null() {
                continue;
            }
            let value = serde_json::to_value(o.value).unwrap_or(Value::Null);
            if !scalar(&value) {
                invalid.push(
                    json!({"id":concept.id.0,"path":o.path,"reason":"nonscalar-facet-value"}),
                );
                continue;
            }
            let key = format!(
                "{}:{}",
                match value {
                    Value::Bool(_) => "0",
                    Value::Number(_) => "1",
                    _ => "2",
                },
                value
            );
            seen.insert(key, value);
        }
        for (key, value) in seen {
            let entry = counts.entry(key).or_insert((value, 0));
            entry.1 += 1;
        }
    }
    if counts.len() > threshold && !include {
        return Ok(
            json!({"kind":"facet-excluded","schema_version":1,"field":field,"reason":"high-cardinality","threshold":threshold,"distinct_values":counts.len(),"complete":complete}),
        );
    }
    let mut values: Vec<_> = counts
        .into_iter()
        .map(|(key, (value, count))| (key, value, count))
        .collect();
    values.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| typed_scalar_order(&a.1, &b.1))
            .then_with(|| a.0.cmp(&b.0))
    });
    const OUTPUT_LIMIT: usize = 1000;
    let omitted = values.len().saturating_sub(OUTPUT_LIMIT);
    values.truncate(OUTPUT_LIMIT);
    Ok(
        json!({"kind":"facet","schema_version":1,"field":field,"basis":if complete{"all-matches"}else{"observed-matches"},"complete":complete&&invalid.is_empty(),"truncated":omitted>0,"omitted_values":omitted,"values":values.into_iter().map(|(_,value,count)|json!({"value":value,"count":count})).collect::<Vec<_>>(),"diagnostics":invalid}),
    )
}

/// Typed values have a deterministic order; numeric ties use their value, not JSON spelling.
fn typed_scalar_order(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let rank = |value: &Value| match value {
        Value::Bool(_) => 0,
        Value::Number(_) => 1,
        Value::String(_) => 2,
        _ => 3,
    };
    match (a, b) {
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        (Value::String(a), Value::String(b)) => a.cmp(b),
        (Value::Number(a), Value::Number(b)) => {
            if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
                return a.cmp(&b);
            }
            if let (Some(a), Some(b)) = (a.as_u64(), b.as_u64()) {
                return a.cmp(&b);
            }
            if let (Some(a), Some(b)) = (a.as_i64(), b.as_u64()) {
                return if a < 0 {
                    Ordering::Less
                } else {
                    (a as u64).cmp(&b)
                };
            }
            if let (Some(a), Some(b)) = (a.as_u64(), b.as_i64()) {
                return if b < 0 {
                    Ordering::Greater
                } else {
                    a.cmp(&(b as u64))
                };
            }
            a.as_f64()
                .and_then(|a| b.as_f64().and_then(|b| a.partial_cmp(&b)))
                .unwrap_or(Ordering::Equal)
        }
        _ => rank(a).cmp(&rank(b)),
    }
}
