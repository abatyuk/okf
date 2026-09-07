//! Map a command result (and lint/stale findings) to an exit code.
//!
//! 0 ok · 1 findings at/above `--fail-on` (later) · 2 usage · 3 env/IO · 4 internal.
use crate::cli::Cli;
use crate::commands;
use clap::error::ErrorKind;
use clap::Parser;

/// Parse argv, dispatch, and return the process exit code.
pub fn run() -> i32 {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // clap prints help/version/usage itself.
            let _ = err.print();
            return match err.kind() {
                ErrorKind::DisplayHelp
                | ErrorKind::DisplayVersion
                | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => 0,
                _ => 2,
            };
        }
    };

    match commands::run(cli) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("okf: {err}");
            err.exit_code()
        }
    }
}
