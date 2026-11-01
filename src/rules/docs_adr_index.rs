//! Check 1 of the template's `scripts/check-docs.sh` at v0.5.0: every ADR and draft is listed once
//! in the index `docs/adr/README.md`, every row names a file that exists, the status and the
//! *Applies to* the row shows are the file's, and a row sits in the table its status belongs to.

use std::collections::BTreeSet;

use super::{Input, Inputs, Rule};
use crate::adr::{self, show};
use crate::bash;
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
    reads: &[Input::Adr],
    check: |inputs| check(inputs).into(),
};

const INDEX: &str = "docs/adr/README.md";

/// One row of the index: the file it links, its *Applies to* cell, its status emoji, its line.
struct Row {
    key: String,
    file: String,
    applies: Vec<u8>,
    status: Option<&'static str>,
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

/// The file each row of the index links, as the shell resolves them for the rule.
pub fn targets(index: &[u8]) -> Vec<String> {
    rows(index).into_iter().map(|row| row.file).collect()
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let Some(index) = inputs.adr.get("README.md") else {
        return vec![finding(
            INDEX,
            None,
            "ADR index not found: docs/adr/README.md".to_owned(),
        )];
    };
    let rows = rows(index);
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
    for name in adr::adr_names(&inputs.adr) {
        if !seen.contains(name) {
            found.push(finding(
                INDEX,
                None,
                format!("ADR index: file '{name}' exists but is not listed in the index"),
            ));
        }
    }
    for row in &rows {
        let target = inputs.adr_targets.get(&row.file).and_then(Option::as_deref);
        found.extend(file_findings(target, row));
    }
    found
}

/// The rows of the index, as the bash's row pattern finds them in the lines `read` reads.
fn rows(index: &[u8]) -> Vec<Row> {
    let mut archive = false;
    let mut rows = Vec::new();
    for (i, line) in bash::read_lines(index).into_iter().enumerate() {
        if is_archive_heading(line) {
            archive = true;
            continue;
        }
        let Some((key, target)) = row_link(line) else {
            continue;
        };
        let cells: Vec<&[u8]> = line.split(|&b| b == b'|').collect();
        let file = target.split(|&b| b == b'#').next().unwrap_or(target);
        rows.push(Row {
            key: show(key),
            file: show(file),
            applies: bash::trim(cells.get(3).copied().unwrap_or(b"")).to_vec(),
            status: adr::status_emoji(line).0,
            line: i + 1,
            archive,
        });
    }
    rows
}

/// Whether the line is the heading `#… Superseded and rejected`.
fn is_archive_heading(line: &[u8]) -> bool {
    let hashes = line.iter().take_while(|&&b| b == b'#').count();
    let rest = &line[hashes..];
    let spaces = rest.iter().take_while(|&&b| bash::is_space(b)).count();
    let words = &rest[spaces..];
    let end = words
        .iter()
        .rposition(|&b| !bash::is_space(b))
        .map_or(0, |i| i + 1);
    hashes > 0 && spaces > 0 && &words[..end] == b"Superseded and rejected"
}

/// The key and the link target of a row `| [<key>](<target>)`, the key four digits or a draft.
fn row_link(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let rest = line.strip_prefix(b"|")?;
    let rest = &rest[rest.iter().take_while(|&&b| bash::is_space(b)).count()..];
    let rest = rest.strip_prefix(b"[")?;
    let close = rest.iter().position(|&b| b == b']')?;
    let key = &rest[..close];
    let is_number = key.len() == 4 && key.iter().all(u8::is_ascii_digit);
    let is_draft = key.len() > b"draft-".len() && key.starts_with(b"draft-");
    if !is_number && !is_draft {
        return None;
    }
    let target = rest[close + 1..].strip_prefix(b"(")?;
    let end = target.iter().position(|&b| b == b')')?;
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
        Some(e @ ("⚪" | "🔴")) if !row.archive => found.push(finding(INDEX, line, format!(
            "ADR index: row for {key} is {e} but sits above the 'Superseded and rejected' heading — move it there, a session reads the first table as binding"
        ))),
        Some(e @ ("🟢" | "🟡")) if row.archive => found.push(finding(INDEX, line, format!(
            "ADR index: row for {key} is {e} but sits below the 'Superseded and rejected' heading — move it up, a binding decision there is never read"
        ))),
        _ => {}
    }
    found
}

/// What the row and the file it links disagree on: the file's existence, status, and *Applies
/// to*. A file with a NUL byte is binary to grep, which then prints no field.
fn file_findings(file: Option<&[u8]>, row: &Row) -> Vec<Finding> {
    let Some(text) = file else {
        return vec![finding(
            INDEX,
            Some(row.line),
            format!(
                "ADR index: lists '{}', but no such ADR file exists",
                row.file
            ),
        )];
    };
    let header = if text.contains(&0) {
        adr::Header::default()
    } else {
        adr::header(text, None)
    };
    let path = format!("docs/adr/{}", row.file);
    let mut found = Vec::new();
    match adr::status_emoji(header.field("Status").unwrap_or(b"")).0 {
        None => found.push(finding(
            &path,
            None,
            "no '**Status:**' line with a status emoji found".to_owned(),
        )),
        Some(e) if Some(e) != row.status => found.push(finding(
            INDEX,
            Some(row.line),
            format!(
                "ADR index: status for '{}' is {} in the index but {e} in the file",
                row.file,
                row.status.unwrap_or("<none>")
            ),
        )),
        Some(_) => {}
    }
    let applies = bash::trim(after_last_label(header.field("Applies to").unwrap_or(b"")));
    if applies.is_empty() {
        found.push(finding(
            &path,
            None,
            "no '**Applies to:**' line naming what the decision constrains".to_owned(),
        ));
    } else if applies != row.applies.as_slice() {
        let shown = if row.applies.is_empty() {
            "<none>".to_owned()
        } else {
            show(&row.applies)
        };
        found.push(finding(
            INDEX,
            Some(row.line),
            format!(
                "ADR index: 'Applies to' for '{}' is '{shown}' in the index but '{}' in the file",
                row.file,
                show(applies)
            ),
        ));
    }
    found
}

/// The bytes after the last `**Applies to:**` of the line, as `${line##*\*\*Applies to:\*\*}`.
fn after_last_label(line: &[u8]) -> &[u8] {
    let label: &[u8] = b"**Applies to:**";
    (0..=line.len().saturating_sub(label.len()))
        .rev()
        .find(|&i| line[i..].starts_with(label))
        .map_or(line, |i| &line[i + label.len()..])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adr_file(status: &str, applies: &str) -> Vec<u8> {
        format!(
            "# ADR-0001: T\n\n- **Status:** {status}\n- **Applies to:** {applies}\n\n## Context\n"
        )
        .into_bytes()
    }

    /// The inputs of an index and the ADR files, each a target too, as the shell resolves them.
    fn dir(index: &str, files: &[(&str, Vec<u8>)]) -> Inputs {
        let mut inputs = Inputs::default();
        for (name, text) in files {
            inputs.adr.insert((*name).to_owned(), text.clone());
        }
        inputs
            .adr
            .insert("README.md".to_owned(), index.as_bytes().to_vec());
        for target in targets(index.as_bytes()) {
            let text = files
                .iter()
                .find(|(n, _)| *n == target)
                .map(|(_, t)| t.clone());
            inputs.adr_targets.insert(target, text);
        }
        inputs
    }

    const HEAD: &str = "| ADR | Title | Applies to | Status |\n|---|---|---|---|\n";

    fn messages(inputs: &Inputs) -> Vec<String> {
        check(inputs).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn an_index_that_matches_its_files_passes() {
        let index = format!("{HEAD}| [0001](0001-a.md) | T | every commit | 🟢 accepted |\n");
        let files = [("0001-a.md", adr_file("🟢 accepted", "every commit"))];
        assert_eq!(messages(&dir(&index, &files)), Vec::<String>::new());
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
        type Case<'a> = (String, Vec<(&'a str, Vec<u8>)>, &'a str);
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
        let found = check(&dir(&index, &[("0001-a.md", b"# ADR-0001: T\n".to_vec())]));
        let files: Vec<&str> = found.iter().map(|f| f.file.as_str()).collect();
        assert_eq!(files, ["docs/adr/0001-a.md", "docs/adr/0001-a.md"]);
    }

    #[test]
    fn the_cell_is_split_at_pipes_and_trimmed_and_a_link_anchor_is_dropped() {
        let index =
            format!("{HEAD}|  [0001](0001-a.md#context) | T |  every commit\t | 🟢 accepted |\n");
        let files = [("0001-a.md", adr_file("🟢 accepted", "every commit"))];
        assert_eq!(messages(&dir(&index, &files)), Vec::<String>::new());
    }

    #[test]
    fn an_unterminated_last_row_is_not_read_as_bash_s_read_drops_it() {
        let index = format!("{HEAD}| [0001](0001-a.md) | T | every commit | 🟢 accepted |");
        let files = [("0001-a.md", adr_file("🟢 accepted", "every commit"))];
        assert_eq!(
            messages(&dir(&index, &files)),
            ["ADR index: file '0001-a.md' exists but is not listed in the index"]
        );
    }

    #[test]
    fn bytes_that_are_no_utf8_compare_as_themselves() {
        let index = [
            HEAD.as_bytes(),
            b"| [0001](0001-a.md) | T | caf\xe9 | \xf0\x9f\x9f\xa2 |\n",
        ]
        .concat();
        let file = b"- **Status:** \xf0\x9f\x9f\xa2 accepted\n- **Applies to:** caf\xe8\n".to_vec();
        let mut inputs = Inputs::default();
        inputs.adr.insert("README.md".to_owned(), index);
        inputs.adr.insert("0001-a.md".to_owned(), file.clone());
        inputs
            .adr_targets
            .insert("0001-a.md".to_owned(), Some(file));
        let found = messages(&inputs);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].starts_with("ADR index: 'Applies to' for '0001-a.md'"));
    }

    #[test]
    fn a_target_with_a_nul_byte_is_binary_to_grep_and_names_no_field() {
        let index = format!("{HEAD}| [0001](0001-a.txt) | T | x | 🟢 |\n");
        let mut inputs = dir(&index, &[]);
        let binary = b"- **Status:** \xf0\x9f\x9f\xa2\n\0".to_vec();
        inputs
            .adr_targets
            .insert("0001-a.txt".to_owned(), Some(binary));
        let found = messages(&inputs);
        assert!(
            found
                .iter()
                .any(|m| m == "no '**Status:**' line with a status emoji found"),
            "{found:?}"
        );
    }
}
