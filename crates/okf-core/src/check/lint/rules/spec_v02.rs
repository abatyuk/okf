//! Advisory validation of optional/recommended OKF v0.2 families.

use std::collections::HashSet;

use regex::Regex;
use serde_yaml::Value;

use crate::check::lint::{Finding, RuleContext};
use crate::model::standard::{parse_timestamp, valid_actor};

pub const RULE: &str = "okf-v02";

pub fn run(ctx: &RuleContext) -> Vec<Finding> {
    let mut out = Vec::new();
    let footnote = Regex::new(r"\[\^([^\]]+)\]").unwrap();
    for concept in &ctx.bundle.concepts {
        let fm = &concept.frontmatter;
        let mut add = |message: String| {
            out.push(Finding {
                rule: RULE.to_string(),
                severity: ctx.config.spec_v02,
                concept: Some(concept.id.0.clone()),
                message,
            })
        };

        if fm.get("tags").is_some_and(|v| !v.is_sequence()) {
            add("`tags` should be a YAML list".to_string());
        }
        if concept
            .description()
            .is_some_and(|d| d.contains(['\n', '\r']))
        {
            add("`description` should be one line".to_string());
        }
        if let Some(status) = fm.get_str("status") {
            if !matches!(status, "draft" | "stable" | "deprecated") {
                add(format!(
                    "invalid status {status:?}: expected draft, stable, or deprecated"
                ));
            }
        }
        check_timestamp(fm.get("stale_after"), "stale_after", &mut add);
        if let Some(generated) = fm.get("generated") {
            match generated.as_mapping() {
                Some(map) => {
                    match map.get("by").and_then(Value::as_str) {
                        Some(by) if valid_actor(by) => {}
                        _ => add("`generated.by` must use the OKF actor convention".to_string()),
                    }
                    check_timestamp(map.get("at"), "generated.at", &mut add);
                }
                None => add("`generated` must be a mapping".to_string()),
            }
        }
        if let Some(verified) = fm.get("verified") {
            let entries: Vec<&Value> = match verified {
                Value::Sequence(seq) => seq.iter().collect(),
                Value::Mapping(_) => vec![verified],
                _ => {
                    add("`verified` must be a mapping or list of mappings".to_string());
                    Vec::new()
                }
            };
            for (i, entry) in entries.into_iter().enumerate() {
                let Some(map) = entry.as_mapping() else {
                    add(format!("verified[{i}] must be a mapping"));
                    continue;
                };
                match map.get("by").and_then(Value::as_str) {
                    Some(by) if valid_actor(by) => {}
                    _ => add(format!(
                        "verified[{i}].by must use the OKF actor convention"
                    )),
                }
                check_required_timestamp(map.get("at"), &format!("verified[{i}].at"), &mut add);
            }
        }

        let mut source_ids = HashSet::new();
        let mut any_usage_count = false;
        if let Some(sources) = fm.get("sources") {
            match sources.as_sequence() {
                Some(entries) => {
                    for (i, entry) in entries.iter().enumerate() {
                        let Some(map) = entry.as_mapping() else {
                            add(format!("sources[{i}] must be a mapping"));
                            continue;
                        };
                        if map
                            .get("resource")
                            .and_then(Value::as_str)
                            .is_none_or(|s| s.trim().is_empty())
                        {
                            add(format!("sources[{i}].resource is required"));
                        }
                        if let Some(id) = map.get("id").and_then(Value::as_str) {
                            if !source_ids.insert(id.to_string()) {
                                add(format!("duplicate sources[].id {id:?}"));
                            }
                        }
                        check_timestamp(
                            map.get("last_modified"),
                            &format!("sources[{i}].last_modified"),
                            &mut add,
                        );
                        if map.get("usage_count").is_some() {
                            any_usage_count = true;
                            if !map.get("usage_count").is_some_and(|v| v.as_u64().is_some()) {
                                add(format!(
                                    "sources[{i}].usage_count must be a non-negative integer"
                                ));
                            }
                        }
                        if let Some(window) = map.get("usage_window") {
                            check_window(window, &format!("sources[{i}].usage_window"), &mut add);
                        }
                    }
                }
                None => add("`sources` must be a list".to_string()),
            }
        }
        if let Some(window) = fm.get("usage_window") {
            check_window(window, "usage_window", &mut add);
        } else if any_usage_count {
            add("usage_count should be framed by a shared or per-source usage_window".to_string());
        }
        for capture in footnote.captures_iter(&concept.body) {
            let id = capture.get(1).unwrap().as_str();
            if !source_ids.contains(id) {
                add(format!("footnote [^{id}] has no matching sources[].id"));
            }
        }

        if concept.concept_type() == Some("Attested Computation") {
            if fm.get_str("runtime").is_none_or(|s| s.trim().is_empty()) {
                add("Attested Computation requires non-empty `runtime`".to_string());
            }
            if fm.get("parameters").is_some_and(|v| !v.is_sequence()) {
                add("Attested Computation `parameters` must be a list".to_string());
            }
            if let Some(parameters) = fm.get("parameters").and_then(Value::as_sequence) {
                let mut names = HashSet::new();
                for (i, parameter) in parameters.iter().enumerate() {
                    let valid = parameter
                        .get("name")
                        .and_then(Value::as_str)
                        .is_some_and(|s| !s.is_empty())
                        && parameter
                            .get("type")
                            .and_then(Value::as_str)
                            .is_some_and(|s| !s.is_empty())
                        && parameter.get("required").and_then(Value::as_bool).is_some();
                    if !valid {
                        add(format!(
                            "parameters[{i}] must contain name, type, and boolean required"
                        ));
                    }
                    if let Some(name) = parameter.get("name").and_then(Value::as_str) {
                        if !names.insert(name.to_string()) {
                            add(format!("duplicate computation parameter {name:?}"));
                        }
                    }
                }
            }
            for family in ["executor", "attester"] {
                if let Some(value) = fm.get(family) {
                    if !value.is_mapping() {
                        add(format!("`{family}` must be a mapping"));
                    } else if value
                        .get("resource")
                        .and_then(Value::as_str)
                        .is_none_or(|s| s.trim().is_empty())
                    {
                        add(format!(
                            "`{family}.resource` must be a non-empty path or URI"
                        ));
                    }
                }
            }
            let (fence_count, fence_has_content) = computation_fences(&concept.body);
            let inline = fence_count > 0;
            let file = fm
                .get_str("computation")
                .is_some_and(|s| !s.trim().is_empty());
            if inline == file {
                add("Attested Computation must use exactly one of inline # Computation fence or `computation` path".to_string());
            }
            if fence_count > 1 {
                add("inline # Computation must contain a single fenced code block".to_string());
            }
            if inline && !fence_has_content {
                add("inline computation fence is empty".to_string());
            }
        }
    }
    out
}

fn check_timestamp<F>(value: Option<&Value>, name: &str, add: &mut F)
where
    F: FnMut(String),
{
    if let Some(value) = value {
        match value.as_str() {
            Some(raw) if parse_timestamp(raw).is_some() => {}
            _ => add(format!(
                "`{name}` must be an ISO 8601 datetime with explicit UTC offset"
            )),
        }
    }
}

fn check_required_timestamp<F>(value: Option<&Value>, name: &str, add: &mut F)
where
    F: FnMut(String),
{
    if value.is_none() {
        add(format!("`{name}` is required"));
    } else {
        check_timestamp(value, name, add);
    }
}

fn check_window<F>(value: &Value, name: &str, add: &mut F)
where
    F: FnMut(String),
{
    match value.as_mapping() {
        Some(map) => {
            check_required_timestamp(map.get("from"), &format!("{name}.from"), add);
            check_required_timestamp(map.get("to"), &format!("{name}.to"), add);
            if let (Some(from), Some(to)) = (
                map.get("from")
                    .and_then(Value::as_str)
                    .and_then(parse_timestamp),
                map.get("to")
                    .and_then(Value::as_str)
                    .and_then(parse_timestamp),
            ) {
                if from > to {
                    add(format!("`{name}.from` must not be after `{name}.to`"));
                }
            }
        }
        None => add(format!("`{name}` must be a {{from,to}} mapping")),
    }
}

fn computation_fences(body: &str) -> (usize, bool) {
    let mut under = false;
    let mut in_fence = false;
    let mut count = 0;
    let mut has_content = false;
    for line in body.lines() {
        let t = line.trim();
        if !in_fence && t.starts_with("# ") {
            under = t.trim_start_matches("# ").trim() == "Computation";
        } else if under && (t.starts_with("```") || t.starts_with("~~~")) {
            if !in_fence {
                count += 1;
            }
            in_fence = !in_fence;
        } else if under && in_fence && !t.is_empty() {
            has_content = true;
        }
    }
    (count, has_content)
}
