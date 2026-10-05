//! Explicit structured input for mutations. Files contain values, not assignments.
use std::io::Read;
use std::mem::MaybeUninit;

use okf_core::error::{OkfError, Result};
use serde_yaml::Value;

/// Resolve literal text, `@file`, or one stdin input per invocation.
#[derive(Default)]
pub struct InputReader {
    stdin_used: bool,
}

impl InputReader {
    pub fn read(&mut self, input: &str) -> Result<String> {
        if input == "-" {
            if self.stdin_used {
                return Err(OkfError::Usage("only one input may consume stdin".into()));
            }
            self.stdin_used = true;
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(|e| OkfError::Environment(format!("cannot read stdin: {e}")))?;
            Ok(text)
        } else if let Some(path) = input.strip_prefix('@') {
            std::fs::read_to_string(path)
                .map_err(|e| OkfError::Environment(format!("cannot read {path}: {e}")))
        } else {
            Ok(input.to_string())
        }
    }
}

/// A complete unified diff for review, including file creation and missing final newlines.
/// Keeping all lines visible avoids introducing a second dependency for small previews.
pub fn preview_diff(path: &std::path::Path, before: &str, after: &str) -> String {
    if before == after {
        return String::new();
    }
    let old_path = if before.is_empty() {
        "/dev/null".into()
    } else {
        path.display().to_string()
    };
    let old_lines = before.lines().count();
    let new_lines = after.lines().count();
    let mut diff = format!(
        "--- {old_path}\n+++ {}\n@@ -{},{} +{},{} @@\n",
        path.display(),
        usize::from(old_lines != 0),
        old_lines,
        usize::from(new_lines != 0),
        new_lines,
    );
    for (prefix, contents) in [('-', before), ('+', after)] {
        for line in contents.lines() {
            diff.push(prefix);
            diff.push_str(line);
            diff.push('\n');
        }
        if !contents.is_empty() && !contents.ends_with('\n') {
            diff.push_str("\\ No newline at end of file\n");
        }
    }
    diff
}

/// Parse one YAML document with string mapping keys and no tags/anchors/aliases.
/// `serde_yaml::Value` rejects duplicate mapping keys, including nested duplicates.
pub fn parse_yaml(input: &str) -> Result<Value> {
    check_tokens(input)?;
    let value: Value = serde_yaml::from_str(input)
        .map_err(|e| OkfError::Usage(format!("invalid structured YAML: {e}")))?;
    check_value(&value, 0)?;
    Ok(value)
}

fn check_value(value: &Value, depth: usize) -> Result<()> {
    if depth > 128 {
        return Err(OkfError::Usage(
            "structured YAML exceeds nesting limit (128)".into(),
        ));
    }
    match value {
        Value::Mapping(map) => {
            for (key, value) in map {
                if !matches!(key, Value::String(_)) {
                    return Err(OkfError::Usage(
                        "structured YAML mapping keys must be strings".into(),
                    ));
                }
                if key.as_str() == Some("<<") {
                    return Err(OkfError::Usage("YAML merge keys are unsupported".into()));
                }
                check_value(value, depth + 1)?;
            }
        }
        Value::Sequence(items) => {
            for item in items {
                check_value(item, depth + 1)?;
            }
        }
        Value::Tagged(_) => return Err(OkfError::Usage("YAML tags are unsupported".into())),
        Value::Number(n) if n.as_f64().is_some_and(|n| !n.is_finite()) => {
            return Err(OkfError::Usage(
                "structured YAML numbers must be finite".into(),
            ));
        }
        _ => {}
    }
    Ok(())
}

/// Inspect tokens before deserialization so aliases cannot expand or lose their identity.
/// Uses the same YAML scanner dependency as serde_yaml; token inspection leaves quoted and
/// block scalar contents untouched.
fn check_tokens(input: &str) -> Result<()> {
    use unsafe_libyaml as unsafe_yaml;
    // SAFETY: parser lives in one boxed allocation until deletion, input remains alive for
    // the entire scan, and each successful scan's initialized token is deleted exactly once.
    unsafe {
        let mut parser = Box::new(MaybeUninit::<unsafe_yaml::yaml_parser_t>::uninit());
        let parser_ptr = parser.as_mut_ptr();
        if unsafe_yaml::yaml_parser_initialize(parser_ptr).fail {
            return Err(OkfError::Internal("cannot initialize YAML scanner".into()));
        }
        unsafe_yaml::yaml_parser_set_input_string(parser_ptr, input.as_ptr(), input.len() as u64);
        let result = loop {
            let mut token = MaybeUninit::<unsafe_yaml::yaml_token_t>::uninit();
            if unsafe_yaml::yaml_parser_scan(parser_ptr, token.as_mut_ptr()).fail {
                break Err(OkfError::Usage("invalid structured YAML syntax".into()));
            }
            let mut token = token.assume_init();
            let kind = token.type_;
            let line = token.start_mark.line + 1;
            unsafe_yaml::yaml_token_delete(&mut token);
            if matches!(
                kind,
                unsafe_yaml::YAML_ALIAS_TOKEN
                    | unsafe_yaml::YAML_ANCHOR_TOKEN
                    | unsafe_yaml::YAML_TAG_TOKEN
                    | unsafe_yaml::YAML_TAG_DIRECTIVE_TOKEN
            ) {
                break Err(OkfError::Usage(format!(
                    "YAML anchors, aliases and tags are unsupported (line {line})"
                )));
            }
            if kind == unsafe_yaml::YAML_STREAM_END_TOKEN {
                break Ok(());
            }
        };
        unsafe_yaml::yaml_parser_delete(parser_ptr);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_objects_lists_json_and_literal_yaml_punctuation() {
        for input in [
            "{within: 72, unit: hours}",
            "[a, b]",
            r#"{"x":[1,true,null]}"#,
            "'!tag &anchor *alias'",
            "text: |\n  !tag &anchor *alias\n",
        ] {
            assert!(parse_yaml(input).is_ok(), "{input}");
        }
    }

    #[test]
    fn rejects_ambiguous_or_unsupported_documents() {
        for input in [
            "a: 1\na: 2",
            "x: {a: 1, a: 2}",
            "1: x",
            "x: !thing hi",
            "x: !!str hi",
            "x: &a hi\ny: *a",
            "<<: {x: 1}",
            "---\na\n---\nb",
            ".inf",
            "{x: [}",
        ] {
            assert!(parse_yaml(input).is_err(), "{input}");
        }
    }

    #[test]
    fn reader_resolves_files_and_rejects_multiple_stdin_consumers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("value.yaml");
        std::fs::write(&path, "{within: 72}").unwrap();
        let mut reader = InputReader::default();
        assert_eq!(
            reader.read(&format!("@{}", path.display())).unwrap(),
            "{within: 72}"
        );
        reader.stdin_used = true;
        assert!(reader.read("-").is_err());
    }
}
