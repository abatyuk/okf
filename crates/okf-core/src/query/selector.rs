//! Small, non-executable selectors over authored YAML. Explicit list traversal preserves paths.
use crate::error::{OkfError, Result};
use serde_yaml::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Key(String),
    Each,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector {
    pub source: String,
    pub steps: Vec<Step>,
}
#[derive(Debug, Clone)]
pub struct Occurrence<'a> {
    pub value: &'a Value,
    pub path: String,
    pub indices: Vec<usize>,
}
#[derive(Debug, Default)]
pub struct Selection<'a> {
    pub leaves: Vec<Occurrence<'a>>,
    pub empty_lists: usize,
}

impl Selector {
    pub fn parse(raw: &str) -> Result<Self> {
        let fail = || {
            OkfError::Usage(format!(
                "invalid selector {raw:?}: use property, .property, [], or [\"literal.key\"]"
            ))
        };
        if raw.is_empty() || raw.starts_with('$') {
            return Err(fail());
        }
        let mut steps = Vec::new();
        let mut at = 0;
        let bytes = raw.as_bytes();
        while at < raw.len() {
            if bytes[at] == b'[' {
                if raw[at..].starts_with("[]") {
                    if steps.is_empty() {
                        return Err(fail());
                    }
                    steps.push(Step::Each);
                    at += 2;
                } else if raw[at..].starts_with("[\"") {
                    let begin = at + 1;
                    let mut end = begin + 1;
                    let mut escaped = false;
                    while end < bytes.len() {
                        if bytes[end] == b'"' && !escaped {
                            break;
                        }
                        if bytes[end] == b'\\' {
                            escaped = !escaped;
                        } else {
                            escaped = false;
                        }
                        end += 1;
                    }
                    if end + 1 >= bytes.len() || bytes[end + 1] != b']' {
                        return Err(fail());
                    }
                    let key: String =
                        serde_json::from_str(&raw[begin..=end]).map_err(|_| fail())?;
                    steps.push(Step::Key(key));
                    at = end + 2;
                } else {
                    return Err(fail());
                }
            } else {
                let begin = at;
                while at < bytes.len() && !matches!(bytes[at], b'.' | b'[') {
                    at += 1;
                }
                let key = &raw[begin..at];
                if key.is_empty()
                    || key.chars().any(|c| {
                        c.is_whitespace() || matches!(c, ']' | '*' | '$' | '|' | '(' | ')')
                    })
                {
                    return Err(fail());
                }
                steps.push(Step::Key(key.into()));
            }
            if at < bytes.len() && bytes[at] == b'.' {
                at += 1;
                if at == bytes.len() || matches!(bytes[at], b'.' | b'[') {
                    return Err(fail());
                }
            } else if at < bytes.len() && bytes[at] != b'[' {
                return Err(fail());
            }
        }
        Ok(Self {
            source: raw.into(),
            steps,
        })
    }
    pub fn select<'a>(&self, value: &'a Value) -> Selection<'a> {
        fn walk<'a>(
            steps: &[Step],
            value: &'a Value,
            path: String,
            indices: Vec<usize>,
            out: &mut Selection<'a>,
        ) {
            match steps.split_first() {
                None => out.leaves.push(Occurrence {
                    value,
                    path,
                    indices,
                }),
                Some((Step::Key(key), rest)) => {
                    if let Some(child) = value
                        .as_mapping()
                        .and_then(|m| m.get(Value::String(key.clone())))
                    {
                        let part = if key.contains('.') {
                            format!("[{}]", serde_json::to_string(key).unwrap())
                        } else if path.is_empty() {
                            key.clone()
                        } else {
                            format!(".{key}")
                        };
                        walk(rest, child, format!("{path}{part}"), indices, out);
                    }
                }
                Some((Step::Each, rest)) => {
                    if let Some(items) = value.as_sequence() {
                        if items.is_empty() && rest.is_empty() {
                            out.empty_lists += 1;
                        }
                        for (i, item) in items.iter().enumerate() {
                            let mut ix = indices.clone();
                            ix.push(i);
                            walk(rest, item, format!("{path}[{i}]"), ix, out);
                        }
                    }
                }
            }
        }
        let mut out = Selection::default();
        walk(&self.steps, value, String::new(), Vec::new(), &mut out);
        out
    }
    pub fn list_shape(&self) -> Vec<Vec<Step>> {
        self.steps
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s, Step::Each))
            .map(|(i, _)| self.steps[..i].to_vec())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_traversal_and_literal_keys() {
        let v:Value=serde_yaml::from_str("norms: [{bearer: x, 'action.name': y}, {bearer: z}]\n'policy.status': active\ntags: []").unwrap();
        let s = Selector::parse("norms[][\"action.name\"]")
            .unwrap()
            .select(&v);
        assert_eq!(s.leaves.len(), 1);
        assert_eq!(s.leaves[0].indices, vec![0]);
        assert_eq!(
            Selector::parse("norms.bearer")
                .unwrap()
                .select(&v)
                .leaves
                .len(),
            0
        );
        assert_eq!(Selector::parse("tags[]").unwrap().select(&v).empty_lists, 1);
        for bad in ["$x", "x[*]", "x[0]", "x..y", "x.", "x.[\"y\"]"] {
            assert!(Selector::parse(bad).is_err(), "{bad}");
        }
    }
}
