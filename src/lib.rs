//! `nuc`: the mechanics of the NUC template. The rules are pure functions over values; every
//! effect lives in [`shell`] (ADR-0009).

pub mod adr;
pub mod bash;
pub mod config;
pub mod finding;
pub mod glob;
pub mod markdown;
pub mod rules;
pub mod shell;
pub mod traceability;
