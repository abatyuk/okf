//! okf-cli: the `okf` binary. Arg parsing, output rendering, exit-code mapping.
//!
//! Thin layer over okf-core (the deterministic "hands"). See ARCHITECTURE.md.
#![allow(dead_code, unused)]

mod cli;
mod commands;
mod exit;
mod output;

fn main() {
    reset_sigpipe();
    std::process::exit(exit::run());
}

/// Restore the default SIGPIPE disposition so `okf … | head` (and other truncated
/// pipes) exit quietly instead of panicking on a broken stdout write.
#[cfg(unix)]
fn reset_sigpipe() {
    // SAFETY: single libc call at startup, before any threads or output.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn reset_sigpipe() {}
