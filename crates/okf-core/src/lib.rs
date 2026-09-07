//! okf-core: deterministic OKF harness logic (the "hands").
//!
//! Pure library — no argv, no stdout, no `process::exit`. All effects go through
//! [`ports`]. See ARCHITECTURE.md for the module map and rationale.
#![allow(dead_code, unused)]

pub mod error;
pub mod model;
pub mod bundle;
pub mod parse;
pub mod ontology;
pub mod graph;
pub mod fingerprint;
pub mod check;
pub mod query;
pub mod mutate;
pub mod render;
pub mod output;
pub mod ports;
