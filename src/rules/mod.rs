//! The rules: one module each, with one rule ID, unique across the crate (G-6,
//! ADR-0009). A rule is a pure function from the [`Inputs`] the shell gathered
//! to its findings.

use crate::finding::Finding;

pub mod docs_adr_header;
pub mod docs_adr_index;
pub mod docs_adr_link_targets;
pub mod docs_adr_numbering;
pub mod docs_adr_refs;
pub mod docs_adr_supersessions;
pub mod docs_template_release;

/// What the shell reads from the repository for the rules, before any of them runs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inputs {
    /// The text of `README.md` at the root, if the file exists.
    pub readme: Option<String>,
    /// The files of `docs/adr/` the bash reads, by name: the ADRs, the index, and the template.
    pub adr: crate::adr::Dir,
    /// Each file a row of the index links, by its target as the row writes it: its bytes when
    /// it is a file, as `-f` finds it, or none.
    pub adr_targets: std::collections::BTreeMap<String, Option<Vec<u8>>>,
    /// Every text file of the repository by its path, as the bash lists them: the files git
    /// lists as tracked or new and not ignored that are files, have a line, and hold no NUL byte.
    pub text_files: std::collections::BTreeMap<String, Vec<u8>>,
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
    docs_adr_supersessions::RULE,
    docs_adr_refs::RULE,
    docs_adr_link_targets::RULE,
    docs_template_release::RULE,
];

/// The rule with the ID, if this `nuc` knows it.
pub fn find(id: &str) -> Option<&'static Rule> {
    ALL.iter().find(|rule| rule.id == id)
}
