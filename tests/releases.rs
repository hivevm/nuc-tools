//! The rules whose bash check changed between the template's releases. Verifies: ADR-0023
//!
//! `tests/parity/releases.tsv` names, for each rule, the releases of the corpus at which its bash
//! check changed, which the parity harness holds to the template's trees. A rule whose check
//! changed reads the release the tree carries, and `rules::RELEASES` knows each release it changed
//! at, unless its row gives the reason it checks every release alike. A reason that opens with
//! releases and a colon, as `v0.8.1: …`, covers those releases alone.

use nuc::config::Version;
use nuc::rules::{self, Input};

const TABLE: &str = include_str!("parity/releases.tsv");

/// A row of the table: the rule ID, the releases its check changed at, and what its reason
/// covers.
struct Row<'a> {
    rule: &'a str,
    changed: Vec<&'a str>,
    covered: Covered<'a>,
}

/// What a reason covers: the releases it opens with before a colon, every release of the row for
/// a reason without them, or nothing, with the opening that is neither.
enum Covered<'a> {
    Releases(Vec<&'a str>),
    All,
    Malformed(&'a str),
}

/// Whether the text opens with what reads as a release, a `v` of either case and a digit.
fn opens_with_release(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some('v' | 'V')) && chars.next().is_some_and(|c| c.is_ascii_digit())
}

/// What a reason covers. A reason that opens with no release is prose and covers every release.
/// One that opens with a release goes on with a list of releases separated by commas and
/// followed by a colon, space allowed around each, which covers those, or with space and a word
/// after its first release, which makes it prose. Anything else is malformed: a release that does not
/// parse, a list cut off before its colon, a release after a release, a bare release, and an
/// empty reason.
fn covered(reason: &str) -> Covered<'_> {
    if reason.trim().is_empty() {
        return Covered::Malformed(reason);
    }
    let mut releases = Vec::new();
    let mut rest = reason;
    loop {
        let item = rest.trim_start();
        if !opens_with_release(item) {
            return if releases.is_empty() {
                Covered::All
            } else {
                Covered::Malformed(reason)
            };
        }
        let end = item
            .find(|c: char| !(c == 'v' || c == '.' || c.is_ascii_digit()))
            .unwrap_or(item.len());
        let release = &item[..end];
        if release.strip_prefix('v').and_then(Version::parse).is_none() {
            return Covered::Malformed(reason);
        }
        let after = item[end..].trim_start();
        match after.chars().next() {
            Some(':') => {
                releases.push(release);
                return Covered::Releases(releases);
            }
            Some(',') => {
                releases.push(release);
                rest = &after[1..];
            }
            Some(c)
                if releases.is_empty()
                    && item[end..].starts_with(char::is_whitespace)
                    && c.is_alphabetic()
                    && !opens_with_release(after) =>
            {
                return Covered::All;
            }
            _ => return Covered::Malformed(reason),
        }
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
                covered: match fields.get(4) {
                    Some(reason) => covered(reason),
                    None => Covered::Releases(Vec::new()),
                },
                changed,
            }
        })
        .collect()
}

/// What the table says against the rules: a rule without a row or with two, a row of no rule, a
/// release that is none, a malformed reason, a reason for a release the row does not list, and a
/// rule whose check changed at a release no reason covers that does not read the release.
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
        let covered: &[&str] = match &row.covered {
            Covered::Releases(releases) => releases,
            Covered::All => &row.changed,
            Covered::Malformed(reason) => {
                found.push(format!(
                    "{}: its reason '{reason}' opens with no list of releases and a colon, and is \
                     no prose",
                    row.rule
                ));
                &[]
            }
        };
        for release in covered.iter().filter(|r| !row.changed.contains(r)) {
            found.push(format!(
                "{}: its reason covers {release}, which the row does not list",
                row.rule
            ));
        }
        let uncovered: Vec<&str> = row
            .changed
            .iter()
            .copied()
            .filter(|r| !covered.contains(r))
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
    let spaced = "docs/x\ts.sh\tf\tv0.6.0,v0.6.1\t v0.6.1 :because\ndocs/y\ts.sh\tg\t-\n";
    assert_eq!(
        disagreements(spaced, &all, &known),
        ["docs/x: its check changed at v0.6.0, and it reads no release"]
    );
    let listed = "docs/x\ts.sh\tf\tv0.6.0,v0.6.1\tv0.6.1 , v0.6.0: because\ndocs/y\ts.sh\tg\t-\n";
    assert_eq!(disagreements(listed, &all, &known), Vec::<String>::new());
    let prose = "docs/x\ts.sh\tf\tv0.6.0,v0.6.1\tv0.6.1 moved a pattern: so every release is alike\n\
                 docs/y\ts.sh\tg\t-\n";
    assert_eq!(disagreements(prose, &all, &known), Vec::<String>::new());
    for reason in [
        "v0.6.1 v0.6.0: because",
        "v0.6.1,: because",
        "v0.6.1, v0.6.0 because",
        "v0.6: because",
        "v0.6,v0.6.1: because",
        "V0.6.1: because",
        "v0.6.1",
        "v0.6.1x: because",
        "",
    ] {
        let table = format!("docs/x\ts.sh\tf\tv0.6.0,v0.6.1\t{reason}\ndocs/y\ts.sh\tg\t-\n");
        assert_eq!(
            disagreements(&table, &all, &known),
            [
                format!(
                    "docs/x: its reason '{reason}' opens with no list of releases and a colon, \
                     and is no prose"
                ),
                "docs/x: its check changed at v0.6.1, which rules::RELEASES does not know".into(),
                "docs/x: its check changed at v0.6.0, v0.6.1, and it reads no release".into(),
            ],
            "{reason}"
        );
    }
    assert_eq!(
        disagreements("docs/y\ts.sh\tg\t-\ndocs/z\ts.sh\th\t-\n", &all, &known),
        ["docs/x: 0 rows, not one", "docs/z: a row of no rule of nuc"]
    );
}
