//! Check 10 of the template's `scripts/check-docs.sh`: every path the `Project Layout` block of
//! `README.md` lists exists, a trailing `/` meaning a directory and a `*` a pattern that matches,
//! and every tracked top-level part of the tree is named, by itself or by a path under it.

use super::{Input, Inputs, Rule, read_lines};
use crate::adr::is_space;
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/project-layout";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Every path the 'Project Layout' block of README.md lists exists, a trailing '/' \
              meaning a directory and a '*' a pattern that matches at least one file; \
              docs/plan/ may be absent, since open work lives there only while there is some. \
              The block is written by hand and nothing else tells a reader that a line of it \
              has gone stale. Every tracked top-level file and directory is named, by itself or \
              by a path under it; below the top level the block is a tour, not a listing.",
    reads: &[Input::Readme, Input::Layout],
    check,
};

const README: &str = "README.md";

/// The directory open work lives in only while there is some.
const PLAN: &str = "docs/plan/";

fn finding(message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        file: README.to_owned(),
        line: None,
        message,
    }
}

/// The paths of the Project Layout block, in order: the first word of each line of the first
/// fence under `## Project Layout`, but blank lines and comments.
pub fn listed(readme: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut in_section = false;
    let mut in_fence = false;
    for line in read_lines(readme.as_bytes()) {
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

fn check(inputs: &Inputs) -> Vec<Finding> {
    let Some(readme) = &inputs.readme else {
        return vec![finding(
            "not found — the Project Layout block cannot be checked".to_owned(),
        )];
    };
    let paths = listed(readme);
    if paths.is_empty() {
        return vec![finding(
            "no paths found under '## Project Layout' — the inventory is unverifiable".to_owned(),
        )];
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
            found.push(finding(format!("Project Layout lists '{path}', {message}")));
        }
    }
    for top in &inputs.layout.top_level {
        let covered = paths
            .iter()
            .any(|path| path == top || path.starts_with(&format!("{top}/")));
        if !covered {
            found.push(finding(format!(
                "Project Layout does not name '{top}', a top-level part of this repository"
            )));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAYOUT: &str = "# P\n\n## Project Layout\n\nIntro.\n\n```text\nREADME.md      the entry\nsrc/           the code\n\n# a comment\ndocs/plan/     open work\nscripts/*.sh   the checks\n```\n\n## Next\n";

    fn messages(readme: &str, found: &[(&str, bool)], top: &[&str]) -> Vec<String> {
        let inputs = Inputs {
            readme: Some(readme.to_owned()),
            layout: super::super::Layout {
                found: found.iter().map(|(p, f)| ((*p).to_owned(), *f)).collect(),
                top_level: top.iter().map(|t| (*t).to_owned()).collect(),
            },
            ..Inputs::default()
        };
        check(&inputs).into_iter().map(|f| f.message).collect()
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
    fn no_readme_and_an_empty_block_fail_alone() {
        let inputs = Inputs::default();
        assert_eq!(
            check(&inputs)[0].message,
            "not found — the Project Layout block cannot be checked"
        );
        assert_eq!(
            messages("# P\n", &[], &["src"]),
            ["no paths found under '## Project Layout' — the inventory is unverifiable"]
        );
    }
}
