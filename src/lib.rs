//! `nuc`: the mechanics of the NUC template. The rules are pure functions over values; every
//! effect lives in [`shell`] (ADR-draft-functional-core).

pub mod config;
pub mod finding;
pub mod rules;
pub mod shell;
