//! The rules whose bash check changed between the template's releases. Verifies: ADR-0023
//!
//! `tests/parity/releases.tsv` names, for each rule, the releases of the corpus at which its bash
//! check changed, which the parity harness holds to the template's trees. A rule whose check
//! changed reads the release the tree carries, and `rules::RELEASES` knows each release it changed
//! at, unless its row gives the reason it checks every release alike. A reason that opens with
//! releases and a colon covers those releases alone.

use nuc::config::Version;
use nuc::rules::{self, Input};

const TABLE: &str = include_str!("parity/releases.tsv");

/// A row of the table: the rule ID, the releases its check changed at, and those of them its
/// reason covers, every one for a reason that names none.
struct Row<'a> {
    rule: &'a str,
    changed: Vec<&'a str>,
    covered: Vec<&'a str>,
}

/// The releases a reason covers: those it opens with, before a colon, or `changed` whole.
fn covered<'a>(reason: Option<&'a str>, changed: &[&'a str]) -> Vec<&'a str> {
    let Some(reason) = reason else {
        return Vec::new();
    };
    match reason.split_once(": ") {
        Some((list, _)) if list.split(',').all(|r| r.starts_with('v')) => list.split(',').collect(),
        _ => changed.to_vec(),
    }
}

fn rows(table: &str) -> Vec<Row<'_>> {
    table
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert!(
                (4..=5).contains(&fields.len()),
                "'{line}' has {} fields, not four or five",
                fields.len()
            );
            let changed = match fields[3] {
                "-" => Vec::new(),
                list => list.split(',').collect(),
            };
            Row {
                rule: fields[0],
                covered: covered(fields.get(4).copied(), &changed),
                changed,
            }
        })
        .collect()
}

/// What the table says against the rules: a rule without a row or with two, a row of no rule, a
/// release that is none, a reason for a release the row does not list, and a rule whose check
/// changed at a release no reason covers that does not read the release.
fn disagreements(table: &str, all: &[rules::Rule], known: &[Version]) -> Vec<String> {
    let rows = rows(table);
    let mut found = Vec::new();
    for rule in all {
        let n = rows.iter().filter(|row| row.rule == rule.id).count();
        if n != 1 {
            found.push(format!("{}: {n} rows, not one", rule.id));
        }
    }
    for row in &rows {
        let Some(rule) = all.iter().find(|rule| rule.id == row.rule) else {
            found.push(format!("{}: a row of no rule of nuc", row.rule));
            continue;
        };
        for release in row.covered.iter().filter(|r| !row.changed.contains(r)) {
            found.push(format!(
                "{}: its reason covers {release}, which the row does not list",
                row.rule
            ));
        }
        let uncovered: Vec<&str> = row
            .changed
            .iter()
            .copied()
            .filter(|r| !row.covered.contains(r))
            .collect();
        for release in &row.changed {
            match release.strip_prefix('v').and_then(Version::parse) {
                None => found.push(format!("{}: '{release}' is no release", row.rule)),
                Some(version) if uncovered.contains(release) && !known.contains(&version) => {
                    found.push(format!(
                        "{}: its check changed at {release}, which rules::RELEASES does not know",
                        row.rule
                    ));
                }
                Some(_) => {}
            }
        }
        if !uncovered.is_empty() && !rule.reads.contains(&Input::Release) {
            found.push(format!(
                "{}: its check changed at {}, and it reads no release",
                row.rule,
                uncovered.join(", ")
            ));
        }
    }
    found
}

#[test]
fn every_rule_whose_check_changed_reads_the_release() {
    let found = disagreements(TABLE, rules::ALL, rules::RELEASES);
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_rule_whose_check_changed_and_that_reads_no_release_fails_and_is_named() {
    let ignores_release = rules::Rule {
        id: "docs/x",
        explain: "",
        reads: &[Input::Readme],
        check: |_| rules::Outcome::default(),
    };
    let reads_release = rules::Rule {
        id: "docs/y",
        reads: &[Input::Release],
        ..ignores_release
    };
    let all = [ignores_release, reads_release];
    let known = [Version(0, 6, 0)];
    let table = "# x\ndocs/x\ts.sh\tf\tv0.6.0\ndocs/y\ts.sh\tg\tv0.6.0\n";
    assert_eq!(
        disagreements(table, &all, &known),
        ["docs/x: its check changed at v0.6.0, and it reads no release"]
    );
    let exempt = "docs/x\ts.sh\tf\tv0.6.0\tbecause\ndocs/y\ts.sh\tg\tv0.6.1\n";
    assert_eq!(
        disagreements(exempt, &all, &known),
        ["docs/y: its check changed at v0.6.1, which rules::RELEASES does not know"]
    );
    let partly = "docs/x\ts.sh\tf\tv0.6.0,v0.6.1\tv0.6.1: because\n\
                  docs/y\ts.sh\tg\tv0.6.0,v0.6.1\tv0.6.1: because\n";
    assert_eq!(
        disagreements(partly, &all, &known),
        ["docs/x: its check changed at v0.6.0, and it reads no release"]
    );
    assert_eq!(
        disagreements(
            "docs/x\ts.sh\tf\t-\tv0.6.1: because\ndocs/y\ts.sh\tg\t-\n",
            &all,
            &known
        ),
        ["docs/x: its reason covers v0.6.1, which the row does not list"]
    );
    assert_eq!(
        disagreements("docs/y\ts.sh\tg\t-\ndocs/z\ts.sh\th\t-\n", &all, &known),
        ["docs/x: 0 rows, not one", "docs/z: a row of no rule of nuc"]
    );
}
