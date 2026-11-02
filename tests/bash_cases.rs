//! The unit tests held to the cases of the template's bash self-tests.
//!
//! The convention "A port copies the bash, quirks included" of `docs/CONVENTIONS.md` has a rule's
//! unit tests copy the cases of the bash self-test of its check. `tests/bash_cases.tsv`, the list
//! kept beside this test, names each case of `scripts/test-check-docs.sh` and
//! `scripts/test-check-traceability.sh` as its `expect` line writes it, with the rule whose part
//! of the bash it tries, or `-` and the reason no rule of `nuc` decides it. A unit test under
//! `src/` that copies a case says so in a comment `// bash: <name>`. The test fails on a case the
//! list leaves out or a row whose case the scripts lack, on a case of a rule no such comment
//! names, and on a comment that names no case of the list. A name the script assembles, as the
//! one of a case inside a loop, is listed and named as the script writes it.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const LIST: &str = include_str!("bash_cases.tsv");

/// The self-tests whose cases the list holds, under `scripts/`.
const SCRIPTS: [&str; 2] = ["test-check-docs.sh", "test-check-traceability.sh"];

/// The comment that names the case a unit test copies.
const NAMES: &str = "// bash: ";

/// A row of the list: the script, the case, and its rule, `None` for a case no rule decides.
#[derive(Debug, PartialEq, Eq)]
struct Row {
    script: String,
    case: String,
    rule: Option<String>,
}

/// The name of each case of a self-test, in order: the text between the quotes of each
/// `expect pass "<name>"` or `expect fail "<name>"` on a line.
fn cases(script: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in script.lines() {
        for open in ["expect pass \"", "expect fail \""] {
            let Some(at) = line.find(open) else {
                continue;
            };
            let rest = &line[at + open.len()..];
            if let Some(end) = rest.find('"') {
                found.push(rest[..end].to_owned());
            }
        }
    }
    found
}

/// The rows of the list, without blank lines and `#` comments, or what makes a line no row.
fn rows(list: &str) -> Result<Vec<Row>, String> {
    let mut rows = Vec::new();
    for (i, line) in list.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let row = match fields.as_slice() {
            [script, case, "-", reason] if !reason.trim().is_empty() => Row {
                script: (*script).to_owned(),
                case: (*case).to_owned(),
                rule: None,
            },
            [script, case, rule] if *rule != "-" => Row {
                script: (*script).to_owned(),
                case: (*case).to_owned(),
                rule: Some((*rule).to_owned()),
            },
            _ => {
                return Err(format!(
                    "line {}: not '<script>\\t<case>\\t<rule>' or '<script>\\t<case>\\t-\\t<reason>': {line}",
                    i + 1
                ));
            }
        };
        rows.push(row);
    }
    Ok(rows)
}

/// The case each `// bash: <name>` comment of the source names, with its line number.
fn named(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let name = line.trim_start().strip_prefix(NAMES)?;
            Some((i + 1, name.trim_end().to_owned()))
        })
        .collect()
}

/// What keeps the list, the scripts, and the comments apart: each script given as its file name
/// and its cases, each comment as where it stands and the case it names.
fn problems(
    scripts: &[(&str, Vec<String>)],
    rows: &[Row],
    comments: &[(String, String)],
    rule_ids: &BTreeSet<String>,
) -> Vec<String> {
    let mut found = Vec::new();
    let listed: BTreeSet<(&str, &str)> = rows
        .iter()
        .map(|r| (r.script.as_str(), r.case.as_str()))
        .collect();
    let mut seen = BTreeSet::new();
    for row in rows {
        if !seen.insert((row.script.as_str(), row.case.as_str())) {
            found.push(format!("{}: '{}' is listed twice", row.script, row.case));
        }
    }
    let mut written = BTreeSet::new();
    for (script, cases) in scripts {
        for case in cases {
            written.insert((*script, case.as_str()));
            if !listed.contains(&(*script, case.as_str())) {
                found.push(format!("{script}: the case '{case}' is not in the list"));
            }
        }
    }
    let commented: BTreeSet<&str> = comments.iter().map(|(_, n)| n.as_str()).collect();
    for row in rows {
        if !written.contains(&(row.script.as_str(), row.case.as_str())) {
            found.push(format!(
                "{}: the list names '{}', which the script lacks",
                row.script, row.case
            ));
        }
        let Some(rule) = &row.rule else {
            continue;
        };
        if !rule_ids.contains(rule) {
            found.push(format!(
                "{}: '{}' is listed for '{rule}', which is no rule of nuc",
                row.script, row.case
            ));
        } else if !commented.contains(row.case.as_str()) {
            found.push(format!(
                "{rule}: no unit test names the case '{}' of {} in a '{NAMES}<name>' comment",
                row.case, row.script
            ));
        }
    }
    let names: BTreeSet<&str> = rows.iter().map(|r| r.case.as_str()).collect();
    for (at, name) in comments {
        if !names.contains(name.as_str()) {
            found.push(format!("{at}: names '{name}', no case of the list"));
        }
    }
    found
}

/// Every `.rs` file under the directory.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("the directory is readable") {
        let path = entry.expect("the entry is readable").path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The `RULE_ID` each module of `src/rules/` declares.
fn rule_ids(root: &Path) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let mut files = Vec::new();
    sources(&root.join("src/rules"), &mut files);
    for file in files {
        let text = fs::read_to_string(&file).expect("the source is readable");
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("pub const RULE_ID: &str = \"")
                && let Some(id) = rest.strip_suffix("\";")
            {
                ids.insert(id.to_owned());
            }
        }
    }
    ids
}

#[test]
fn each_case_of_a_rule_is_named_by_a_unit_test_that_copies_it() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let scripts: Vec<(&str, Vec<String>)> = SCRIPTS
        .iter()
        .map(|name| {
            let text = fs::read_to_string(root.join("scripts").join(name))
                .expect("the self-test is readable");
            (*name, cases(&text))
        })
        .collect();
    let rows = rows(LIST).expect("the list is well formed");
    let mut files = Vec::new();
    sources(&root.join("src"), &mut files);
    files.sort();
    let mut comments = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).expect("the source is readable");
        let shown = file
            .strip_prefix(root)
            .unwrap_or(file)
            .display()
            .to_string();
        for (line, name) in named(&text) {
            comments.push((format!("{shown}:{line}"), name));
        }
    }
    let found = problems(&scripts, &rows, &comments, &rule_ids(root));
    assert!(found.is_empty(), "{}", found.join("\n"));
}

const SCRIPT: &str = "expect() {\n  local want=\"$1\"\n}\n\
                      expect pass \"a baseline\" \"$d\"\n\
                      GITHUB_BASE_REF=\"\" expect fail \"a case of ${x}\" \"$d\" \"needle\"\n\
                      # no case here\n";

fn row(case: &str, rule: Option<&str>) -> Row {
    Row {
        script: "t.sh".to_owned(),
        case: case.to_owned(),
        rule: rule.map(str::to_owned),
    }
}

fn ids() -> BTreeSet<String> {
    ["docs/a".to_owned()].into_iter().collect()
}

#[test]
fn a_case_name_is_read_as_the_expect_line_writes_it() {
    assert_eq!(cases(SCRIPT), ["a baseline", "a case of ${x}"]);
}

#[test]
fn the_list_reads_a_rule_or_a_reason_and_refuses_anything_else() {
    let list = "# a comment\n\nt.sh\ta\tdocs/a\nt.sh\tb\t-\twhy\n";
    assert_eq!(
        rows(list),
        Ok(vec![row("a", Some("docs/a")), row("b", None)])
    );
    assert!(rows("t.sh\tb\t-\n").is_err());
    assert!(rows("t.sh\tb\n").is_err());
    assert!(rows("t.sh\tb\tdocs/a\twhy\n").is_err());
}

#[test]
fn a_comment_names_its_case_at_its_line() {
    let source = "fn f() {\n    // bash: a baseline\n    /// bash: no comment of this kind\n}\n";
    assert_eq!(named(source), [(2, "a baseline".to_owned())]);
}

#[test]
fn a_case_missing_from_the_list_or_from_the_unit_tests_is_found() {
    let scripts = [("t.sh", cases(SCRIPT))];
    let comment = |n: &str| ("src/x.rs:1".to_owned(), n.to_owned());
    let all = [
        row("a baseline", None),
        row("a case of ${x}", Some("docs/a")),
    ];
    assert_eq!(
        problems(&scripts, &all, &[comment("a case of ${x}")], &ids()),
        Vec::<String>::new()
    );
    assert_eq!(
        problems(&scripts, &all[..1], &[comment("a case of ${x}")], &ids()),
        [
            "t.sh: the case 'a case of ${x}' is not in the list",
            "src/x.rs:1: names 'a case of ${x}', no case of the list",
        ]
    );
    assert_eq!(
        problems(&scripts, &all, &[], &ids()),
        [
            "docs/a: no unit test names the case 'a case of ${x}' of t.sh in a '// bash: <name>' comment"
        ]
    );
}

#[test]
fn a_row_the_scripts_lack_a_rule_nuc_lacks_and_a_row_twice_are_found() {
    let scripts = [("t.sh", cases(SCRIPT))];
    let rows = [
        row("a baseline", None),
        row("a baseline", None),
        row("a case of ${x}", Some("docs/b")),
        row("gone", None),
    ];
    assert_eq!(
        problems(&scripts, &rows, &[], &ids()),
        [
            "t.sh: 'a baseline' is listed twice",
            "t.sh: 'a case of ${x}' is listed for 'docs/b', which is no rule of nuc",
            "t.sh: the list names 'gone', which the script lacks",
        ]
    );
}
