//! `nuc`: the mechanics of the NUC template. The rules are pure functions over values; every
//! effect lives in [`shell`] (ARCH-rules-outcome-hooks-answer).

pub mod adr;
pub mod bash;
pub mod config;
pub mod finding;
pub mod format;
pub mod glob;
pub mod hook;
pub mod markdown;
pub mod measure;
pub mod rules;
pub mod shell;
#[cfg(test)]
mod testing;
pub mod traceability;
