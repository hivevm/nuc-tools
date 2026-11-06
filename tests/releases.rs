//! The rules whose bash check changed between the template's releases. Verifies: ADR-0023
//!
//! `tests/parity/releases.tsv` names, for each rule, the releases of the corpus at which its bash
//! check changed, which the parity harness holds to the template's trees. A rule whose check
//! changed reads the release the tree carries, and `rules::RELEASES` knows each release it changed
//! at, unless its row gives the reason it checks every release alike.

use nuc::config::Version;
use nuc::rules::{self, Input};

const TABLE: &str = include_str!("parity/releases.tsv");

/// A row of the table: the rule ID, the releases its check changed at, and the reason it reads
/// none, if the row gives one.
struct Row<'a> {
    rule: &'a str,
    changed: Vec<&'a str>,
    reason: Option<&'a str>,
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
            Row {
                rule: fields[0],
                changed: match fields[3] {
                    "-" => Vec::new(),
                    list => list.split(',').collect(),
                },
                reason: fields.get(4).copied(),
            }
        })
        .collect()
}

/// What the table says against the rules: a rule without a row or with two, a row of no rule, a
/// release that is none, and a rule whose check changed that neither reads the release nor gives
/// a reason.
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
        for release in &row.changed {
            match release.strip_prefix('v').and_then(Version::parse) {
                None => found.push(format!("{}: '{release}' is no release", row.rule)),
                Some(version) if row.reason.is_none() && !known.contains(&version) => {
                    found.push(format!(
                        "{}: its check changed at {release}, which rules::RELEASES does not know",
                        row.rule
                    ));
                }
                Some(_) => {}
            }
        }
        if !row.changed.is_empty() && row.reason.is_none() && !rule.reads.contains(&Input::Release)
        {
            found.push(format!(
                "{}: its check changed at {}, and it reads no release",
                row.rule,
                row.changed.join(", ")
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
    assert_eq!(
        disagreements("docs/y\ts.sh\tg\t-\ndocs/z\ts.sh\th\t-\n", &all, &known),
        ["docs/x: 0 rows, not one", "docs/z: a row of no rule of nuc"]
    );
}
