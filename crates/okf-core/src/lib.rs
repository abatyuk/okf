//! okf-core: deterministic OKF harness logic (the "hands").
//!
//! Pure library — no argv, no stdout, no `process::exit`. All effects go through
//! [`ports`]. See ARCHITECTURE.md for the module map and rationale.
#![allow(dead_code, unused)]

pub mod bundle;
pub mod check;
pub mod error;
pub mod fingerprint;
pub mod graph;
pub mod model;
pub mod mutate;
pub mod ontology;
pub mod output;
pub mod parse;
pub mod ports;
pub mod query;
pub mod render;
