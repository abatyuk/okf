use crate::cli::ChangeSetCmd;
use crate::{output, structured};
use okf_core::bundle::resolve::resolve_bundle;
use okf_core::error::{OkfError, Result};
use okf_core::mutate::{changeset, transaction};
use serde_json::json;

pub fn run(command: &ChangeSetCmd, json_output: bool) -> Result<i32> {
    let (args, apply) = match command {
        ChangeSetCmd::Plan(a) => (a, false),
        ChangeSetCmd::Apply(a) => (a, !a.dry_run),
        ChangeSetCmd::Recover(a) => {
            let root = resolve_bundle(a.bundle.as_deref())?;
            let paths = transaction::recover(&root)?;
            if json_output {
                output::print_line(&json!({"kind":"changeset-recovery","paths":paths}))?;
            } else {
                output::print_text_line(format_args!("recovered {} file(s)", paths.len()))?;
            }
            return Ok(0);
        }
    };
    let root = resolve_bundle(args.bundle.as_deref())?;
    let raw = structured::InputReader::default().read(&args.from)?;
    let value = structured::parse_yaml(&raw)?;
    let set = serde_yaml::from_value(value)
        .map_err(|e| OkfError::Usage(format!("invalid change set: {e}")))?;
    let plan = changeset::prepare(&root, &set)?;
    if args
        .expect
        .as_deref()
        .is_some_and(|expected| expected != plan.base_digest)
    {
        return Err(OkfError::Usage(
            "change-set base digest differs from --expect; preview again".into(),
        ));
    }
    let digest = plan.base_digest.clone();
    let count = plan.changes.len();
    let records = plan.changes.iter().map(|c| {
        let diff = match (c.before.as_deref(), c.after.as_deref()) {
            (before, after) => match (std::str::from_utf8(before.unwrap_or_default()), std::str::from_utf8(after.unwrap_or_default())) {
                (Ok(old), Ok(new)) => Some(structured::preview_diff(&c.path, old, new)),
                _ => None,
            }
        };
        json!({"kind":"changeset-file","path":c.path,"operation":if c.before.is_none(){"create"}else if c.after.is_none(){"remove"}else{"replace"},"before_bytes":c.before.as_ref().map(Vec::len),"after_bytes":c.after.as_ref().map(Vec::len),"diff":diff})
    }).collect::<Vec<_>>();
    if apply {
        changeset::apply(&root, plan, args.expect.as_deref())?;
    }
    for record in records {
        if json_output {
            output::print_line(&record)?;
        } else if let Some(diff) = record["diff"].as_str() {
            output::print_text(format_args!("{diff}"))?;
        } else {
            output::print_text_line(format_args!(
                "{} {} (binary)",
                record["operation"], record["path"]
            ))?;
        }
    }
    if json_output {
        output::print_line(
            &json!({"kind":"changeset-summary","base_digest":digest,"files":count,"applied":apply,"publication":"journaled-per-file"}),
        )?;
    } else {
        output::print_text_line(format_args!(
            "{} {count} file(s); base digest: {digest}",
            if apply { "applied" } else { "planned" }
        ))?;
    }
    Ok(0)
}
