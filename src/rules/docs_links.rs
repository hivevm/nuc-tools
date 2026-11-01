//! Check 3 of the template's `scripts/check-docs.sh` at v0.5.0: every relative Markdown link
//! resolves to a file or directory that exists, and its anchor to a heading of the Markdown file
//! or to lines of any file it names. Links in fences, to the web, and with the template's
//! placeholder `NNNN` are not claims about files; superseded and rejected ADRs are not read.

use super::{Input, Inputs, Rule};
use crate::adr::{self, show};
use crate::finding::Finding;
use crate::markdown::{self, Target};

pub const RULE_ID: &str = "docs/links";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Every relative Markdown link in a tracked Markdown file resolves to a file or \
              directory that exists, and its '#anchor' to a heading of a Markdown file, or to \
              its lines as 'Lnn' or 'Lnn-Lmm' in any file. Links in fenced code are \
              illustrations, a target with the template's 'NNNN' is a placeholder, and \
              superseded and rejected ADRs describe a tree that no longer exists.",
    reads: &[Input::Adr, Input::TextFiles, Input::LinkTargets],
    check: |inputs| check(inputs).into(),
};

/// A path a relative link names, as the bash tests it: whether it exists, as `-e` finds it, and
/// the bytes of a file an anchor of the link looks into.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolved {
    pub exists: bool,
    pub file: Option<Vec<u8>>,
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let inactive = adr::inactive_paths(&inputs.adr);
    let mut found = Vec::new();
    for (file, text) in &inputs.text_files {
        if !file.ends_with(".md") || inactive.contains(file.as_str()) {
            continue;
        }
        for target in markdown::relative_targets(file, text) {
            found.extend(target_finding(inputs, file, text, &target));
        }
    }
    found
}

/// The finding of one link, if its path does not exist or its anchor names nothing there.
fn target_finding(inputs: &Inputs, file: &str, text: &[u8], target: &Target) -> Option<Finding> {
    let finding = |message: String| Finding {
        rule: RULE_ID,
        file: file.to_owned(),
        line: Some(target.line),
        message,
    };
    let (path, content) = match &target.path {
        None => (file.to_owned(), Some(text)),
        Some(path) => {
            let resolved = inputs.link_targets.get(path)?;
            if !resolved.exists {
                return Some(finding(format!(
                    "broken relative link -> '{}'",
                    show(&target.target)
                )));
            }
            (path.clone(), resolved.file.as_deref())
        }
    };
    let fragment = target.fragment.as_deref().filter(|f| !f.is_empty())?;
    let content = content?;
    let resolves = markdown::line_anchor_resolves(fragment, content).unwrap_or_else(|| {
        !path.ends_with(".md")
            || markdown::heading_slugs(content)
                .iter()
                .any(|slug| slug == fragment)
    });
    (!resolves).then(|| {
        finding(format!(
            "link '{}' has no matching anchor '#{}' in {path}",
            show(&target.target),
            show(fragment)
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(files: &[(&str, &str)], targets: &[(&str, bool, Option<&str>)]) -> Inputs {
        let mut inputs = Inputs::default();
        for (path, text) in files {
            inputs
                .text_files
                .insert((*path).to_owned(), text.as_bytes().to_vec());
        }
        for (path, exists, file) in targets {
            let resolved = Resolved {
                exists: *exists,
                file: file.map(|t| t.as_bytes().to_vec()),
            };
            inputs.link_targets.insert((*path).to_owned(), resolved);
        }
        inputs
    }

    fn messages(inputs: &Inputs) -> Vec<String> {
        check(inputs).into_iter().map(|f| f.message).collect()
    }

    const AGENTS: &str = "# Agents\n\n## 3. ADR rules\n\n## Bar & Done ##\n\n## Last";

    #[test]
    fn links_that_resolve_pass() {
        let readme = "See [a](AGENTS.md#3-adr-rules), [b](AGENTS.md#bar--done), [c](AGENTS.md#last), [d](docs/), [e](#top), [f](AGENTS.md#L2-L4), [g](https://x.invalid/y), [h](NNNN-x.md)\n\n# Top\n";
        let inputs = inputs(
            &[("README.md", readme)],
            &[("AGENTS.md", true, Some(AGENTS)), ("docs/", true, None)],
        );
        assert_eq!(messages(&inputs), Vec::<String>::new());
    }

    #[test]
    fn a_missing_file_and_a_missing_anchor_fail_with_the_bash_messages() {
        let readme = "See [m](missing.md) and [r](AGENTS.md#3-rules).\n";
        let inputs = inputs(
            &[("README.md", readme)],
            &[
                ("missing.md", false, None),
                ("AGENTS.md", true, Some(AGENTS)),
            ],
        );
        assert_eq!(
            messages(&inputs),
            [
                "broken relative link -> 'missing.md'",
                "link 'AGENTS.md#3-rules' has no matching anchor '#3-rules' in AGENTS.md"
            ]
        );
    }

    #[test]
    fn the_path_is_joined_to_the_file_s_directory_unresolved() {
        let text = "[x](../../AGENTS.md#nope)\n";
        let inputs = inputs(
            &[("docs/adr/0001-a.md", text)],
            &[("docs/adr/../../AGENTS.md", true, Some(AGENTS))],
        );
        assert_eq!(
            messages(&inputs),
            [
                "link '../../AGENTS.md#nope' has no matching anchor '#nope' in docs/adr/../../AGENTS.md"
            ]
        );
    }

    #[test]
    fn an_anchor_into_a_file_that_is_no_markdown_is_only_checked_as_lines() {
        let inputs = inputs(
            &[("README.md", "[s](x.sh#main) [t](x.sh#L9)\n")],
            &[("x.sh", true, Some("a\nb\n"))],
        );
        assert_eq!(
            messages(&inputs),
            ["link 'x.sh#L9' has no matching anchor '#L9' in x.sh"]
        );
    }

    #[test]
    fn links_in_fences_and_in_inactive_adrs_are_not_read() {
        let fenced = "```\n[m](missing.md)\n```\n";
        let superseded = "- **Status:** ⚪ superseded\n[m](missing.md)\n";
        let mut inputs = inputs(
            &[("README.md", fenced), ("docs/adr/0001-a.md", superseded)],
            &[("docs/adr/missing.md", false, None)],
        );
        inputs
            .adr
            .insert("0001-a.md".to_owned(), superseded.as_bytes().to_vec());
        assert_eq!(messages(&inputs), Vec::<String>::new());
    }
}
