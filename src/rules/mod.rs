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
pub mod docs_check_lists;
pub mod docs_inherited_adrs;
pub mod docs_links;
pub mod docs_no_draft_on_main;
pub mod docs_project_layout;
pub mod docs_prose_width;
pub mod docs_section_refs;
pub mod docs_skill_pointers;
pub mod docs_template_identity;
pub mod docs_template_release;
pub mod traceability_citations;
pub mod traceability_coverage;
pub mod traceability_criteria;

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
    pub link_targets: std::collections::BTreeMap<String, docs_links::Resolved>,
    /// The bytes of `AGENTS.md` at the root, if it is a file.
    pub agents: Option<Vec<u8>>,
    /// The template release the tree carries, for a rule whose bash check changed between
    /// releases; none for `unreleased`, which is checked as the newest release this `nuc` knows.
    pub release: Option<Version>,
    /// The files of the check lists.
    pub check_lists: docs_check_lists::CheckLists,
    /// What the paths of the README's Project Layout name, and the top-level parts of the tree.
    pub layout: docs_project_layout::Layout,
    /// The skills and their pointers.
    pub skills: docs_skill_pointers::Skills,
    /// Whether `TEMPLATE-SETUP.md` is a file: the setup still runs.
    pub setup_file: bool,
    /// The files check 13 reads for the template's placeholders, by path, each that is a file.
    pub identity: std::collections::BTreeMap<String, Vec<u8>>,
    /// The Markdown files with a line over 100 bytes that git's attributes mark
    /// `linguist-generated`, as `set` or `true`.
    pub generated: std::collections::BTreeSet<String>,
    /// What CI says about the change, and the drafts of the record.
    pub landing: docs_no_draft_on_main::Landing,
    /// The bytes of `docs/SPECIFICATION.md`, if it is a file.
    pub specification: Option<Vec<u8>>,
    /// Every file that may carry a `Verifies:` marker, by its path, as the bash's traceability
    /// check reads them: listed by git as tracked or new and not ignored, a file, not Markdown,
    /// not binary to `grep -I`, and holding the marker.
    pub markers: std::collections::BTreeMap<String, Vec<u8>>,
}

pub use crate::adr::read_lines;

impl Inputs {
    /// Whether the tree is checked as the bash check of `release` or of a later one.
    pub fn carries_at_least(&self, release: Version) -> bool {
        self.release.is_none_or(|carried| carried >= release)
    }
}

/// The template releases at which a ported bash check changed, the earliest first: a rule checks
/// a tree as the latest of them at or before the release it carries.
pub const RELEASES: &[Version] = &[Version(0, 5, 0), Version(0, 6, 0)];

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
    /// The workflows, `scripts/check-all.sh`, and the scripts under `scripts/`.
    CheckLists,
    /// The paths of the README's Project Layout, tested, and the tracked top-level parts.
    Layout,
    /// The skills and their pointers.
    Skills,
    /// Whether `TEMPLATE-SETUP.md` is a file.
    SetupFile,
    /// The files the template's placeholders stand in.
    Identity,
    /// The Markdown files git's attributes mark generated.
    Generated,
    /// The environment CI sets for the change, and the draft files of `docs/adr/`.
    Landing,
    /// `docs/SPECIFICATION.md`.
    Specification,
    /// The files that carry a `Verifies:` marker.
    Markers,
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
    docs_check_lists::RULE,
    docs_project_layout::RULE,
    docs_skill_pointers::RULE,
    docs_inherited_adrs::RULE,
    docs_template_identity::RULE,
    docs_prose_width::RULE,
    docs_no_draft_on_main::RULE,
    docs_template_release::RULE,
    traceability_criteria::RULE,
    traceability_citations::RULE,
    traceability_coverage::RULE,
];

/// The rule with the ID, if this `nuc` knows it.
pub fn find(id: &str) -> Option<&'static Rule> {
    ALL.iter().find(|rule| rule.id == id)
}
