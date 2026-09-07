//! NDJSON serializer (one JSON object per line).
use crate::error::{OkfError, Result};
use serde_json::Value as Json;

/// Serialize a single JSON value to one NDJSON line (no trailing newline).
pub fn to_line(value: &Json) -> Result<String> {
    serde_json::to_string(value).map_err(|e| OkfError::Internal(e.to_string()))
}

/// Serialize a sequence of JSON values to NDJSON text, one object per line, each
/// terminated by `\n`.
pub fn to_ndjson<'a, I>(values: I) -> Result<String>
where
    I: IntoIterator<Item = &'a Json>,
{
    let mut out = String::new();
    for v in values {
        out.push_str(&to_line(v)?);
        out.push('\n');
    }
    Ok(out)
}
