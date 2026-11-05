//! Check 10 of the template's `scripts/check-docs.sh`: every path the `Project Layout` block of
//! `CONTRIBUTING.md` (`README.md` before v0.7.0) lists exists, a trailing `/` meaning a
//! directory and a `*` a pattern that matches, and every tracked top-level part of the tree is
//! named, by itself or by a path under it.

use super::{Input, Inputs, Outcome, Rule, read_lines};
use crate::bash::is_space;
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/project-layout";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Every path the 'Project Layout' block of CONTRIBUTING.md (README.md before template \
              release v0.7.0) lists exists, a trailing '/' meaning a directory and a '*' a \
              pattern that matches at least one file; docs/plan/ may be absent, since open work \
              lives there only while there is some. The block is written by hand and nothing \
              else tells a reader that a line of it has gone stale. Every tracked top-level file \
              and directory is named, by itself or by a path under it; below the top level the \
              block is a tour, not a listing.",
    reads: &[
        Input::Readme,
        Input::Contributing,
        Input::Layout,
        Input::Release,
    ],
    check,
};

/// What check 10 tests on the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Layout {
    /// Each path the Project Layout block lists, by its text: whether it names what is there,
    /// a match of a pattern with `*`, a directory for a trailing `/`, or else a file.
    pub found: std::collections::BTreeMap<String, bool>,
    /// The first part of every path git tracks, sorted by its bytes and each once by them, as
    /// `cut -d/ -f1 | LC_ALL=C sort -u` gives them; a byte of no UTF-8 is replaced afterwards.
    pub top_level: Vec<String>,
    /// The parts of `top_level` whose bytes are no UTF-8, which a finding cannot spell.
    pub unspelled: std::collections::BTreeSet<String>,
}

/// The directory open work lives in only while there is some.
const PLAN: &str = "docs/plan/";

fn finding(file: &str, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        file: file.to_owned(),
        line: None,
        message,
    }
}

/// The paths of the Project Layout block, in order: the first word of each line of the first
/// fence under `## Project Layout`, but blank lines and comments.
pub fn listed(text: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut in_section = false;
    let mut in_fence = false;
    for line in read_lines(text.as_bytes()) {
        let fence = line.starts_with(b"```") || line.starts_with(b"~~~");
        if !in_section {
            in_section = line == b"## Project Layout";
            continue;
        }
        if !in_fence {
            if fence {
                in_fence = true;
                continue;
            }
            if line.starts_with(b"##") && line.get(2).is_some_and(|&b| is_space(b)) {
                break;
            }
            continue;
        }
        if fence {
            break;
        }
        let path = &line[..line.iter().take_while(|&&b| !is_space(b)).count()];
        if path.is_empty() || path.starts_with(b"#") {
            continue;
        }
        paths.push(String::from_utf8_lossy(path).into_owned());
    }
    paths
}

/// Whether the shell tests the path as a pattern.
pub fn is_pattern(path: &str) -> bool {
    path.contains('*')
}

/// The findings of the rule, or its refusal where one would name a top-level part whose name is
/// no UTF-8, which `nuc` cannot spell as the bash does (Q-1).
fn check(inputs: &Inputs) -> Outcome {
    let (guide, text) = inputs.guide();
    let Some(text) = text else {
        return vec![finding(
            guide,
            "not found — the Project Layout block cannot be checked".to_owned(),
        )]
        .into();
    };
    let paths = listed(text);
    if paths.is_empty() {
        return vec![finding(
            guide,
            "no paths found under '## Project Layout' — the inventory is unverifiable".to_owned(),
        )]
        .into();
    }
    let mut found = Vec::new();
    for path in &paths {
        let there = inputs.layout.found.get(path).copied().unwrap_or(false);
        let message = if is_pattern(path) {
            "which matches nothing in this repository"
        } else if path.ends_with('/') {
            if path == PLAN {
                continue;
            }
            "which is not a directory of this repository"
        } else {
            "which does not exist"
        };
        if !there {
            found.push(finding(
                guide,
                format!("Project Layout lists '{path}', {message}"),
            ));
        }
    }
    for top in &inputs.layout.top_level {
        let covered = paths
            .iter()
            .any(|path| path == top || path.starts_with(&format!("{top}/")));
        if !covered {
            let message = format!(
                "Project Layout does not name '{top}', a top-level part of this repository"
            );
            if inputs.layout.unspelled.contains(top) {
                return Outcome {
                    refusal: Some(format!(
                        "{guide}: {message}, whose name is no UTF-8 — not decided"
                    )),
                    ..Outcome::default()
                };
            }
            found.push(finding(guide, message));
        }
    }
    found.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAYOUT: &str = "# P\n\n## Project Layout\n\nIntro.\n\n```text\nREADME.md      the entry\nsrc/           the code\n\n# a comment\ndocs/plan/     open work\nscripts/*.sh   the checks\n```\n\n## Next\n";

    fn messages(guide: &str, found: &[(&str, bool)], top: &[&str]) -> Vec<String> {
        let inputs = Inputs {
            contributing: Some(guide.to_owned()),
            layout: super::Layout {
                found: found.iter().map(|(p, f)| ((*p).to_owned(), *f)).collect(),
                top_level: top.iter().map(|t| (*t).to_owned()).collect(),
                ..super::Layout::default()
            },
            ..Inputs::default()
        };
        check(&inputs)
            .findings
            .into_iter()
            .map(|f| f.message)
            .collect()
    }

    #[test]
    fn the_block_lists_its_first_words_but_blank_lines_and_comments() {
        assert_eq!(
            listed(LAYOUT),
            ["README.md", "src/", "docs/plan/", "scripts/*.sh"]
        );
        assert_eq!(
            listed("## Project Layout\n\n## Next\n```\nx\n```\n"),
            Vec::<String>::new()
        );
        assert_eq!(listed("## Project Layout\n```\nlast\n```"), ["last"]);
        assert_eq!(listed("## Project Layout\n```\nlast"), Vec::<String>::new());
    }

    #[test]
    fn a_layout_whose_paths_exist_and_name_the_top_level_passes() {
        let found = [
            ("README.md", true),
            ("src/", true),
            ("docs/plan/", false),
            ("scripts/*.sh", true),
        ];
        // bash: docs/plan/ named while there is no open work
        assert_eq!(
            messages(LAYOUT, &found, &["README.md", "docs", "scripts", "src"]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_missing_path_and_an_unnamed_top_level_part_fail() {
        let found = [
            ("README.md", false),
            ("src/", false),
            ("docs/plan/", false),
            ("scripts/*.sh", false),
        ];
        // bash: a Project Layout entry that no longer exists
        // bash: a top-level file the Project Layout does not name
        // bash: a Project Layout pattern that matches nothing
        assert_eq!(
            messages(LAYOUT, &found, &["Cargo.toml", "src"]),
            [
                "Project Layout lists 'README.md', which does not exist",
                "Project Layout lists 'src/', which is not a directory of this repository",
                "Project Layout lists 'scripts/*.sh', which matches nothing in this repository",
                "Project Layout does not name 'Cargo.toml', a top-level part of this repository",
            ]
        );
    }

    #[test]
    fn no_guide_and_an_empty_block_fail_alone() {
        let inputs = Inputs::default();
        assert_eq!(
            check(&inputs).findings[0].message,
            "not found — the Project Layout block cannot be checked"
        );
        assert_eq!(
            messages("# P\n", &[], &["src"]),
            ["no paths found under '## Project Layout' — the inventory is unverifiable"]
        );
    }

    /// Verifies: ADR-0021
    #[test]
    fn before_v0_7_0_the_readme_holds_the_block_and_from_it_the_contributing_guide() {
        let inputs = |release| Inputs {
            readme: Some(LAYOUT.to_owned()),
            contributing: Some("# Contributing\n".to_owned()),
            release: Some(release),
            layout: super::Layout {
                found: [("README.md", true), ("src/", true), ("scripts/*.sh", true)]
                    .iter()
                    .map(|(p, f)| ((*p).to_owned(), *f))
                    .collect(),
                top_level: vec!["src".to_owned()],
                ..super::Layout::default()
            },
            ..Inputs::default()
        };
        let reported = |release| -> Vec<String> {
            check(&inputs(release))
                .findings
                .into_iter()
                .map(|f| format!("{}: {}", f.file, f.message))
                .collect()
        };
        assert_eq!(
            reported(crate::config::Version(0, 6, 2)),
            Vec::<String>::new()
        );
        assert_eq!(
            reported(super::super::CONTRIBUTING_GUIDE),
            [
                "CONTRIBUTING.md: no paths found under '## Project Layout' — the inventory is unverifiable"
            ]
        );
    }
}
