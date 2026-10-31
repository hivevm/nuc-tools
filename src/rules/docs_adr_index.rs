//! Check 1 of the template's `scripts/check-docs.sh` at v0.5.0: every ADR and draft is listed once
//! in the index `docs/adr/README.md`, every row names a file that exists, the status and the
//! *Applies to* the row shows are the file's, and a row sits in the table its status belongs to.

use std::collections::BTreeSet;

use super::{Inputs, Rule};
use crate::adr::{self, Dir};
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/adr-index";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Every ADR in docs/adr/ is listed once in the index docs/adr/README.md, with the \
              status and the 'Applies to' its file states, and every row links a file that \
              exists. A superseded or rejected row sits below the 'Superseded and rejected' \
              heading, an accepted or proposed one above it. The index routes a reader from a \
              change to the ADRs that bind it, so a row that misstates what its ADR governs, or \
              stands in the wrong table, misroutes them.",
    check,
};

const INDEX: &str = "docs/adr/README.md";

/// One row of the index: the file it links, its *Applies to* cell, its status emoji, its line.
struct Row {
    key: String,
    file: String,
    applies: String,
    status: Option<char>,
    line: usize,
    archive: bool,
}

fn finding(file: &str, line: Option<usize>, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        file: file.to_owned(),
        line,
        message,
    }
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let dir = &inputs.adr;
    let Some(index) = dir.get("README.md") else {
        return vec![finding(
            INDEX,
            None,
            "ADR index not found: docs/adr/README.md".to_owned(),
        )];
    };
    let rows = rows(&index.text);
    let mut found: Vec<Finding> = rows.iter().flat_map(row_findings).collect();
    let mut seen = BTreeSet::new();
    for row in &rows {
        if !seen.insert(row.file.as_str()) {
            found.push(finding(INDEX, Some(row.line), format!(
                "ADR index: lists '{}' twice — keep the row that matches the file, delete the other",
                row.file
            )));
        }
    }
    for name in adr::adr_names(dir) {
        if !seen.contains(name) {
            found.push(finding(
                INDEX,
                None,
                format!("ADR index: file '{name}' exists but is not listed in the index"),
            ));
        }
    }
    found.extend(rows.iter().flat_map(|row| file_findings(dir, row)));
    found
}

/// The rows of the index, as the bash's row pattern finds them.
fn rows(index: &str) -> Vec<Row> {
    let mut archive = false;
    let mut rows = Vec::new();
    for (i, line) in index.split('\n').enumerate() {
        if is_archive_heading(line) {
            archive = true;
            continue;
        }
        let Some((key, target)) = row_link(line) else {
            continue;
        };
        let cells: Vec<&str> = line.split('|').collect();
        rows.push(Row {
            key: key.to_owned(),
            file: target.split('#').next().unwrap_or(target).to_owned(),
            applies: adr::trim(cells.get(3).copied().unwrap_or("")).to_owned(),
            status: adr::status_emoji(line).0,
            line: i + 1,
            archive,
        });
    }
    rows
}

/// Whether the line is the heading `#… Superseded and rejected`.
fn is_archive_heading(line: &str) -> bool {
    let hashes = line.len() - line.trim_start_matches('#').len();
    let rest = &line[hashes..];
    let words = rest.trim_start_matches(adr::is_space);
    hashes > 0
        && words.len() < rest.len()
        && words.trim_end_matches(adr::is_space) == "Superseded and rejected"
}

/// The key and the link target of a row `| [<key>](<target>)`, the key four digits or a draft.
fn row_link(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix('|')?.trim_start_matches(adr::is_space);
    let rest = rest.strip_prefix('[')?;
    let close = rest.find(']')?;
    let key = &rest[..close];
    let is_number = key.len() == 4 && key.bytes().all(|b| b.is_ascii_digit());
    let is_draft = key.len() > "draft-".len() && key.starts_with("draft-");
    if !is_number && !is_draft {
        return None;
    }
    let target = rest[close + 1..].strip_prefix('(')?;
    let end = target.find(')')?;
    (end > 0).then(|| (key, &target[..end]))
}

/// What the row itself gets wrong: the file it links, and the table it sits in.
fn row_findings(row: &Row) -> Vec<Finding> {
    let mut found = Vec::new();
    let line = Some(row.line);
    let key = &row.key;
    if key.starts_with("draft-") {
        if row.file != format!("{key}.md") {
            found.push(finding(
                INDEX,
                line,
                format!(
                    "ADR index: row for {key} links to '{}', not to '{key}.md'",
                    row.file
                ),
            ));
        }
    } else if !row.file.starts_with(&format!("{key}-")) {
        found.push(finding(
            INDEX,
            line,
            format!(
                "ADR index: row for {key} links to '{}', which does not start with '{key}-'",
                row.file
            ),
        ));
    }
    match row.status {
        Some(e @ ('⚪' | '🔴')) if !row.archive => found.push(finding(INDEX, line, format!(
            "ADR index: row for {key} is {e} but sits above the 'Superseded and rejected' heading — move it there, a session reads the first table as binding"
        ))),
        Some(e @ ('🟢' | '🟡')) if row.archive => found.push(finding(INDEX, line, format!(
            "ADR index: row for {key} is {e} but sits below the 'Superseded and rejected' heading — move it up, a binding decision there is never read"
        ))),
        _ => {}
    }
    found
}

/// What the row and its file disagree on: the file's existence, status, and *Applies to*.
fn file_findings(dir: &Dir, row: &Row) -> Vec<Finding> {
    let Some(file) = dir.get(&row.file) else {
        return vec![finding(
            INDEX,
            Some(row.line),
            format!(
                "ADR index: lists '{}', but no such ADR file exists",
                row.file
            ),
        )];
    };
    let header = adr::header(&file.text, None);
    let path = format!("docs/adr/{}", row.file);
    let mut found = Vec::new();
    match adr::status_emoji(header.field("Status").unwrap_or("")).0 {
        None => found.push(finding(
            &path,
            None,
            format!(
                "{}: no '**Status:**' line with a status emoji found",
                row.file
            ),
        )),
        Some(e) if Some(e) != row.status => found.push(finding(
            INDEX,
            Some(row.line),
            format!(
                "ADR index: status for '{}' is {} in the index but {e} in the file",
                row.file,
                row.status.map_or("<none>".to_owned(), String::from)
            ),
        )),
        Some(_) => {}
    }
    let line = header.field("Applies to").unwrap_or("");
    let applies = adr::trim(
        line.rsplit_once("**Applies to:**")
            .map_or(line, |(_, rest)| rest),
    );
    if applies.is_empty() {
        found.push(finding(
            &path,
            None,
            format!(
                "{}: no '**Applies to:**' line naming what the decision constrains",
                row.file
            ),
        ));
    } else if applies != row.applies {
        let shown = if row.applies.is_empty() {
            "<none>"
        } else {
            &row.applies
        };
        found.push(finding(INDEX, Some(row.line), format!(
            "ADR index: 'Applies to' for '{}' is '{shown}' in the index but '{applies}' in the file",
            row.file
        )));
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adr::File;

    fn adr_file(status: &str, applies: &str) -> File {
        File {
            text: format!(
                "# ADR-0001: T\n\n- **Status:** {status}\n- **Applies to:** {applies}\n\n## Context\n"
            ),
            listed: true,
        }
    }

    fn dir(index: &str, files: &[(&str, File)]) -> Inputs {
        let mut adr: Dir = files
            .iter()
            .map(|(n, f)| ((*n).to_owned(), f.clone()))
            .collect();
        adr.insert(
            "README.md".to_owned(),
            File {
                text: index.to_owned(),
                listed: true,
            },
        );
        Inputs {
            adr,
            ..Inputs::default()
        }
    }

    const HEAD: &str = "| ADR | Title | Applies to | Status |\n|---|---|---|---|\n";

    fn messages(inputs: &Inputs) -> Vec<String> {
        check(inputs).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn an_index_that_matches_its_files_passes() {
        let index = format!("{HEAD}| [0001](0001-a.md) | T | every commit | 🟢 accepted |\n");
        assert_eq!(
            messages(&dir(
                &index,
                &[("0001-a.md", adr_file("🟢 accepted", "every commit"))]
            )),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_missing_index_fails() {
        assert_eq!(
            messages(&Inputs::default()),
            ["ADR index not found: docs/adr/README.md"]
        );
    }

    #[test]
    fn each_disagreement_fails_with_the_bash_message() {
        let a = ("0001-a.md", adr_file("🟢 accepted", "every commit"));
        type Case<'a> = (String, Vec<(&'a str, File)>, &'a str);
        let cases: [Case; 9] = [
            (
                HEAD.to_owned(),
                vec![a.clone()],
                "file '0001-a.md' exists but is not listed",
            ),
            (
                format!(
                    "{HEAD}| [0001](0001-a.md) | T | every commit | 🟢 |\n| [0001](0001-a.md) | T | every commit | 🟢 |\n"
                ),
                vec![a.clone()],
                "lists '0001-a.md' twice",
            ),
            (
                format!("{HEAD}| [0002](0002-b.md) | T | x | 🟢 |\n"),
                vec![],
                "lists '0002-b.md', but no such ADR file exists",
            ),
            (
                format!("{HEAD}| [0001](0001-a.md) | T | every commit | 🟡 |\n"),
                vec![a.clone()],
                "status for '0001-a.md' is 🟡 in the index but 🟢 in the file",
            ),
            (
                format!("{HEAD}| [0001](0001-a.md) | T | other | 🟢 |\n"),
                vec![a.clone()],
                "'Applies to' for '0001-a.md' is 'other' in the index but 'every commit'",
            ),
            (
                format!("{HEAD}| [0001](0002-a.md) | T | x | 🟢 |\n"),
                vec![],
                "row for 0001 links to '0002-a.md', which does not start with '0001-'",
            ),
            (
                format!("{HEAD}| [draft-x](draft-y.md) | T | x | 🟡 |\n"),
                vec![],
                "row for draft-x links to 'draft-y.md', not to 'draft-x.md'",
            ),
            (
                format!("{HEAD}| [0001](0001-a.md) | T | every commit | ⚪ |\n"),
                vec![(
                    "0001-a.md",
                    adr_file("⚪ superseded by ADR-0002", "every commit"),
                )],
                "is ⚪ but sits above",
            ),
            (
                format!(
                    "{HEAD}\n### Superseded and rejected\n\n{HEAD}| [0001](0001-a.md) | T | every commit | 🟢 |\n"
                ),
                vec![a.clone()],
                "is 🟢 but sits below",
            ),
        ];
        for (index, files, needle) in cases {
            let found = messages(&dir(&index, &files));
            assert!(
                found.iter().any(|m| m.contains(needle)),
                "{needle}: {found:?}"
            );
        }
    }

    #[test]
    fn a_file_without_status_or_applies_fails_at_the_file() {
        let index = format!("{HEAD}| [0001](0001-a.md) | T | x | 🟢 |\n");
        let bare = File {
            text: "# ADR-0001: T\n".to_owned(),
            listed: true,
        };
        let found = check(&dir(&index, &[("0001-a.md", bare)]));
        let files: Vec<&str> = found.iter().map(|f| f.file.as_str()).collect();
        assert_eq!(files, ["docs/adr/0001-a.md", "docs/adr/0001-a.md"]);
    }

    #[test]
    fn the_cell_is_split_at_pipes_and_trimmed_and_a_link_anchor_is_dropped() {
        let index =
            format!("{HEAD}|  [0001](0001-a.md#context) | T |  every commit\t | 🟢 accepted |\n");
        assert_eq!(
            messages(&dir(
                &index,
                &[("0001-a.md", adr_file("🟢 accepted", "every commit"))]
            )),
            Vec::<String>::new()
        );
    }
}
