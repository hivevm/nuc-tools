//! The rules: one module each, with one rule ID, unique across the crate (G-6,
//! ARCH-rules-outcome-hooks-answer). A rule is a pure function from the [`Inputs`] the shell
//! gathered to its outcome: its findings, a refusal of its own, and notes for the summary.

use crate::config::Version;
use crate::finding::Finding;

pub mod docs_adr_header;
pub mod docs_adr_identifiers;
pub mod docs_adr_index;
pub mod docs_adr_link_targets;
pub mod docs_adr_numbering;
pub mod docs_adr_refs;
pub mod docs_adr_supersessions;
pub mod docs_base_blobs;
pub mod docs_check_lists;
pub mod docs_inherited_adrs;
pub mod docs_links;
pub mod docs_no_draft_on_main;
pub mod docs_opens_no_issue;
pub mod docs_project_layout;
pub mod docs_prose_width;
pub mod docs_section_refs;
pub mod docs_skill_pointers;
pub mod docs_template_identity;
pub mod docs_template_release;
pub mod hooks_pin;
pub mod hooks_toml;
pub mod shell_mode;
pub mod traceability_citations;
pub mod traceability_coverage;
pub mod traceability_criteria;
pub mod traceability_sensor_tests;

/// What the shell reads from the repository for the rules, before any of them runs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inputs {
    /// The text of `README.md` at the root, if the file exists.
    pub readme: Option<String>,
    /// The text of `CONTRIBUTING.md` at the root, if the file exists: from template release
    /// v0.7.0 on, it holds the required checks and the Project Layout.
    pub contributing: Option<String>,
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
    /// What the paths of the Project Layout name, and the top-level parts of the tree.
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
    /// The ADR files the first-parent history deleted, as `docs/adr-identifiers` reads them.
    pub history: docs_adr_identifiers::History,
    /// The blobs of the base and the list of them, as `docs/base-blobs` reads them.
    pub base: docs_base_blobs::Base,
    /// The bytes of `docs/SPECIFICATION.md`, if it is a file.
    pub specification: Option<Vec<u8>>,
    /// Every file that may carry a `Verifies:` marker, by its path, as the bash's traceability
    /// check reads them: listed by git as tracked or new and not ignored, a file, not Markdown,
    /// not binary to `grep -I`, and holding the marker.
    pub markers: std::collections::BTreeMap<String, Vec<u8>>,
    /// The sensors and their self-tests, as `traceability/sensor-tests` reads them.
    pub sensors: traceability_sensor_tests::Sensors,
    /// `README.md` at the merge base of `HEAD` with `origin/main`, else with `main`, empty where
    /// that commit has none; none where neither has a merge base with `HEAD`.
    pub release_base: Option<Vec<u8>>,
    /// The rule keys the header of `scripts/check-docs.sh` lists, with the old keys it names for
    /// them; none where the tree has no such file.
    pub rule_keys: Option<std::collections::BTreeSet<String>>,
    /// The index entries of the scripts `shell/mode` holds, as `git ls-files -s` lists them.
    pub index_modes: Vec<shell_mode::IndexEntry>,
    /// The blob list of the hooks and the blob of each hook, as `scripts/check-hooks.sh` reads
    /// them.
    pub hooks: crate::pin::Hooks,
    /// `nuc.toml` as `nuc` read it for the run. The shell always gives it, since `nuc check`
    /// refuses a tree without one before any rule runs: none occurs only in unit tests.
    pub toml: Option<crate::config::Config>,
}

pub use crate::bash::read_lines;

impl Inputs {
    /// Whether the tree is checked as the bash check of `release` or of a later one.
    pub fn carries_at_least(&self, release: Version) -> bool {
        self.release.is_none_or(|carried| carried >= release)
    }

    /// The file that holds the required checks and the Project Layout on the release the tree is
    /// checked as, by its name, and its text if it exists.
    pub fn guide(&self) -> (&'static str, Option<&str>) {
        if guide_is_contributing(self.release) {
            ("CONTRIBUTING.md", self.contributing.as_deref())
        } else {
            ("README.md", self.readme.as_deref())
        }
    }
}

/// The template releases at which a ported bash check changed, the earliest first: a rule checks
/// a tree as the latest of them at or before the release it carries.
pub const RELEASES: &[Version] = &[
    Version(0, 5, 0),
    Version(0, 6, 0),
    Version(0, 6, 1),
    Version(0, 7, 0),
    CATEGORY_IDENTIFIERS,
    TEMPLATE_FINDINGS,
    HOOK_PINS,
];

/// The release from which a hook the blob list `.githooks/nuc.blobs` pins is another repository's
/// file, held by its pin, and no text file of the tree a rule reads (NUC-hooks-run-nuc §2): the
/// bash's collect_text_files leaves it out, which `docs/adr-refs`, `docs/section-refs`, and
/// `docs/opens-no-issue` read, while the functions of those rules stay as they were.
pub const HOOK_PINS: Version = Version(0, 12, 0);

/// The release from which a derived project sends its findings about the template by hand, the
/// release line moves only over accepted `NUC-` ADRs, and each rule key of `scripts/check-docs.sh`
/// an ADR names is one its header lists.
pub const TEMPLATE_FINDINGS: Version = Version(0, 10, 0);

/// The release from which an ADR is named `<CAT>-<slug>` by category, its file name its
/// identifier, and the record no longer knows numbers, drafts, or inherited ADRs.
pub const CATEGORY_IDENTIFIERS: Version = Version(0, 9, 0);

/// The release from which the required checks and the Project Layout stand in `CONTRIBUTING.md`.
pub const CONTRIBUTING_GUIDE: Version = Version(0, 7, 0);

/// Whether the guide of a tree checked as `release` is `CONTRIBUTING.md` rather than the README:
/// from [`CONTRIBUTING_GUIDE`] on, and for none, the newest release.
pub fn guide_is_contributing(release: Option<Version>) -> bool {
    release.is_none_or(|release| release >= CONTRIBUTING_GUIDE)
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
    /// `CONTRIBUTING.md` at the root.
    Contributing,
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
    /// The paths of the Project Layout, in `README.md` and in `CONTRIBUTING.md`, tested, and the
    /// tracked top-level parts.
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
    /// The ADR files the first-parent history deleted, as `docs/adr-identifiers` reads them.
    History,
    /// The blobs of the base and the list of them, as `docs/base-blobs` reads them.
    Base,
    /// `docs/SPECIFICATION.md`.
    Specification,
    /// The files that carry a `Verifies:` marker.
    Markers,
    /// The sensors of `scripts/`, their self-tests, and the ADR that decides the checks.
    Sensors,
    /// `README.md` at the merge base with `main`.
    ReleaseBase,
    /// The rule keys of the header of `scripts/check-docs.sh`.
    RuleKeys,
    /// The modes in git's index of the scripts under `scripts/` and the skills.
    IndexModes,
    /// The blob list of the hooks, `.githooks/nuc.blobs`.
    HookList,
    /// The blob git would store for each hook of the five under `.githooks/`.
    HookBlobs,
    /// `nuc.toml` as `nuc` read it.
    Toml,
}

/// One rule: its ID, the reason `nuc explain` gives, what it reads, and the check.
pub struct Rule {
    pub id: &'static str,
    /// The reasons its findings carry, each with the ID the key `<id>:<reason>` of a line
    /// (API-command-line-with-measure); a finding with another is refused.
    pub reasons: &'static [&'static str],
    pub explain: &'static str,
    /// The inputs the check reads; the shell reads only those a rule turned on needs.
    pub reads: &'static [Input],
    pub check: fn(&Inputs) -> Outcome,
}

/// What a rule returns: its findings; a refusal of its own, the reason it cannot decide the tree
/// as its bash check would; and notes for the summary of the run, which are no findings
/// (ARCH-rules-outcome-hooks-answer).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    pub findings: Vec<Finding>,
    pub refusal: Option<String>,
    pub notes: Vec<String>,
}

impl Outcome {
    /// Why the outcome of the rule `id`, whose findings carry one of `reasons`, cannot be carried
    /// out: its own refusal; a finding it reports under another rule's ID, which every format
    /// would name wrongly and the SARIF could not place in its list of rules
    /// (API-output-formats-carry-reason); or a finding with a reason it does not declare, whose key
    /// no list holds (API-command-line-with-measure). Each ends the run not decided (Q-1).
    pub fn refusal_of(&self, id: &str, reasons: &[&str]) -> Option<String> {
        if let Some(refusal) = &self.refusal {
            return Some(refusal.clone());
        }
        self.findings.iter().find_map(|f| {
            if f.rule != id {
                Some(format!(
                    "{id}: reports a finding under the rule ID '{}', which is not its own — not \
                     decided",
                    f.rule
                ))
            } else if !reasons.contains(&f.reason) {
                Some(format!(
                    "{id}: reports a finding with the reason '{}', which it does not declare — not \
                     decided",
                    f.reason
                ))
            } else {
                None
            }
        })
    }
}

impl From<Vec<Finding>> for Outcome {
    fn from(findings: Vec<Finding>) -> Self {
        Self {
            findings,
            ..Self::default()
        }
    }
}

/// Every rule this `nuc` knows, in the order their findings are reported.
pub const ALL: &[Rule] = &[
    docs_adr_index::RULE,
    docs_adr_numbering::RULE,
    docs_adr_identifiers::RULE,
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
    docs_base_blobs::RULE,
    docs_template_identity::RULE,
    docs_prose_width::RULE,
    docs_no_draft_on_main::RULE,
    docs_opens_no_issue::RULE,
    docs_template_release::RULE,
    hooks_pin::RULE,
    hooks_toml::RULE,
    shell_mode::RULE,
    traceability_criteria::RULE,
    traceability_citations::RULE,
    traceability_coverage::RULE,
    traceability_sensor_tests::RULE,
];

/// The rule with the ID, if this `nuc` knows it.
pub fn find(id: &str) -> Option<&'static Rule> {
    ALL.iter().find(|rule| rule.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_guide_is_contributing_from_its_release_on_and_where_none_is_placed() {
        assert!(!guide_is_contributing(Some(Version(0, 6, 1))));
        assert!(guide_is_contributing(Some(CONTRIBUTING_GUIDE)));
        assert!(guide_is_contributing(Some(Version(0, 8, 0))));
        assert!(guide_is_contributing(None));
    }

    #[test]
    fn an_outcome_refuses_a_finding_of_another_rule_and_its_own_refusal_comes_first() {
        let finding = |rule| Finding {
            rule,
            reason: "r",
            file: "a.md".to_owned(),
            line: None,
            message: "m".to_owned(),
        };
        let own: Outcome = vec![finding("docs/links")].into();
        assert_eq!(own.refusal_of("docs/links", &["r"]), None);
        assert_eq!(
            own.refusal_of("docs/links", &["s"]).as_deref(),
            Some(
                "docs/links: reports a finding with the reason 'r', which it does not declare — not decided"
            )
        );
        let foreign: Outcome = vec![finding("docs/links"), finding("docs/adr-index")].into();
        assert_eq!(
            foreign.refusal_of("docs/links", &["r"]).as_deref(),
            Some(
                "docs/links: reports a finding under the rule ID 'docs/adr-index', which is not its \
                 own — not decided"
            )
        );
        let refused = Outcome {
            refusal: Some("why".to_owned()),
            ..foreign
        };
        assert_eq!(
            refused.refusal_of("docs/links", &["r"]).as_deref(),
            Some("why")
        );
    }
}
