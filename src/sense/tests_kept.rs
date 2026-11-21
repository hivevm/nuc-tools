//! `nuc sense tests-kept`: what a change does that would weaken the test suite, as the template's
//! `scripts/sensor-tests-kept.sh` of the tree's release reports it: test files deleted or moved out
//! of the tests, and skip or focus markers on added lines. The shell resolves the change and asks
//! git for its renames and deletions and for its diff; the sensor reads them as the bash's `read`
//! and awk do, under `LC_ALL=C`.

use regex::bytes::{Regex, RegexBuilder};

use super::{Refusal, Sensor};
use crate::bash;
use crate::config::Version;

/// The sensor, which reports alike from the template's first release up to `BY_EXTENSION`: the
/// releases before v0.5.0 take the root and find main's merge base in their own lines, with the
/// same result.
pub const SENSOR: Sensor = Sensor {
    name: "tests-kept",
    since: Version(0, 1, 0),
    changes: &[BY_EXTENSION, ARGUMENTS],
    reasons: &[
        "ref",
        "git",
        super::DRIVER,
        super::ENVIRONMENT,
        super::HUNK,
        super::RELEASE,
    ],
};

/// The label that opens each line of the report.
pub const LABEL: &str = "Tests kept sensor";

/// The release from which each toolchain's markers are read in its files alone, by extension; the
/// diff names its paths unquoted, `core.quotePath=false`; and a tab git ends a path with is no part
/// of it.
pub const BY_EXTENSION: Version = Version(0, 6, 2);

/// The release from which an argument past the base and the head is refused, where the releases
/// before ignore it.
pub const ARGUMENTS: Version = Version(0, 10, 0);

/// The paths the diff of the markers leaves out: the sensor, its self-test, and the catalogue entry
/// whose table names the markers, which carry them as data.
pub const LEFT_OUT: [&str; 3] = [
    ":(exclude)scripts/sensor-tests-kept.sh",
    ":(exclude)scripts/test-sensor-tests-kept.sh",
    ":(exclude).agents/skills/propose-adr/concepts/toolchain.md",
];

/// The markers per toolchain, from `BY_EXTENSION`: the extensions of its files and the expression
/// read in them, as the bash's table holds them.
const MARKERS: [(&str, &str); 6] = [
    (
        "java kt kts groovy gradle scala",
        r"@Disabled|@Ignore|enabled *= *false",
    ),
    (
        "js jsx mjs cjs ts tsx mts cts vue svelte",
        r"(^|[^A-Za-z0-9_.])(it|test|describe|context)\.(skip|only)\(|(^|[^A-Za-z0-9_.])(xit|xtest|xdescribe|fit|fdescribe|ftest)\(",
    ),
    ("py", r"pytest\.mark\.skip|pytest\.skip\(|unittest\.skip"),
    ("go", r"t\.Skip[a-zA-Z]*\("),
    ("rs", r"#\[ignore\]"),
    ("cs fs vb", r#"Skip *= *""#),
];

/// The markers before `BY_EXTENSION`: one expression read in every file.
const ALL_MARKERS: &str = r#"@Disabled|@Ignore|enabled *= *false|t\.Skip[a-zA-Z]*\(|pytest\.mark\.skip|pytest\.skip\(|unittest\.skip|#\[ignore\]|(^|[^A-Za-z0-9_.])(it|test|describe|context)\.(skip|only)\(|(^|[^A-Za-z0-9_])(xit|xtest|xdescribe|fit|fdescribe|ftest)\(|Skip *= *""#;

/// An expression as awk reads it in the C locale: bytes, not characters.
fn expression(text: &str) -> Regex {
    RegexBuilder::new(text)
        .unicode(false)
        .build()
        .expect("the markers are valid expressions")
}

/// Whether a segment or the basename of the path contains `test` or `spec`, in any case of ASCII.
pub fn is_test_path(path: &[u8]) -> bool {
    path.split(|&b| b == b'/').any(|segment| {
        let lower = segment.to_ascii_lowercase();
        bash::contains(&lower, b"test") || bash::contains(&lower, b"spec")
    })
}

/// The findings of `git diff --name-status -M --diff-filter=DR`: each test file deleted, and each
/// moved to a path that is no test's.
pub fn removals(name_status: &[u8]) -> Vec<Vec<u8>> {
    let mut findings = Vec::new();
    for line in bash::read_lines(name_status) {
        let (status, old, new) = bash::read_tab_fields(line);
        if status == b"D" {
            if is_test_path(&old) {
                findings.push([&b"deleted test file: "[..], &old].concat());
            }
        } else if status.starts_with(b"R") && is_test_path(&old) && !is_test_path(&new) {
            findings.push(
                [
                    &b"test file moved out of the tests: "[..],
                    &old,
                    b" -> ",
                    &new,
                ]
                .concat(),
            );
        }
    }
    findings
}

/// The findings of `git diff -U0`: each added line that carries a marker of the toolchain of its
/// file, as `file:line: text`, read as the bash's awk reads the diff.
pub fn markers(diff: &[u8], release: Version) -> Result<Vec<Vec<u8>>, Refusal> {
    let by_extension = release >= BY_EXTENSION;
    let all = expression(ALL_MARKERS);
    let table: Vec<(Vec<&str>, Regex)> = MARKERS
        .iter()
        .map(|(extensions, text)| (extensions.split(' ').collect(), expression(text)))
        .collect();
    let mut findings = Vec::new();
    let mut file: Vec<u8> = Vec::new();
    let mut marker: Option<&Regex> = None;
    let mut line: u64 = 0;
    for text in bash::read_lines(diff) {
        if text.starts_with(b"+++ ") {
            // awk's substr($0, 7): from the seventh byte on, which drops the 'b/' of the path.
            file = text.get(6..).unwrap_or_default().to_vec();
            if by_extension {
                if file.ends_with(b"\t") {
                    file.pop();
                }
                let lower = file.to_ascii_lowercase();
                let base = match lower.iter().rposition(|&b| b == b'/') {
                    Some(at) => &lower[at + 1..],
                    None => &lower[..],
                };
                let extension = match base.iter().rposition(|&b| b == b'.') {
                    Some(at) => &base[at + 1..],
                    None => &b""[..],
                };
                marker = table
                    .iter()
                    .find(|(extensions, _)| extensions.iter().any(|e| e.as_bytes() == extension))
                    .map(|(_, regex)| regex);
            } else {
                marker = Some(&all);
            }
            continue;
        }
        if text.starts_with(b"@@") {
            line = bash::hunk_first_added(text).map_err(super::hunk_refusal(SENSOR.name))?;
            continue;
        }
        if text.starts_with(b"+") {
            if marker.is_some_and(|regex| regex.is_match(text)) {
                let mut finding = b"skip or focus marker added: ".to_vec();
                finding.extend_from_slice(&file);
                // A line past what awk counts exactly is not decided.
                if line > bash::AWK_EXACT {
                    return Err(super::hunk_refusal(SENSOR.name)(bash::Overflow));
                }
                finding.extend_from_slice(format!(":{line}: ").as_bytes());
                finding.extend_from_slice(&text[1..]);
                findings.push(finding);
            }
            // Within u64: the head reads at most AWK_EXACT, and the diff holds fewer lines.
            line += 1;
        }
    }
    Ok(findings)
}

/// The report where base and head are the same commit.
pub fn same_commit() -> Vec<u8> {
    format!("{LABEL}: nothing to compare — base and head are the same commit.\n").into_bytes()
}

/// The report of the findings between the two commits, named by their short hashes.
pub fn report(base: &str, head: &str, findings: &[Vec<u8>]) -> Vec<u8> {
    if findings.is_empty() {
        return format!("{LABEL} ({base}..{head}): nothing to report.\n").into_bytes();
    }
    let mut out = format!(
        "{LABEL} ({base}..{head}): {} finding(s) for the reviewer — each needs its reason in the pull request (AGENTS.md, section 5):\n",
        findings.len()
    )
    .into_bytes();
    for finding in findings {
        out.extend_from_slice(b"  - ");
        out.extend_from_slice(finding);
        out.push(b'\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEWEST: Version = Version(0, 12, 0);

    #[test]
    fn a_hunk_head_awk_reads_inexactly_is_not_decided() {
        for diff in [
            &b"+++ b/a.rs\n@@ -1 +99999999999999999999 @@\n+#[ignore]\n"[..],
            b"+++ b/a.rs\n@@ -1 +9007199254740992,2 @@\n+a\n+#[ignore]\n",
            b"+++ b/a.rs\n@@ -1 +9007199254740993 @@\n+a\n",
        ] {
            let refusal = markers(diff, NEWEST).unwrap_err();
            assert!(
                refusal.to_string().ends_with("[sense/tests-kept:hunk]"),
                "{refusal}"
            );
        }
        let diff = b"+++ b/a.rs\n@@ -1 +9007199254740991,2 @@\n+a\n+#[ignore]\n";
        assert_eq!(
            texts(markers(diff, NEWEST).unwrap()),
            ["skip or focus marker added: a.rs:9007199254740992: #[ignore]"]
        );
    }

    fn texts(findings: Vec<Vec<u8>>) -> Vec<String> {
        findings
            .into_iter()
            .map(|f| String::from_utf8(f).unwrap())
            .collect()
    }

    #[test]
    fn a_test_path_has_a_segment_with_test_or_spec_in_any_case() {
        assert!(is_test_path(b"tests/a.rs"));
        assert!(is_test_path(b"src/MySpec.java"));
        assert!(!is_test_path(b"manifest.json"));
        assert!(is_test_path(b"src/latest/x.rs"));
        assert!(!is_test_path(b"src/lib.rs"));
    }

    #[test]
    fn deletions_and_moves_out_are_found_and_moves_within_are_not() {
        let out = b"D\ttests/a.rs\nD\tsrc/b.rs\nR100\ttests/c.rs\tsrc/c.rs\nR090\ttests/d.rs\ttest/d.rs\n";
        assert_eq!(
            texts(removals(out)),
            [
                "deleted test file: tests/a.rs",
                "test file moved out of the tests: tests/c.rs -> src/c.rs"
            ]
        );
    }

    #[test]
    fn a_marker_counts_only_in_the_files_of_its_toolchain_from_its_release() {
        let diff = b"diff --git a/a.rs b/a.rs\n--- a/a.rs\n+++ b/a.rs\n@@ -0,0 +3,2 @@\n+fn x() {}\n+#[ignore]\n+++ b/b.js\n@@ -1 +7 @@\n+  it.only(\"x\")\n+++ b/c.rs\n@@ -0,0 +1 @@\n+fit(\n";
        assert_eq!(
            texts(markers(diff, NEWEST).unwrap()),
            [
                "skip or focus marker added: a.rs:4: #[ignore]",
                "skip or focus marker added: b.js:7:   it.only(\"x\")"
            ]
        );
        assert_eq!(texts(markers(diff, Version(0, 6, 1)).unwrap()).len(), 3);
    }

    #[test]
    fn a_path_with_a_space_loses_its_tab_from_its_release_on() {
        let diff = b"+++ b/a b.py\t\n@@ -0,0 +1 @@\n+@pytest.mark.skip\n";
        assert_eq!(
            texts(markers(diff, NEWEST).unwrap()),
            ["skip or focus marker added: a b.py:1: @pytest.mark.skip"]
        );
        assert_eq!(
            texts(markers(diff, Version(0, 6, 1)).unwrap()),
            ["skip or focus marker added: a b.py\t:1: @pytest.mark.skip"]
        );
    }

    #[test]
    fn the_report_counts_its_findings() {
        assert_eq!(
            String::from_utf8(report("a", "b", &[])).unwrap(),
            "Tests kept sensor (a..b): nothing to report.\n"
        );
        let out = String::from_utf8(report("a", "b", &[b"x".to_vec()])).unwrap();
        assert!(out.starts_with("Tests kept sensor (a..b): 1 finding(s) for the reviewer"));
        assert!(out.ends_with(":\n  - x\n"));
    }
}
