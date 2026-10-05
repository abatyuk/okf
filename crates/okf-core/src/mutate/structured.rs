//! Structured frontmatter assignments, object paths and RFC 6902 patches.
use serde::Deserialize;
use serde_yaml::{Mapping, Value};

use crate::error::{OkfError, Result};
use crate::model::frontmatter::Frontmatter;

#[derive(Debug, Clone, Default)]
pub struct StructuredEdits {
    /// Replace complete, literal top-level fields with typed YAML values.
    pub sets: Vec<(String, Value)>,
    /// Set object paths, creating absent intermediate objects.
    pub set_paths: Vec<(String, Value)>,
    /// Delete object paths; absent keys are no-ops.
    pub unset_paths: Vec<String>,
    /// Sequential RFC 6902 operations on the frontmatter root.
    pub patch: Vec<PatchOperation>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum PatchOperation {
    Add { path: String, value: Value },
    Remove { path: String },
    Replace { path: String, value: Value },
    Test { path: String, value: Value },
    Move { from: String, path: String },
    Copy { from: String, path: String },
}

fn usage(message: impl Into<String>) -> OkfError {
    OkfError::Usage(message.into())
}

/// Dotted object keys and JSON-quoted bracket keys, e.g. `a.b` or `a["b.c"]`.
/// Array indices, wildcards and unquoted brackets deliberately are not write paths.
pub fn object_path(raw: &str) -> Result<Vec<String>> {
    let mut rest = raw;
    let mut keys = Vec::new();
    let mut need_segment = true;
    while !rest.is_empty() {
        if rest.starts_with('[') {
            let quoted = &rest[1..];
            if !quoted.starts_with('"') {
                return Err(usage(format!(
                    "invalid object path {raw:?}: brackets require a JSON-quoted key"
                )));
            }
            let mut values = serde_json::Deserializer::from_str(quoted).into_iter::<String>();
            let key = values
                .next()
                .transpose()
                .map_err(|e| usage(format!("invalid object path {raw:?}: {e}")))?
                .ok_or_else(|| usage("missing bracket key"))?;
            let consumed = values.byte_offset();
            rest = &quoted[consumed..];
            if !rest.starts_with(']') {
                return Err(usage(format!(
                    "invalid object path {raw:?}: missing closing bracket"
                )));
            }
            rest = &rest[1..];
            keys.push(key);
        } else {
            if !need_segment {
                return Err(usage(format!(
                    "invalid object path {raw:?}: expected dot or bracket"
                )));
            }
            let end = rest.find(['.', '[', ']']).unwrap_or(rest.len());
            if end == 0 {
                return Err(usage(format!(
                    "invalid object path {raw:?}: empty or malformed segment"
                )));
            }
            let segment = &rest[..end];
            if !segment
                .bytes()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
                || !segment
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
            {
                return Err(usage(format!("invalid object path {raw:?}: use bracket-quoted keys for punctuation or numeric keys")));
            }
            keys.push(segment.to_string());
            rest = &rest[end..];
        }
        need_segment = false;
        if rest.starts_with('.') {
            rest = &rest[1..];
            need_segment = true;
            if rest.is_empty() || rest.starts_with(['.', '[', ']']) {
                return Err(usage(format!("invalid object path {raw:?}: empty segment")));
            }
        } else if !rest.is_empty() && !rest.starts_with('[') {
            return Err(usage(format!(
                "invalid object path {raw:?}: expected dot or bracket"
            )));
        }
    }
    if keys.len() > 128 {
        return Err(usage("object path exceeds nesting limit (128)"));
    }
    if keys.is_empty() || keys.iter().any(|k| k.trim().is_empty()) {
        return Err(usage(format!("invalid object path {raw:?}: empty key")));
    }
    Ok(keys)
}

/// Split a path assignment at an equals sign outside JSON-quoted bracket keys.
pub fn parse_path_assignment(raw: &str) -> Result<(String, String)> {
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in raw.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && c == '\\' {
            escaped = true;
            continue;
        }
        if c == '"' {
            quoted = !quoted;
        }
        if c == '=' && !quoted {
            let path = raw[..i].trim();
            object_path(path)?;
            return Ok((path.to_string(), raw[i + 1..].to_string()));
        }
    }
    Err(usage("invalid --set-path: expected path=value"))
}

/// Decode a strict JSON Pointer. Empty points at the entire frontmatter map.
pub fn pointer(raw: &str) -> Result<Vec<String>> {
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    let raw = raw
        .strip_prefix('/')
        .ok_or_else(|| usage("patch paths must be JSON Pointers (empty or beginning with /)"))?;
    raw.split('/')
        .map(|token| {
            let mut result = String::new();
            let mut chars = token.chars();
            while let Some(c) = chars.next() {
                if c == '~' {
                    result.push(match chars.next() {
                        Some('0') => '~',
                        Some('1') => '/',
                        _ => return Err(usage("invalid JSON Pointer escape: expected ~0 or ~1")),
                    });
                } else {
                    result.push(c);
                }
            }
            Ok(result)
        })
        .collect()
}

fn overlaps(left: &[String], right: &[String]) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

impl StructuredEdits {
    pub fn is_empty(&self) -> bool {
        self.sets.is_empty()
            && self.set_paths.is_empty()
            && self.unset_paths.is_empty()
            && self.patch.is_empty()
    }

    /// Reject ambiguous combinations while retaining legacy flag precedence. Patch operations
    /// may overlap each other, since their order is explicitly specified by the patch document.
    pub fn validate_conflicts(&self, legacy_keys: &[String]) -> Result<()> {
        let mut paths: Vec<Vec<String>> = Vec::new();
        for (key, _) in &self.sets {
            if key.trim().is_empty() {
                return Err(usage("--set-yaml requires a non-empty key"));
            }
            paths.push(vec![key.clone()]);
        }
        for (path, _) in &self.set_paths {
            paths.push(object_path(path)?);
        }
        for path in &self.unset_paths {
            paths.push(object_path(path)?);
        }
        for (i, path) in paths.iter().enumerate() {
            if paths[..i].iter().any(|other| overlaps(path, other)) {
                return Err(usage(
                    "overlapping structured assignments; assign each path once",
                ));
            }
            if legacy_keys
                .iter()
                .any(|key| overlaps(path, std::slice::from_ref(key)))
            {
                return Err(usage(
                    "structured assignment conflicts with a legacy operation on the same field",
                ));
            }
        }
        for operation in &self.patch {
            for raw in operation.paths() {
                let path = pointer(raw)?;
                if paths.iter().any(|other| overlaps(&path, other))
                    || legacy_keys
                        .iter()
                        .any(|key| overlaps(&path, std::slice::from_ref(key)))
                {
                    return Err(usage("patch conflicts with another frontmatter operation"));
                }
            }
        }
        Ok(())
    }

    /// Work on a clone: even core callers retain the original map if any operation fails.
    pub fn apply(&self, frontmatter: &mut Frontmatter) -> Result<()> {
        let mut root = Value::Mapping(
            frontmatter
                .map
                .iter()
                .map(|(k, v)| (Value::String(k.clone()), v.clone()))
                .collect(),
        );
        for (key, value) in &self.sets {
            root.as_mapping_mut()
                .unwrap()
                .insert(Value::String(key.clone()), value.clone());
        }
        for (path, value) in &self.set_paths {
            assign_object(&mut root, &object_path(path)?, Some(value.clone()))?;
        }
        for path in &self.unset_paths {
            assign_object(&mut root, &object_path(path)?, None)?;
        }
        for operation in &self.patch {
            operation.apply(&mut root)?;
        }
        let map = root
            .as_mapping()
            .ok_or_else(|| usage("frontmatter root must remain a mapping"))?;
        let mut result = indexmap::IndexMap::new();
        for (key, value) in map {
            let key = key
                .as_str()
                .filter(|key| !key.trim().is_empty())
                .ok_or_else(|| usage("frontmatter keys must be non-empty strings"))?;
            result.insert(key.to_string(), value.clone());
        }
        frontmatter.map = result;
        Ok(())
    }
}

fn assign_object(node: &mut Value, keys: &[String], value: Option<Value>) -> Result<()> {
    let map = node
        .as_mapping_mut()
        .ok_or_else(|| usage("object path traverses a scalar or list"))?;
    let key = Value::String(keys[0].clone());
    if keys.len() == 1 {
        match value {
            Some(value) => {
                map.insert(key, value);
            }
            None => {
                map.remove(&key);
            }
        }
        return Ok(());
    }
    if !map.contains_key(&key) {
        if value.is_none() {
            return Ok(());
        }
        map.insert(key.clone(), Value::Mapping(Mapping::new()));
    }
    assign_object(map.get_mut(&key).unwrap(), &keys[1..], value)
}

fn index(token: &str, len: usize, insert: bool) -> Result<usize> {
    if insert && token == "-" {
        return Ok(len);
    }
    if token.is_empty()
        || (token.len() > 1 && token.starts_with('0'))
        || !token.bytes().all(|c| c.is_ascii_digit())
    {
        return Err(usage(format!("invalid patch array index {token:?}")));
    }
    let i = token
        .parse::<usize>()
        .map_err(|_| usage("patch array index is too large"))?;
    if i > len || (!insert && i == len) {
        return Err(usage("patch array index out of bounds"));
    }
    Ok(i)
}

fn lookup<'a>(node: &'a Value, path: &[String]) -> Result<&'a Value> {
    if path.is_empty() {
        return Ok(node);
    }
    let child = match node {
        Value::Mapping(map) => map
            .get(Value::String(path[0].clone()))
            .ok_or_else(|| usage("patch path does not exist"))?,
        Value::Sequence(seq) => &seq[index(&path[0], seq.len(), false)?],
        _ => return Err(usage("patch path traverses a scalar")),
    };
    lookup(child, &path[1..])
}

fn lookup_mut<'a>(node: &'a mut Value, path: &[String]) -> Result<&'a mut Value> {
    if path.is_empty() {
        return Ok(node);
    }
    let child = match node {
        Value::Mapping(map) => map
            .get_mut(Value::String(path[0].clone()))
            .ok_or_else(|| usage("patch path does not exist"))?,
        Value::Sequence(seq) => {
            let i = index(&path[0], seq.len(), false)?;
            &mut seq[i]
        }
        _ => return Err(usage("patch path traverses a scalar")),
    };
    lookup_mut(child, &path[1..])
}

fn add_value(root: &mut Value, path: &[String], value: Value) -> Result<()> {
    let Some((token, parent)) = path.split_last() else {
        *root = value;
        return Ok(());
    };
    match lookup_mut(root, parent)? {
        Value::Mapping(map) => {
            map.insert(Value::String(token.clone()), value);
        }
        Value::Sequence(seq) => {
            let i = index(token, seq.len(), true)?;
            seq.insert(i, value);
        }
        _ => return Err(usage("patch destination parent is a scalar")),
    }
    Ok(())
}

fn remove_value(root: &mut Value, path: &[String]) -> Result<Value> {
    let Some((token, parent)) = path.split_last() else {
        return Ok(std::mem::replace(root, Value::Null));
    };
    match lookup_mut(root, parent)? {
        Value::Mapping(map) => map
            .remove(Value::String(token.clone()))
            .ok_or_else(|| usage("patch removal path does not exist")),
        Value::Sequence(seq) => {
            let i = index(token, seq.len(), false)?;
            Ok(seq.remove(i))
        }
        _ => Err(usage("patch removal parent is a scalar")),
    }
}

/// JSON Patch numeric tests compare mathematical values, including inside objects/lists.
/// Compare integral floats by an exact bounded integer conversion, avoiding rounding a large
/// integer through f64 (which could incorrectly equate adjacent values above 2^53).
fn patch_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
                return a == b;
            }
            if let (Some(a), Some(b)) = (a.as_u64(), b.as_u64()) {
                return a == b;
            }
            if a.is_f64() && b.is_f64() {
                return a.as_f64() == b.as_f64();
            }
            let (float, integer) = if a.is_f64() { (a, b) } else { (b, a) };
            let Some(f) = float.as_f64() else {
                return false;
            };
            if !f.is_finite() || f.trunc() != f {
                return false;
            }
            if let Some(i) = integer.as_i64() {
                (-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&f)
                    && (f as i64) == i
            } else if let Some(i) = integer.as_u64() {
                (0.0..18_446_744_073_709_551_616.0).contains(&f) && (f as u64) == i
            } else {
                false
            }
        }
        (Value::Sequence(a), Value::Sequence(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| patch_equal(a, b))
        }
        (Value::Mapping(a), Value::Mapping(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(key, value)| b.get(key).is_some_and(|other| patch_equal(value, other)))
        }
        _ => left == right,
    }
}

impl PatchOperation {
    fn paths(&self) -> Vec<&str> {
        match self {
            Self::Add { path, .. }
            | Self::Remove { path }
            | Self::Replace { path, .. }
            | Self::Test { path, .. } => vec![path],
            Self::Move { from, path } | Self::Copy { from, path } => vec![from, path],
        }
    }

    fn apply(&self, root: &mut Value) -> Result<()> {
        match self {
            Self::Add { path, value } => add_value(root, &pointer(path)?, value.clone()),
            Self::Remove { path } => {
                remove_value(root, &pointer(path)?)?;
                Ok(())
            }
            Self::Replace { path, value } => {
                *lookup_mut(root, &pointer(path)?)? = value.clone();
                Ok(())
            }
            Self::Test { path, value } => {
                if !patch_equal(lookup(root, &pointer(path)?)?, value) {
                    return Err(usage(format!("patch test failed at {path:?}")));
                }
                Ok(())
            }
            Self::Copy { from, path } => {
                let value = lookup(root, &pointer(from)?)?.clone();
                add_value(root, &pointer(path)?, value)
            }
            Self::Move { from, path } => {
                let from = pointer(from)?;
                let path = pointer(path)?;
                if path.len() > from.len() && path.starts_with(&from) {
                    return Err(usage("cannot move a value into its descendant"));
                }
                // Looking up first also validates a move from a path onto itself.
                lookup(root, &from)?;
                if from == path {
                    return Ok(());
                }
                let value = remove_value(root, &from)?;
                add_value(root, &path, value)
            }
        }
    }
}
