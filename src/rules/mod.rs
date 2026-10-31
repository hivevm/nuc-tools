//! The rules: one module each, with one rule ID, unique across the crate (G-6,
//! ADR-0009). A rule is a pure function from the [`Inputs`] the shell gathered
//! to its findings.

use crate::config::Version;
use crate::finding::Finding;

pub mod docs_adr_header;
pub mod docs_adr_index;
pub mod docs_adr_link_targets;
pub mod docs_adr_numbering;
pub mod docs_adr_refs;
pub mod docs_adr_supersessions;
pub mod docs_links;
pub mod docs_section_refs;
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
    /// Each path a relative link of a Markdown file names, joined to the file's directory as
    /// text, resolved.
    pub link_targets: std::collections::BTreeMap<String, Resolved>,
    /// The bytes of `AGENTS.md` at the root, if it is a file.
    pub agents: Option<Vec<u8>>,
    /// The template release the tree carries, for a rule whose bash check changed between
    /// releases; none for `unreleased`, which is checked as the newest release this `nuc` knows.
    pub release: Option<Version>,
}

impl Inputs {
    /// Whether the tree is checked as the bash check of `release` or of a later one.
    pub fn carries_at_least(&self, release: Version) -> bool {
        self.release.is_none_or(|carried| carried >= release)
    }
}

/// The template release a README names on its `Template release` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Carried {
    Release(Version),
    Unreleased,
}

/// What the shell reads from the repository for a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    /// `README.md` at the root.
    Readme,
    /// The files of `docs/adr/` and the targets of the index.
    Adr,
    /// Every text file of the repository.
    TextFiles,
    /// The paths the relative links of the Markdown files name, resolved.
    LinkTargets,
    /// `AGENTS.md` at the root.
    Agents,
    /// The template release `README.md` names, which the shell refuses when it cannot place it.
    Release,
}

/// A path a relative link names, as the bash tests it: whether it exists, as `-e` finds it, and
/// the bytes of a file an anchor of the link looks into.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolved {
    pub exists: bool,
    pub file: Option<Vec<u8>>,
}

/// One rule: its ID, the reason `nuc explain` gives, what it reads, and the check.
pub struct Rule {
    pub id: &'static str,
    pub explain: &'static str,
    /// The inputs the check reads; the shell reads only those a rule turned on needs.
    pub reads: &'static [Input],
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
    docs_links::RULE,
    docs_section_refs::RULE,
    docs_template_release::RULE,
];

/// The rule with the ID, if this `nuc` knows it.
pub fn find(id: &str) -> Option<&'static Rule> {
    ALL.iter().find(|rule| rule.id == id)
}
