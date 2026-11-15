//! The rules held to the cases of the template's fixture tables.
//! Verifies: TEST-template-table-cases
//!
//! `tests/tables/` holds the tables of the template's checks `nuc` ports, copied byte for byte
//! from the newest release it ports, and `tests/tables.blobs` the blob of each at that release's
//! tag. For each case, this test builds the tree its table's header and its steps describe through
//! the real git CLI, as the template's self-test of the check builds it, runs `nuc check`, and
//! compares the exit and the expected key and subjects as the header reads them. Every case that
//! does not agree stands in `tests/tables.cases`, pending on the ticket that ports it or not
//! ported (ARCH-rules-need-only-git).
//!
//! Beside what the tables describe, a case's tree gets what `nuc` needs and the bash does not: a
//! `nuc.toml`, which `.git/info/exclude` keeps from git's list of files as the parity harness does,
//! and, in the tables whose tree has no README, one naming the release `unreleased`, staged where
//! the table stages its files; and, in the record table's tree, a stand-in `scripts/check-docs.sh`
//! that holds only the rule keys of the header of the copies' release, which the bash reads from
//! its own script and `docs/adr-refs` from the tree's. `nuc.toml` turns on the case's rule, or, for
//! a case of every check and in the tables that name no rule, every rule of the table's area `nuc`
//! knows; a rule `nuc` does not know is refused, and the case does not agree. The rules run as
//! `unreleased`, the newest release `nuc` knows, which is the release of the copies once the port
//! reaches it; until then, a case that release changed stands as pending. The scripts the steps of
//! `check-shell.txt` write are stand-ins with the kind's name, and the copy of the template's
//! `scripts/lib/` that table's tree holds is left out: `nuc` reads no script's content and no file
//! below `scripts/lib/` (ARCH-rules-need-only-git). In that table `nuc` runs `shell/mode`, the one
//! rule of the area it ports. Every tree starts on the branch `main`, which the bash of the
//! traceability and the shell tables leaves to git's default and no case reads. The trees hold
//! symbolic links, so the cases run on Unix; the pins and the list are held on every platform.

// The runner and its helpers serve the cases alone, which run on Unix only; elsewhere they stand
// unused, and the pins and the judgement still compile and run.
#![cfg_attr(not(unix), allow(dead_code, unused_imports))]

mod support;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Output;

use support::{Fixture, stderr, stdout};

/// The copies, by file name under `tests/tables/`.
const TABLES: [(&str, &str); 3] = [
    (
        "check-docs-record.txt",
        include_str!("tables/check-docs-record.txt"),
    ),
    (
        "check-traceability.txt",
        include_str!("tables/check-traceability.txt"),
    ),
    ("check-shell.txt", include_str!("tables/check-shell.txt")),
];
const BLOBS: &str = include_str!("tables.blobs");
const CASES: &str = include_str!("tables.cases");

/// The lines of a list kept beside this test, comments and blank lines left out.
fn entries(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

#[test]
fn each_copy_is_the_blob_its_line_names() {
    let dir = support::sources::root().join("tests/tables");
    let mut listed = Vec::new();
    for line in entries(BLOBS) {
        let fields: Vec<&str> = line.split(' ').collect();
        let [release, blob, name] = fields[..] else {
            panic!("tests/tables.blobs: '{line}' is not '<release> <blob> <file>'");
        };
        assert!(
            release.starts_with('v') && blob.len() == 40,
            "tests/tables.blobs: '{line}'"
        );
        let output = support::git_in(&dir, &["hash-object", "--", name]);
        assert!(output.status.success(), "{name}: {}", stderr(&output));
        assert_eq!(
            stdout(&output).trim(),
            blob,
            "tests/tables/{name} is not the blob tests/tables.blobs names: copy it anew from the \
             template's tag {release}, never edit it"
        );
        listed.push(name.to_owned());
    }
    let mut copies: Vec<String> = fs::read_dir(&dir)
        .expect("tests/tables/ is read")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    copies.sort();
    listed.sort();
    assert_eq!(
        copies, listed,
        "a copy without its line, or a line without its copy"
    );
    let mut embedded: Vec<String> = TABLES.iter().map(|(name, _)| (*name).to_owned()).collect();
    embedded.sort();
    assert_eq!(
        embedded, listed,
        "this test reads another set of tables than the list names"
    );
}

/// One case of a table: its rule, if the table names one, whether it passes, the expected field,
/// its name, and its steps, the table's notation of identifiers read.
#[derive(Debug, Clone)]
struct Case {
    table: &'static str,
    rule: Option<String>,
    pass: bool,
    field: String,
    name: String,
    steps: Vec<String>,
}

/// `table_hyphens` of the template: a colon after two to five capital letters, not preceded by a
/// letter, a digit, or '_', and before a letter or a digit, is a hyphen.
fn hyphens(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = bytes.to_vec();
    for (i, &b) in bytes.iter().enumerate() {
        if b != b':' || !bytes.get(i + 1).is_some_and(u8::is_ascii_alphanumeric) {
            continue;
        }
        let caps = bytes[..i]
            .iter()
            .rev()
            .take_while(|b| b.is_ascii_uppercase())
            .count();
        let start = i - caps;
        let free =
            start == 0 || !(bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
        if (2..=5).contains(&caps) && free {
            out[i] = b'-';
        }
    }
    String::from_utf8(out).expect("UTF-8 stays UTF-8")
}

/// The cases of a table, read as the template's self-test reads them.
fn cases(table: &'static str, text: &str) -> Vec<Case> {
    let mut found = Vec::new();
    for line in text.lines() {
        let record = table == "check-docs-record.txt";
        let starts = if record {
            line.starts_with("all |")
                || line
                    .split_once(" | ")
                    .is_some_and(|(rule, _)| rule.starts_with("docs/") && !rule.contains(' '))
        } else {
            line.starts_with("pass") || line.starts_with("fail")
        };
        if !starts {
            continue;
        }
        let line = hyphens(line);
        let mut fields = line.split(" | ");
        let rule = record.then(|| fields.next().expect("a rule").to_owned());
        let want = fields.next().expect("pass or fail");
        let field = fields.next().expect("a field").to_owned();
        let name = fields.next().expect("a name").to_owned();
        let tree = fields.collect::<Vec<_>>().join(" | ");
        let steps = if tree == "-" {
            Vec::new()
        } else {
            tree.split("; ")
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect()
        };
        assert!(
            want == "pass" || want == "fail",
            "{table}: {name}: '{want}'"
        );
        assert!(
            want == "pass" || field != "-",
            "{table}: the failing case '{name}' names no key"
        );
        found.push(Case {
            table,
            rule,
            pass: want == "pass",
            field,
            name,
            steps,
        });
    }
    assert!(!found.is_empty(), "{table}: no case read");
    found
}

/// `table_datum` of the template: whether the text holds the subject with neither an ASCII letter,
/// a digit, '-', nor '_' on either side; the text after the first such place, where the next
/// subject is looked for.
fn datum<'a>(text: &'a str, subject: &str) -> Option<&'a str> {
    let inner = |b: u8| b.is_ascii_alphanumeric() || b == b'-' || b == b'_';
    let mut from = 0;
    while let Some(at) = text[from..].find(subject).map(|at| at + from) {
        let end = at + subject.len();
        let before = text[..at].bytes().last().is_some_and(inner);
        let after = text[end..].bytes().next().is_some_and(inner);
        if !before && !after {
            return Some(&text[end..]);
        }
        from = at + subject.chars().next().map_or(1, char::len_utf8);
    }
    None
}

/// `table_holds` of the template: whether the output meets the field of a case.
fn holds(output: &str, field: &str) -> bool {
    if field == "-" {
        return true;
    }
    let negated = field.starts_with('!');
    let parts: Vec<&str> = field.trim_start_matches('!').split(" @ ").collect();
    let key = parts[0];
    let well_formed = key.split_once(':').is_some_and(|(rule, reason)| {
        rule.split_once('/').is_some_and(|(area, name)| {
            !area.is_empty()
                && area.bytes().all(|b| b.is_ascii_lowercase())
                && !name.is_empty()
                && name.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
        }) && !reason.is_empty()
            && reason.bytes().all(|b| b.is_ascii_lowercase())
    }) && parts[1..].iter().all(|s| {
        !s.is_empty()
            && ((s.starts_with('"')
                && s.ends_with('"')
                && s.len() > 2
                && !s[1..s.len() - 1].contains('"'))
                || (!s.contains(' ') && !s.contains('"')))
    });
    assert!(
        well_formed,
        "fixture table: '{field}' is no key and subjects"
    );
    let found = output.lines().any(|line| {
        let Some(mut rest) = line.strip_suffix(&format!("[{key}]")) else {
            return false;
        };
        parts[1..].iter().all(|subject| match datum(rest, subject) {
            Some(after) => {
                rest = after;
                true
            }
            None => false,
        })
    });
    found != negated
}

/// What one run of a case gave.
struct Run {
    output: Output,
    agrees: bool,
}

/// The status of an ADR as the tables write it.
fn status_of(status: &str) -> String {
    match status {
        "accepted" => "🟢 accepted".to_owned(),
        "proposed" => "🟡 proposed".to_owned(),
        "rejected" => "🔴 rejected".to_owned(),
        other => format!(
            "⚪ superseded by {}",
            other
                .strip_prefix("superseded:")
                .expect("a status of the tables")
        ),
    }
}

/// The text of a step as the record table reads it: each `\n` a line break, each `<prose N>` N
/// characters of `word ` repeated.
fn table_text(text: &str) -> String {
    let mut text = text.replace("\\n", "\n");
    while let Some(at) = text.find("<prose ") {
        let end = at + text[at..].find('>').expect("a closed placeholder");
        let n: usize = text[at + 7..end].parse().expect("a number of characters");
        let prose: String = "word ".repeat(n / 5 + 1).chars().take(n).collect();
        text.replace_range(at..=end, &prose);
    }
    text
}

/// The bytes of a file of the fixture.
fn read(fixture: &Fixture, path: &str) -> String {
    fs::read_to_string(fixture.root().join(path)).unwrap_or_default()
}

/// The file with its lines edited by `edit`, which gets each line and returns its replacement,
/// each line ended by a line break as awk writes it.
fn rewrite(fixture: &Fixture, path: &str, edit: impl FnMut(&str) -> Vec<String>) {
    assert!(
        fixture.root().join(path).is_file(),
        "fixture table: no file {path} to edit"
    );
    let text = read(fixture, path);
    let mut out: Vec<String> = text.lines().flat_map(edit).collect();
    if !text.is_empty() {
        out.push(String::new());
    }
    fixture.write(path, &out.join("\n"));
}

/// The text inserted as lines before the first line that starts with the prefix, as
/// `insert_before` of the template's self-test.
fn insert_before(fixture: &Fixture, path: &str, prefix: &str, text: &str) {
    let mut done = false;
    rewrite(fixture, path, |line| {
        if !done && line.starts_with(prefix) {
            done = true;
            vec![text.to_owned(), line.to_owned()]
        } else {
            vec![line.to_owned()]
        }
    });
}

/// The line `<path>  # added` after the first line of the Project Layout block naming AGENTS.md.
fn layout(fixture: &Fixture, path: &str) {
    let mut done = false;
    rewrite(fixture, "CONTRIBUTING.md", |line| {
        let mut out = vec![line.to_owned()];
        if !done && line.split_whitespace().next() == Some("AGENTS.md") {
            done = true;
            out.push(format!("{path}  # added"));
        }
        out
    });
}

/// Git in the fixture, which must succeed.
fn git(fixture: &Fixture, args: &[&str]) -> String {
    let output = fixture.git(args);
    assert!(output.status.success(), "git {args:?}: {}", stderr(&output));
    stdout(&output)
}

/// The fixture of the record table: the self-test's repository, its first ADR removed and its
/// index the record's.
fn record_base(fixture: &Fixture) {
    fixture
        .write("AGENTS.md", "# Agent Guide\n\n## 1. Principles\n\nBe simple.\n\n## 2. Rules\n\nFollow them.\n")
        .write("README.md", "# Fixture\n\n## Template\n\n- **Template release:** unreleased\n")
        .write(
            "CONTRIBUTING.md",
            "# Contributing\n\n## Project Layout\n\n```\n\
             README.md             # for those who use it\n\
             CONTRIBUTING.md       # this file\n\
             AGENTS.md             # the rule file\n\
             docs/adr/             # decisions\n\
             scripts/              # the checks\n\
             .github/              # the workflows\n\
             .agents/skills/       # the skills\n\
             .claude/skills/       # their pointers\n\
             ```\n\n## Repository settings\n\n<!-- required-checks begin -->\n`docs`\n\
             <!-- required-checks end -->\n",
        )
        .write(".github/workflows/checks.yml", "jobs:\n  docs:\n    steps:\n      - run: bash scripts/check-docs.sh\n")
        .write("scripts/check-all.sh", "run \"Documentation consistency\" scripts/check-docs.sh\n")
        .write("scripts/check-docs.sh", STAND_IN_HEADER)
        .write(
            "docs/adr/template.md",
            "# <CAT>-<slug>: title\n\n- **Status:** 🟡 proposed\n\n*Complete the header and delete this paragraph.*\n",
        );
    let mut rows = String::new();
    for role in ["harness", "toolchain"] {
        let id = format!("{}-fixture-{role}", "OPS");
        fixture.write(
            &format!("docs/adr/{id}.md"),
            &format!("# {id}: decision\n\n- **Status:** 🟢 accepted\n- **Role:** {role}\n- **Applies to:** everything\n"),
        );
        rows.push_str(&format!(
            "| [{id}]({id}.md) | Decision | everything | 🟢 accepted |\n"
        ));
    }
    let table = "| ADR | Title | Applies to | Status |\n|-----|-------|------------|--------|\n";
    fixture.write(
        "docs/adr/README.md",
        &format!(
            "# ADRs\n\n### Categories\n\n- **NUC** — the template.\n\n### Binding\n\n{table}{rows}\
             ### Superseded and rejected\n\n{table}"
        ),
    );
    fixture.write(
        ".agents/skills/review/SKILL.md",
        "---\nname: review\ndescription: Review a change.\n---\n\n# Review\n",
    );
    symlink(
        "../../.agents/skills/review",
        &fixture.root().join(".claude/skills/review"),
    );
}

#[cfg(unix)]
fn symlink(target: &str, at: &Path) {
    fs::create_dir_all(at.parent().expect("a parent")).expect("directories");
    std::os::unix::fs::symlink(target, at).expect("linked");
}

#[cfg(not(unix))]
fn symlink(_: &str, _: &Path) {
    unreachable!("the cases run on Unix");
}

/// A step of the record table, split as `step` of the template's self-test splits it.
struct Step<'a> {
    op: &'a str,
    id: &'a str,
    arg: &'a str,
    rest: &'a str,
    text: &'a str,
    file: String,
}

const INDEX: &str = "docs/adr/README.md";

/// One step of the record table, as `step` of the template's self-test; the short hash of the
/// last delete, for `<deleted>`.
fn record_step(fixture: &Fixture, step: &str, deleted: &mut String) {
    let mut words = step.split_whitespace();
    let op = words.next().expect("a step");
    let id = words.next().unwrap_or("");
    let arg = words.next().unwrap_or("");
    let rest = step
        .strip_prefix(op)
        .and_then(|r| r.strip_prefix(' '))
        .map(|r| {
            r.strip_prefix(id)
                .and_then(|r| r.strip_prefix(' '))
                .unwrap_or("")
        })
        .unwrap_or("");
    let s = Step {
        op,
        id,
        arg,
        rest,
        text: step,
        file: format!("docs/adr/{id}.md"),
    };
    let done = record_entry(fixture, &s, deleted)
        || record_file(fixture, &s, deleted)
        || record_git(fixture, &s, deleted);
    assert!(done, "fixture table: unknown step '{op}'");
}

/// A step on an ADR and its row; whether the step is one.
fn record_entry(fixture: &Fixture, s: &Step, deleted: &mut String) -> bool {
    let Step {
        op, id, arg, rest, ..
    } = *s;
    let (file, index) = (s.file.clone(), INDEX);
    match op {
        "adr" => {
            record_step(fixture, &format!("file {id} {arg}"), deleted);
            let table = if arg.starts_with("superseded:") || arg == "rejected" {
                "archive"
            } else {
                "binding"
            };
            record_step(fixture, &format!("row {table} {id} {arg}"), deleted);
        }
        "file" => {
            fixture.write(
                &file,
                &format!(
                    "# {id}: decision\n\n- **Status:** {}\n- **Applies to:** everything\n\n## Decision\n\nWe will.\n",
                    status_of(arg)
                ),
            );
        }
        "row" => {
            let mut rest = rest.split_whitespace();
            let (key, status) = (rest.next().expect("an id"), rest.next().expect("a status"));
            let row = format!(
                "| [{key}]({key}.md) | Decision | everything | {} |",
                status_of(status)
            );
            if id == "binding" {
                insert_before(fixture, index, "### Superseded", &row);
            } else {
                let text = read(fixture, index);
                fixture.write(index, &format!("{text}{row}\n"));
            }
        }
        "field" => {
            let value = rest.strip_prefix(arg).map(str::trim_start).unwrap_or("");
            insert_before(
                fixture,
                &file,
                "- **Applies to:**",
                &format!("- **{arg}:** {value}"),
            );
        }
        "status" => rewrite(fixture, &file, |line| {
            vec![if line.starts_with("- **Status:** ") {
                format!("- **Status:** {}", status_of(arg))
            } else {
                line.to_owned()
            }]
        }),
        "applies" => {
            for path in [file.as_str(), index] {
                let row = format!("| [{id}]({id}.md) |");
                rewrite(fixture, path, |line| {
                    let mut line = line.to_owned();
                    if line.starts_with("- **Applies to:**") {
                        line = format!("- **Applies to:** {rest}");
                    }
                    if line.starts_with(&row)
                        && let Some(at) = line.find("| everything |")
                    {
                        line = format!("{}{rest}{}", &line[..at + 2], &line[at + 12..]);
                    }
                    vec![line]
                });
            }
        }
        "supersedes" => supersedes(fixture, &file, arg),
        "heading" => {
            let mut first = true;
            rewrite(fixture, &file, |line| {
                let out = if first {
                    rest.to_owned()
                } else {
                    line.to_owned()
                };
                first = false;
                vec![out]
            });
        }
        _ => return false,
    }
    true
}

/// A Supersedes field before Applies to, linking each identifier of the comma-separated list; one
/// after '+' on a continuation line.
fn supersedes(fixture: &Fixture, file: &str, arg: &str) {
    let mut list = "- **Supersedes:**".to_owned();
    let mut sep = String::new();
    for item in arg.split(',') {
        let item = match item.strip_prefix('+') {
            Some(item) => {
                sep.push_str("\n  ");
                item
            }
            None => item,
        };
        list.push_str(&format!("{sep} [{item}]({item}.md)"));
        sep = ",".to_owned();
    }
    insert_before(fixture, file, "- **Applies to:**", &list);
}

/// A step on a file of the tree; whether the step is one.
fn record_file(fixture: &Fixture, s: &Step, deleted: &mut String) -> bool {
    let Step {
        op,
        id,
        arg,
        rest,
        text: step,
        ..
    } = *s;
    let index = INDEX;
    match op {
        "text" => {
            let text = read(fixture, id);
            fixture.write(id, &format!("{text}{}\n", table_text(rest)));
        }
        "write" => drop(fixture.write(id, &table_text(rest))),
        "edit" => {
            let (old, new) = match rest.strip_suffix(" =>") {
                Some(old) => (old, None),
                None => rest
                    .split_once(" => ")
                    .map(|(o, n)| (o, Some(n)))
                    .expect("old => new"),
            };
            let mut done = false;
            rewrite(fixture, id, |line| {
                if !done && line.starts_with(old) {
                    done = true;
                    new.map(str::to_owned).into_iter().collect()
                } else {
                    vec![line.to_owned()]
                }
            });
            assert!(done, "fixture table: no line of {id} starts with '{old}'");
        }
        "rm" => {
            let path = fixture.root().join(id);
            if path.is_dir() && !path.is_symlink() {
                fs::remove_dir_all(&path).expect("removed");
            } else {
                fs::remove_file(&path).expect("removed");
            }
        }
        "link" => symlink(arg, &fixture.root().join(id)),
        "layout" => layout(fixture, id),
        "category" => insert_before(
            fixture,
            index,
            "### Binding",
            &format!("- **{id}** — a category of the project."),
        ),
        "blobs" => blobs(fixture),
        "release" => record_step(
            fixture,
            &format!(
                "edit README.md - **Template release:** => - **Template release:** {}",
                &step[8..]
            ),
            deleted,
        ),
        "setup" => {
            fixture.write("TEMPLATE-SETUP.md", "# Template setup\n");
            layout(fixture, "TEMPLATE-SETUP.md");
        }
        "skill" => drop(fixture.write(".agents/skills/review/SKILL.md", &unescape(&step[6..]))),
        _ => return false,
    }
    true
}

/// A step on git's history; whether the step is one.
fn record_git(fixture: &Fixture, s: &Step, deleted: &mut String) -> bool {
    let Step { op, id, .. } = *s;
    let (file, index) = (s.file.clone(), INDEX);
    match op {
        "commit" => {
            git(fixture, &["add", "-A"]);
            git(fixture, &["commit", "-q", "-m", "commit"]);
        }
        "delete" => {
            git(fixture, &["rm", "-q", &file]);
            let row = format!("| [{id}]");
            rewrite(fixture, index, |line| {
                if line.starts_with(&row) {
                    Vec::new()
                } else {
                    vec![line.to_owned()]
                }
            });
            record_step(fixture, "commit", deleted);
            *deleted = git(fixture, &["rev-parse", "--short", "HEAD"])
                .trim()
                .to_owned();
        }
        "branch" => drop(git(fixture, &["switch", "-q", "-c", id])),
        "back" => drop(git(fixture, &["switch", "-q", "-"])),
        "rename" => drop(git(fixture, &["branch", "-q", "-m", id])),
        "merge" => drop(git(fixture, &["merge", "-q", "--no-ff", "-m", "merge", id])),
        _ => return false,
    }
    true
}

/// The text as `printf '%b'` reads it: `\n`, `\t`, and `\\`.
fn unescape(text: &str) -> String {
    text.replace("\\\\", "\u{0}")
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace('\u{0}', "\\")
}

/// `docs/adr/NUC.blobs` as `adr_blobs` of the template writes it: each `NUC-` file of the record
/// in byte order, then AGENTS.md, a line `<blob> <name>` each; empty without a `NUC-` file.
fn blobs(fixture: &Fixture) {
    let dir = fixture.root().join("docs/adr");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .expect("the record is read")
        .map(|e| {
            e.expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|n| n.starts_with("NUC-") && n.ends_with(".md") && dir.join(n).is_file())
        .collect();
    names.sort();
    let mut list = String::new();
    if !names.is_empty() {
        if fixture.root().join("AGENTS.md").is_file() {
            names.push("../../AGENTS.md".to_owned());
        }
        for name in &names {
            let blob = stdout(&support::git_in(&dir, &["hash-object", "--", name]));
            list.push_str(&format!(
                "{} {}\n",
                blob.trim(),
                name.trim_start_matches("../../")
            ));
        }
    }
    fixture.write("docs/adr/NUC.blobs", &list);
}

/// One step of the traceability table.
fn traceability_step(fixture: &Fixture, step: &str) {
    let (op, rest) = step.split_once(' ').unwrap_or((step, ""));
    match op {
        "spec" => drop(fixture.write(
            "docs/SPECIFICATION.md",
            &format!(
                "# Specification\n\n## Goals / Success Criteria\n\n- {rest}\n\n## Quality Goals\n\n\
                 - **Q-1** — fast enough.\n\n## Non-Goals\n\n- none\n"
            ),
        )),
        "adr" => {
            let (id, status) = rest.split_once(' ').expect("an id and a status");
            fixture.write(
                &format!("docs/adr/{id}.md"),
                &format!(
                    "# {id}: decision\n\n- **Status:** {}\n\n## Enforcement\n\nReview only.\n",
                    status_of(status)
                ),
            );
        }
        "mark" => {
            let (file, ids) = rest.split_once(' ').expect("a file and ids");
            fixture.write(file, &format!("{}: {ids}\n", "Verifies"));
        }
        "file" => drop(fixture.write(rest, "text\n")),
        _ => panic!("fixture table: unknown step '{op}'"),
    }
}

/// One step of the shell table.
fn shell_step(fixture: &Fixture, step: &str) {
    let mut words = step.splitn(3, ' ');
    let (op, path, rest) = (
        words.next().expect("a step"),
        words.next().expect("a path"),
        words.next().unwrap_or(""),
    );
    match op {
        "script" => drop(fixture.write(
            path,
            &format!("#!/usr/bin/env bash\n# A stand-in of the kind {rest}.\n"),
        )),
        "write" => drop(fixture.write(path, &format!("{rest}\n"))),
        "mode" => {
            git(fixture, &["add", "--", path]);
            let chmod = match rest {
                "100755" => "--chmod=+x",
                "100644" => "--chmod=-x",
                _ => panic!("fixture table: unknown mode '{rest}'"),
            };
            git(fixture, &["update-index", chmod, "--", path]);
        }
        "link" => {
            symlink(rest, &fixture.root().join(path));
            git(fixture, &["add", "--", path]);
        }
        "conflict" => {
            let blob = git(fixture, &["hash-object", "-w", "--", path]);
            let blob = blob.trim();
            let info: String = (1..=3)
                .map(|stage| format!("100644 {blob} {stage}\t{path}\n"))
                .collect();
            let mut child = std::process::Command::new("git")
                .current_dir(fixture.root())
                .args(["update-index", "--index-info"])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .expect("git runs");
            std::io::Write::write_all(&mut child.stdin.take().expect("stdin"), info.as_bytes())
                .expect("written");
            assert!(child.wait().expect("git ends").success());
        }
        _ => panic!("fixture table: unknown step '{op}'"),
    }
}

/// The rules a case turns on.
fn rules_of(case: &Case) -> Vec<String> {
    let area = match case.table {
        "check-docs-record.txt" => "docs/",
        "check-traceability.txt" => "traceability/",
        _ => "shell/",
    };
    match case.rule.as_deref() {
        Some(rule) if rule != "all" => vec![rule.to_owned()],
        _ => nuc::rules::ALL
            .iter()
            .map(|rule| rule.id)
            .filter(|id| id.starts_with(area))
            .map(str::to_owned)
            .collect(),
    }
}

/// The README of the tables whose tree has none, which `nuc` needs to run at all.
const STAND_IN_README: &str = "# Fixture\n\n- **Template release:** unreleased\n";

/// The rule keys of the header of the template's `scripts/check-docs.sh` at the release of the
/// copies, v0.11.0, which the bash reads from its own script and `docs/adr-refs` reads from the
/// tree's: the record table's tree holds them as a stand-in script.
const STAND_IN_HEADER: &str = "#!/usr/bin/env bash\n\
#   docs/adr-index\n#   docs/adr-identifiers (was: docs/adr-numbering)\n#   docs/links\n\
#   docs/section-refs\n#   docs/adr-refs\n#   docs/adr-link-targets\n#   docs/adr-header\n\
#   docs/adr-supersessions\n#   docs/check-lists\n#   docs/project-layout\n\
#   docs/skill-pointers\n#   docs/base-blobs\n#   docs/template-identity\n\
#   docs/opens-no-issue\n#   docs/template-release\n#   docs/prose-width\nset -uo pipefail\n";

/// Build the case's tree, run `nuc check` on it, and decide whether it agrees.
fn run(case: &Case, n: usize) -> Run {
    let fixture = Fixture::new(&format!("table-{n}"));
    let mut deleted = String::new();
    match case.table {
        "check-docs-record.txt" => {
            record_base(&fixture);
            for step in &case.steps {
                record_step(&fixture, step, &mut deleted);
            }
            git(&fixture, &["add", "-A"]);
        }
        "check-traceability.txt" => {
            fs::create_dir_all(fixture.root().join("docs/adr")).expect("docs/adr/");
            fs::create_dir_all(fixture.root().join("tests")).expect("tests/");
            for step in &case.steps {
                traceability_step(&fixture, step);
            }
            fixture.write("README.md", STAND_IN_README);
            git(&fixture, &["add", "-A"]);
        }
        _ => {
            fixture.write(".gitignore", "build/\n");
            fixture.write(".shellcheckrc", "disable=SC1091,SC2154\n");
            for step in &case.steps {
                shell_step(&fixture, step);
            }
            // The bash stages nothing in this table, so neither is the README.
            fixture.write("README.md", STAND_IN_README);
        }
    }
    let rules: Vec<String> = rules_of(case).iter().map(|r| format!("\"{r}\"")).collect();
    fixture.write(
        "nuc.toml",
        &format!("minimum = \"0.0.0\"\nrules = [{}]\n", rules.join(", ")),
    );
    let exclude = fixture.root().join(".git/info/exclude");
    let kept = fs::read_to_string(&exclude).unwrap_or_default();
    fs::write(&exclude, format!("{kept}/nuc.toml\n")).expect("the exclude is written");
    let output = fixture.nuc(&["check"]);
    let field = case.field.replace("<deleted>", &deleted);
    let want = if case.pass { 0 } else { 1 };
    let agrees = output.status.code() == Some(want) && holds(&stdout(&output), &field);
    Run { output, agrees }
}

/// The rule a case stands under in the list: its table's rule, else the rule of its key, else the
/// table's area.
fn listed_rule(case: &Case) -> String {
    case.rule.clone().unwrap_or_else(|| {
        let key = case.field.trim_start_matches('!');
        key.split(':').next().filter(|_| key != "-").map_or_else(
            || {
                if case.table == "check-shell.txt" {
                    "shell".to_owned()
                } else {
                    "traceability".to_owned()
                }
            },
            str::to_owned,
        )
    })
}

/// What a run of a case decided, for `judge`.
struct Verdict<'a> {
    table: &'a str,
    name: &'a str,
    rule: &'a str,
    pass: bool,
    agrees: bool,
}

/// The problems of one case against its line of `tests/tables.cases`, if it has one: a case that
/// does not agree and is not listed; a listed pending case that agrees; a line under another rule;
/// a pending case whose ticket is not a file under `docs/plan/`; a case not ported of a rule `nuc`
/// ports; a failing case of a rule `nuc` leaves to the bash that does not stand as not ported; and
/// a status of neither form.
fn judge(
    case: &Verdict,
    entry: Option<(String, String)>,
    exists: &dyn Fn(&str) -> bool,
    in_force: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let Verdict {
        table,
        name,
        rule,
        pass,
        agrees,
    } = *case;
    let at = format!("{table} | {name}");
    let unported_rule = NOT_PORTED.contains(&rule);
    let must_be_unported = !pass && unported_rule;
    let Some((listed, status)) = entry else {
        return if agrees && !must_be_unported {
            Vec::new()
        } else {
            vec![format!(
                "{at} | {rule} | pending <ticket>  — does not agree"
            )]
        };
    };
    let mut problems = Vec::new();
    if listed != rule {
        problems.push(format!("{at}: listed under {listed}, its rule is {rule}"));
    }
    if let Some(ticket) = status.strip_prefix("pending ") {
        if agrees {
            problems.push(format!(
                "{at}: agrees now — take it out of tests/tables.cases"
            ));
        }
        if !ticket.starts_with("docs/plan/") || !exists(ticket) {
            problems.push(format!(
                "{at}: pending on {ticket}, which is no ticket under docs/plan/ — its port has landed, so the case agrees or the port is not done"
            ));
        }
    } else if status.starts_with("not ported ") {
        if !unported_rule {
            problems.push(format!("{at}: not ported, but {rule} is a rule nuc ports"));
        }
    } else if let Some(decided) = status.strip_prefix("decided by ") {
        let (adr, reason) = decided.split_once(' ').unwrap_or((decided, ""));
        if agrees {
            problems.push(format!(
                "{at}: agrees now — take it out of tests/tables.cases"
            ));
        }
        if !reason.chars().any(char::is_alphanumeric) {
            problems.push(format!(
                "{at}: decided by {adr}, but the line gives no reason"
            ));
        }
        if !in_force(adr) {
            problems.push(format!(
                "{at}: decided by {adr}, which is no accepted ADR of docs/adr/ — the case is pending again, or another ADR decides it"
            ));
        }
    } else {
        problems.push(format!(
            "{at}: '{status}' is neither 'pending <ticket>', 'not ported <reason>', nor 'decided by <ADR> <reason>'"
        ));
    }
    if must_be_unported && !status.starts_with("not ported ") {
        problems.push(format!(
            "{at}: a case of {rule}, which nuc leaves to the bash, stands only as not ported"
        ));
    }
    problems
}

/// The rules `nuc` leaves to the bash (ARCH-rules-need-only-git).
const NOT_PORTED: [&str; 2] = ["shell/bash-floor", "shell/root"];

#[cfg(unix)]
#[test]
fn every_case_agrees_or_is_listed() {
    let all: Vec<Case> = TABLES
        .iter()
        .flat_map(|(name, text)| cases(name, text))
        .collect();
    let mut listed: BTreeMap<(String, String), (String, String)> = BTreeMap::new();
    for line in entries(CASES) {
        let fields: Vec<&str> = line.split(" | ").collect();
        let [table, name, rule, status] = fields[..] else {
            panic!("tests/tables.cases: '{line}' is not '<table> | <name> | <rule> | <status>'");
        };
        let fresh = listed.insert(
            (table.to_owned(), name.to_owned()),
            (rule.to_owned(), status.to_owned()),
        );
        assert!(
            fresh.is_none(),
            "tests/tables.cases: '{table} | {name}' is listed twice"
        );
    }
    let runs: Vec<(Case, Run)> = std::thread::scope(|scope| {
        let chunks: Vec<_> = all
            .chunks(all.len().div_ceil(8))
            .enumerate()
            .map(|(c, chunk)| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .enumerate()
                        .map(|(i, case)| (case.clone(), run(case, c * 1000 + i)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        chunks
            .into_iter()
            .flat_map(|h| h.join().expect("a chunk runs"))
            .collect()
    });
    let root = support::sources::root();
    // An ADR in force for a decided case: its file under docs/adr/, its status line accepted.
    let accepted = |adr: &str| {
        !adr.contains('/')
            && fs::read_to_string(root.join(format!("docs/adr/{adr}.md"))).is_ok_and(|text| {
                text.lines()
                    .find(|l| l.starts_with("- **Status:**"))
                    .is_some_and(|l| l.contains("🟢"))
            })
    };
    let mut problems = Vec::new();
    for (case, run) in &runs {
        let table = case.table.trim_end_matches(".txt");
        let entry = listed.remove(&(table.to_owned(), case.name.clone()));
        let rule = listed_rule(case);
        let verdict = Verdict {
            table,
            name: &case.name,
            rule: &rule,
            pass: case.pass,
            agrees: run.agrees,
        };
        let exists = |ticket: &str| root.join(ticket).is_file();
        for problem in judge(&verdict, entry, &exists, &accepted) {
            problems.push(if problem.ends_with("does not agree") {
                format!(
                    "{problem}: exit {:?}\n      stdout: {}\n      stderr: {}",
                    run.output.status.code(),
                    stdout(&run.output).trim().replace('\n', "\n              "),
                    stderr(&run.output).trim().lines().last().unwrap_or("")
                )
            } else {
                problem
            });
        }
    }
    for ((table, name), _) in listed {
        problems.push(format!(
            "{table} | {name}: tests/tables.cases names no case of the copies"
        ));
    }
    assert!(
        problems.is_empty(),
        "{} problem(s):\n  {}",
        problems.len(),
        problems.join("\n  ")
    );
}

#[test]
fn a_case_is_judged_against_its_line_as_the_decision_asks() {
    let ticket = "docs/plan/x/01-y.md";
    // Cargo.toml exists, so that only the prefix refuses it as a ticket.
    let exists = |t: &str| t == ticket || t == "Cargo.toml";
    let adr = format!("{}-{}", "API", "x-y");
    let in_force = |a: &str| a == adr;
    let pending = format!("pending {ticket}");
    let case = |rule, agrees| Verdict {
        table: "t",
        name: "n",
        rule,
        pass: false,
        agrees,
    };
    let line = |rule: &str, status: &str| Some((rule.to_owned(), status.to_owned()));
    // Agreeing and unlisted, not agreeing and pending on a ticket that exists, or not ported: none.
    assert!(judge(&case("docs/a", true), None, &exists, &in_force).is_empty());
    assert!(
        judge(
            &case("docs/a", false),
            line("docs/a", &pending),
            &exists,
            &in_force
        )
        .is_empty()
    );
    assert!(
        judge(
            &case("shell/root", false),
            line("shell/root", "not ported r"),
            &exists,
            &in_force
        )
        .is_empty()
    );
    // Three failures of the decision's Enforcement, and the rest of the list's rules; a line that
    // names no case is held by the loop over the lines left, a copy's byte by the pin's test.
    let problem = |c: Verdict, e, needle: &str| {
        let found = judge(&c, e, &exists, &in_force);
        assert!(
            found.iter().any(|p| p.contains(needle)),
            "{needle}: {found:?}"
        );
    };
    problem(case("docs/a", false), None, "does not agree");
    problem(case("docs/a", true), line("docs/a", &pending), "agrees now");
    problem(
        case("docs/a", false),
        line("docs/a", "pending docs/plan/x/02-gone.md"),
        "no ticket under docs/plan/",
    );
    problem(
        case("docs/a", false),
        line("docs/a", "pending Cargo.toml"),
        "no ticket under docs/plan/",
    );
    problem(
        case("docs/a", false),
        line("docs/b", &pending),
        "listed under docs/b",
    );
    problem(
        case("docs/a", false),
        line("docs/a", "not ported r"),
        "is a rule nuc ports",
    );
    problem(case("shell/root", false), None, "does not agree");
    problem(
        case("shell/root", false),
        line("shell/root", &pending),
        "stands only as not ported",
    );
    problem(case("docs/a", false), line("docs/a", "later"), "neither");
    // A case an ADR in force decides otherwise: listed with that ADR and why, while it does not
    // agree and the ADR is accepted.
    let decided = format!("decided by {adr} it says so");
    assert!(
        judge(
            &case("docs/a", false),
            line("docs/a", &decided),
            &exists,
            &in_force
        )
        .is_empty()
    );
    problem(case("docs/a", true), line("docs/a", &decided), "agrees now");
    problem(
        case("docs/a", false),
        line("docs/a", &format!("decided by {adr}")),
        "gives no reason",
    );
    problem(
        case("docs/a", false),
        line("docs/a", &format!("decided by {}-{} r", "API", "gone-away")),
        "no accepted ADR",
    );
}

#[test]
fn the_notation_and_the_matching_read_as_the_template_reads_them() {
    assert_eq!(
        hyphens("SEC:a-rule and NUC:b xSEC:c AB:1 A:b ABCDEF:g"),
        // Assembled, so that the gate's bash checks read no reference here.
        format!("{0}-a-rule and NUC-b x{0}:c AB-1 A:b ABCDEF:g", "SEC")
    );
    assert_eq!(
        hyphens("docs/adr-identifiers:taken @ OPS:old-rule.md"),
        format!("docs/adr-identifiers:taken @ {}-old-rule.md", "OPS")
    );
    let out = "docs/a-rule-two.md: x: docs/a-rule.md names 12 and 2 [docs/x:why]";
    assert!(holds(out, "docs/x:why @ docs/a-rule.md @ 2"));
    assert!(!holds(out, "docs/x:why @ 2 @ docs/a-rule.md"));
    assert!(holds(out, "!docs/x:why @ 2 @ docs/a-rule.md"));
    assert!(!holds(out, "docs/x:other"));
    assert!(holds(out, "-"));
    assert_eq!(table_text("a\\nb <prose 7>"), "a\nb word wo");
    assert_eq!(unescape("a\\nb\\\\n"), "a\nb\\n");
}
