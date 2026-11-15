//! The check `docs/adr-identifiers` of the template's `scripts/check-docs.sh` from v0.9.0 on: an
//! ADR is named `<CAT>-<slug>`, its category fixed or in the category list of the index, its
//! heading opens with its identifier, no numbered ADR file is left, and no ADR takes the
//! identifier of one the first-parent history deleted. A tree of an earlier release has no such
//! check, and this rule reports nothing there.

use super::{CATEGORY_IDENTIFIERS, Input, Inputs, Rule};
use crate::adr;
use crate::finding::Finding;
use std::collections::BTreeMap;

pub const RULE_ID: &str = "docs/adr-identifiers";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &[
        "numbered", "form", "category", "heading", "history", "taken",
    ],
    explain: "Each ADR is named '<CAT>-<slug>': two to five capital letters, a hyphen, and two \
              to five lower case words or numbers joined by hyphens. Its category is NUC, SEC, \
              OPS, or API, or one the index lists under Categories; its heading opens with \
              '# <CAT>-<slug>:'. No numbered ADR file is left from before the migration, and no \
              ADR takes the identifier of one the first-parent history deleted, since commits and \
              code cite an identifier for good: a successor takes a new slug.",
    reads: &[Input::Adr, Input::Release, Input::History],
    check: |inputs| check(inputs).into(),
};

/// What the first-parent history of `HEAD` says about the deleted ADR files: the short hash of
/// the latest commit that deleted each name, or why git could not read it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct History {
    pub deleted: BTreeMap<String, String>,
    pub error: Option<String>,
}

/// The arguments of the `git log` the bash runs, `HEAD` and the pathspec of the record's files
/// included.
pub const LOG: [&str; 10] = [
    "log",
    "--first-parent",
    "--diff-merges=first-parent",
    "--no-renames",
    "--diff-filter=D",
    "--name-only",
    "--format=commit %h",
    "HEAD",
    "--",
    ":(glob)docs/adr/[A-Za-z]*-*.md",
];

/// The deleted names of the log's output, each with the commit of its latest delete, as the bash
/// reads the lines: `commit <hash>` names the commit of the names below it.
pub fn deleted(log: &str) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    let mut commit = "";
    for line in log.lines() {
        if let Some(hash) = line.strip_prefix("commit ") {
            commit = hash;
        } else if !line.is_empty() {
            let name = line.rsplit('/').next().unwrap_or(line);
            found
                .entry(name.to_owned())
                .or_insert_with(|| commit.to_owned());
        }
    }
    found
}

fn finding(reason: &'static str, file: &str, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        reason,
        file: format!("docs/adr/{file}"),
        line: None,
        message,
    }
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    if !inputs.carries_at_least(CATEGORY_IDENTIFIERS) {
        return Vec::new();
    }
    let mut found: Vec<Finding> = inputs
        .adr
        .keys()
        .filter(|name| adr::is_numbered_name(name))
        .map(|name| {
            finding(
                "numbered",
                name,
                format!(
                    "{name}: a numbered ADR, a form no check reads — migrate the record with the \
                     normalize skill of a v0.9 release, which maps it to '<CAT>-<slug>'"
                ),
            )
        })
        .collect();
    let categories = adr::categories(inputs.adr.get("README.md").map(Vec::as_slice));
    let names = adr::category_names(&inputs.adr);
    for name in &names {
        let id = name.strip_suffix(".md").unwrap_or(name);
        if !adr::is_identifier(id) {
            found.push(finding(
                "form",
                name,
                format!(
                    "{name}: the identifier is not '<CAT>-<slug>': two to five capital letters, a \
                     hyphen, and two to five lower case words or numbers joined by hyphens"
                ),
            ));
            continue;
        }
        let category = id.split('-').next().unwrap_or(id);
        if !categories.iter().any(|c| c == category) {
            found.push(finding(
                "category",
                name,
                format!(
                    "{name}: the category {category} is neither fixed nor in the category list of \
                     docs/adr/README.md"
                ),
            ));
        }
        let heading = adr::title(&inputs.adr[*name]);
        if !heading.is_some_and(|h| h.starts_with(format!("# {id}:").as_bytes())) {
            let shown = heading.map_or("<none>".to_owned(), adr::show);
            found.push(finding(
                "heading",
                name,
                format!("{name}: heading says '{shown}', but the filename says {id}"),
            ));
        }
    }
    if let Some(error) = &inputs.history.error {
        found.push(Finding {
            rule: RULE_ID,
            reason: "history",
            file: "docs/adr".to_owned(),
            line: None,
            message: format!(
                "ADR identifiers: git log cannot read the deleted ADRs (git 2.31 or later is \
                 needed): {error}"
            ),
        });
    }
    for name in &names {
        if let Some(commit) = inputs.history.deleted.get(*name) {
            let id = name.strip_suffix(".md").unwrap_or(name);
            found.push(finding(
                "taken",
                name,
                format!(
                    "{name}: the identifier {id} was taken by a deleted ADR, deleted in {commit} — a \
                     successor takes a new slug"
                ),
            ));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Version;
    use crate::testing::cat_id;

    fn dir(files: &[(String, String)]) -> adr::Dir {
        files
            .iter()
            .map(|(n, t)| (n.clone(), t.as_bytes().to_vec()))
            .collect()
    }

    /// The file of the identifier and its text, the heading `# <id>: x` unless given.
    fn adr(category: &str, slug: &str, text: Option<&str>) -> (String, String) {
        let id = cat_id(category, slug);
        let text = text.map_or_else(|| format!("# {id}: x\n"), str::to_owned);
        (format!("{id}.md"), text)
    }

    fn reasons(inputs: &Inputs) -> Vec<(&'static str, String)> {
        check(inputs)
            .into_iter()
            .map(|f| (f.reason, f.file))
            .collect()
    }

    #[test]
    fn each_cause_is_reported_with_its_reason_and_file() {
        let index = "# ADRs\n\n### Categories\n\n- **UX** — ours.\n\n### Binding\n";
        let inputs = Inputs {
            adr: dir(&[
                ("README.md".to_owned(), index.to_owned()),
                ("0001-old.md".to_owned(), "# old\n".to_owned()),
                adr("SEC", "Host_socket", None),
                adr("XYZ", "some-thing", None),
                adr("UX", "dark-mode", None),
                adr(
                    "SEC",
                    "a-rule",
                    Some(&format!("x\n# {}: decision\n", cat_id("SEC", "b-rule"))),
                ),
                adr("OPS", "old-rule", None),
            ]),
            history: History {
                deleted: [(
                    format!("{}.md", cat_id("OPS", "old-rule")),
                    "abc1234".to_owned(),
                )]
                .into(),
                error: None,
            },
            ..Inputs::default()
        };
        assert_eq!(
            reasons(&inputs),
            [
                ("numbered", "docs/adr/0001-old.md".to_owned()),
                (
                    "form",
                    format!("docs/adr/{}.md", cat_id("SEC", "Host_socket"))
                ),
                (
                    "heading",
                    format!("docs/adr/{}.md", cat_id("SEC", "a-rule"))
                ),
                (
                    "category",
                    format!("docs/adr/{}.md", cat_id("XYZ", "some-thing"))
                ),
                (
                    "taken",
                    format!("docs/adr/{}.md", cat_id("OPS", "old-rule"))
                ),
            ]
        );
    }

    #[test]
    fn a_tree_before_v0_9_0_has_no_such_check() {
        let inputs = Inputs {
            adr: dir(&[("0001-old.md".to_owned(), "# old\n".to_owned())]),
            release: Some(Version(0, 8, 1)),
            ..Inputs::default()
        };
        assert!(check(&inputs).is_empty());
    }

    #[test]
    fn the_latest_delete_of_a_name_is_the_one_named() {
        let log =
            "commit new1\ndocs/adr/A-b-c.md\n\ncommit old2\ndocs/adr/A-b-c.md\ndocs/adr/B-c-d.md\n";
        let found = deleted(log);
        assert_eq!(found["A-b-c.md"], "new1");
        assert_eq!(found["B-c-d.md"], "old2");
    }

    #[test]
    fn a_history_git_cannot_read_is_reported() {
        let inputs = Inputs {
            history: History {
                deleted: BTreeMap::new(),
                error: Some("fatal: unrecognized argument".to_owned()),
            },
            ..Inputs::default()
        };
        assert_eq!(reasons(&inputs), [("history", "docs/adr".to_owned())]);
    }
}
