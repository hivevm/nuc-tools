//! The rules whose bash check changed between the template's releases.
//! Verifies: API-version-coupling-hook-defaults
//!
//! `tests/parity/releases.tsv` names, for each rule, the releases of the corpus at which its bash
//! check changed, which the parity harness holds to the template's trees. A rule whose check
//! changed reads the release the tree carries, and `rules::RELEASES` knows each release it changed
//! at, unless its row gives the reason it checks some or every release alike. The releases a reason
//! covers stand in a field of their own before it, "-" for every release of the row.

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

/// What a reason covers: the releases its field names, every release of the row for "-", or
/// nothing, with what is wrong with the row.
enum Covered<'a> {
    Releases(Vec<&'a str>),
    All,
    Malformed(String),
}

/// Whether the text reads as a release: a `v`, then a version that parses, nothing around it.
fn is_release(text: &str) -> bool {
    text.strip_prefix('v').and_then(Version::parse).is_some()
}

/// Whether the word reads as a release, well formed or not: a `v` of either case and a digit.
fn looks_like_release(word: &str) -> bool {
    let mut chars = word.chars();
    matches!(chars.next(), Some('v' | 'V')) && chars.next().is_some_and(|c| c.is_ascii_digit())
}

/// Whether a reason opens with releases rather than prose, the form the field of covered releases
/// replaced. A reason whose first word reads as a release is prose only when that word is one
/// release and nothing more, and the word after it is a word, as in `v0.8.0 moved …`; a colon or a
/// comma after it, a second release, letters stuck to it, or nothing after it is a list.
fn opens_with_releases(reason: &str) -> bool {
    let mut words = reason.split_whitespace();
    let Some(first) = words.next() else {
        return false;
    };
    if !looks_like_release(first) {
        return false;
    }
    let prose = is_release(first)
        && words
            .next()
            .is_some_and(|next| next.starts_with(char::is_alphabetic) && !looks_like_release(next));
    !prose
}

/// What the field of covered releases and the reason of a row cover. A release in the field that
/// does not parse, a space included, is refused, and so is an empty reason or one that opens with
/// a list of releases: nothing falls back to covering every release.
fn covered<'a>(field: &'a str, reason: &str) -> Covered<'a> {
    if reason.trim().is_empty() {
        return Covered::Malformed("its reason is empty".into());
    }
    if opens_with_releases(reason) {
        return Covered::Malformed(
            "its reason opens with a list of releases, which belong in the field before it".into(),
        );
    }
    if field == "-" {
        return Covered::All;
    }
    let releases: Vec<&str> = field.split(',').collect();
    match releases.iter().find(|release| !is_release(release)) {
        Some(bad) => Covered::Malformed(format!(
            "its covered releases '{field}' name '{bad}', which is no release"
        )),
        None => Covered::Releases(releases),
    }
}

fn rows(table: &str) -> Vec<Row<'_>> {
    table
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert!(
                fields.len() >= 4,
                "'{line}' has {} fields, not four or six",
                fields.len()
            );
            let changed = match fields[3] {
                "-" => Vec::new(),
                list => list.split(',').collect(),
            };
            Row {
                rule: fields[0],
                covered: match fields[4..] {
                    [] => Covered::Releases(Vec::new()),
                    [field, reason] => covered(field, reason),
                    _ => Covered::Malformed(format!(
                        "{} fields, where a reason needs the releases it covers in the field \
                         before it",
                        fields.len()
                    )),
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
            Covered::Malformed(what) => {
                found.push(format!("{}: {what}", row.rule));
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

/// Two rules for the tables of the tests: `docs/x` reads no release, `docs/y` reads it; and the
/// one release they know.
fn two_rules() -> ([rules::Rule; 2], [Version; 1]) {
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
    ([ignores_release, reads_release], [Version(0, 6, 0)])
}

#[test]
fn a_rule_whose_check_changed_and_that_reads_no_release_fails_and_is_named() {
    let (all, known) = two_rules();
    let table = "# x\ndocs/x\ts.sh\tf\tv0.6.0\ndocs/y\ts.sh\tg\tv0.6.0\n";
    assert_eq!(
        disagreements(table, &all, &known),
        ["docs/x: its check changed at v0.6.0, and it reads no release"]
    );
    let exempt = "docs/x\ts.sh\tf\tv0.6.0\t-\tbecause\ndocs/y\ts.sh\tg\tv0.6.1\n";
    assert_eq!(
        disagreements(exempt, &all, &known),
        ["docs/y: its check changed at v0.6.1, which rules::RELEASES does not know"]
    );
    let partly = "docs/x\ts.sh\tf\tv0.6.0,v0.6.1\tv0.6.1\tbecause\n\
                  docs/y\ts.sh\tg\tv0.6.0,v0.6.1\tv0.6.1\tbecause\n";
    assert_eq!(
        disagreements(partly, &all, &known),
        ["docs/x: its check changed at v0.6.0, and it reads no release"]
    );
    assert_eq!(
        disagreements(
            "docs/x\ts.sh\tf\t-\tv0.6.1\tbecause\ndocs/y\ts.sh\tg\t-\n",
            &all,
            &known
        ),
        ["docs/x: its reason covers v0.6.1, which the row does not list"]
    );
    let listed = "docs/x\ts.sh\tf\tv0.6.0,v0.6.1\tv0.6.1,v0.6.0\tbecause\ndocs/y\ts.sh\tg\t-\n";
    assert_eq!(disagreements(listed, &all, &known), Vec::<String>::new());
    let prose = "docs/x\ts.sh\tf\tv0.6.0,v0.6.1\t-\tv0.6.1 moved a pattern: so every release is alike\n\
                 docs/y\ts.sh\tg\t-\n";
    assert_eq!(disagreements(prose, &all, &known), Vec::<String>::new());
    assert_eq!(
        disagreements("docs/y\ts.sh\tg\t-\ndocs/z\ts.sh\th\t-\n", &all, &known),
        ["docs/x: 0 rows, not one", "docs/z: a row of no rule of nuc"]
    );
}

#[test]
fn a_malformed_row_covers_nothing_and_is_named() {
    let (all, known) = two_rules();
    // Each of these fell back to covering every release in the grammar of a reason's opening.
    let uncovered = [
        "docs/x: its check changed at v0.6.1, which rules::RELEASES does not know".to_owned(),
        "docs/x: its check changed at v0.6.0, v0.6.1, and it reads no release".to_owned(),
    ];
    for (field, bad) in [
        ("v0.6", "v0.6"),
        ("V0.6.1", "V0.6.1"),
        ("v0.6.1x", "v0.6.1x"),
        ("v0.6.1,", ""),
        ("v0.6.1, v0.6.0", " v0.6.0"),
        ("0.6.1", "0.6.1"),
        ("", ""),
    ] {
        let table =
            format!("docs/x\ts.sh\tf\tv0.6.0,v0.6.1\t{field}\tbecause\ndocs/y\ts.sh\tg\t-\n");
        let mut expected = vec![format!(
            "docs/x: its covered releases '{field}' name '{bad}', which is no release"
        )];
        expected.extend(uncovered.iter().cloned());
        assert_eq!(disagreements(&table, &all, &known), expected, "{field}");
    }
    for (reason, what) in [
        ("", "its reason is empty"),
        (" ", "its reason is empty"),
        (
            "v0.6.1: because",
            "its reason opens with a list of releases, which belong in the field before it",
        ),
        (
            "v0.6.1 , V0.6.0: because",
            "its reason opens with a list of releases, which belong in the field before it",
        ),
        (
            "v0.6.1 v0.6.0: because",
            "its reason opens with a list of releases, which belong in the field before it",
        ),
        (
            "v0.6.1x: because",
            "its reason opens with a list of releases, which belong in the field before it",
        ),
        (
            "v0.6.1",
            "its reason opens with a list of releases, which belong in the field before it",
        ),
        (
            "v0.6,v0.6.1: because",
            "its reason opens with a list of releases, which belong in the field before it",
        ),
    ] {
        let table = format!("docs/x\ts.sh\tf\tv0.6.0,v0.6.1\t-\t{reason}\ndocs/y\ts.sh\tg\t-\n");
        let mut expected = vec![format!("docs/x: {what}")];
        expected.extend(uncovered.iter().cloned());
        assert_eq!(disagreements(&table, &all, &known), expected, "{reason}");
    }
    let five = "docs/x\ts.sh\tf\tv0.6.0,v0.6.1\tv0.6.1: because\ndocs/y\ts.sh\tg\t-\n";
    let mut expected = vec![
        "docs/x: 5 fields, where a reason needs the releases it covers in the field before it"
            .to_owned(),
    ];
    expected.extend(uncovered.iter().cloned());
    assert_eq!(disagreements(five, &all, &known), expected);
    let seven = "docs/x\ts.sh\tf\tv0.6.0,v0.6.1\t-\tbecause\textra\ndocs/y\ts.sh\tg\t-\n";
    let mut expected = vec![
        "docs/x: 7 fields, where a reason needs the releases it covers in the field before it"
            .to_owned(),
    ];
    expected.extend(uncovered.iter().cloned());
    assert_eq!(disagreements(seven, &all, &known), expected);
}
