//! The rules: one module each, with one rule ID, unique across the crate (G-6,
//! ADR-0009). A rule is a pure function from the [`Inputs`] the shell gathered
//! to its findings.

use crate::finding::Finding;

pub mod docs_adr_header;
pub mod docs_adr_index;
pub mod docs_adr_numbering;
pub mod docs_template_release;

/// What the shell reads from the repository for the rules, before any of them runs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inputs {
    /// The text of `README.md` at the root, if the file exists.
    pub readme: Option<String>,
    /// The files directly in `docs/adr/`, by name; empty when the directory does not exist.
    pub adr: crate::adr::Dir,
}

/// One rule: its ID, the reason `nuc explain` gives, and the check.
pub struct Rule {
    pub id: &'static str,
    pub explain: &'static str,
    pub check: fn(&Inputs) -> Vec<Finding>,
}

/// Every rule this `nuc` knows, in the order their findings are reported.
pub const ALL: &[Rule] = &[
    docs_adr_index::RULE,
    docs_adr_numbering::RULE,
    docs_adr_header::RULE,
    docs_template_release::RULE,
];

/// The rule with the ID, if this `nuc` knows it.
pub fn find(id: &str) -> Option<&'static Rule> {
    ALL.iter().find(|rule| rule.id == id)
}
