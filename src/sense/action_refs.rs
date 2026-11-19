//! `nuc sense action-refs`: how the workflows reference the GitHub Actions they run, as the
//! template's `scripts/sensor-action-refs.sh` of the tree's release reports it, and with
//! `--require <major-tag|sha>` the check of the form the project decided. The shell lists the
//! workflows as the bash's globs do and reads them; the sensor reads each `uses:` line as the
//! bash's grep and sed do, under `LC_ALL=C`.

use regex::bytes::{Regex, RegexBuilder};

use super::Sensor;
use crate::bash;
use crate::config::Version;

/// The sensor, which reports alike in every release from the template's first.
pub const SENSOR: Sensor = Sensor {
    name: "action-refs",
    since: Version(0, 1, 0),
    changes: &[],
    reasons: &["require", "file", super::RELEASE],
};

/// The directory of the workflows.
pub const WORKFLOWS: &str = ".github/workflows";

/// The file whose `github-actions` entry keeps a major tag current.
pub const DEPENDABOT: &str = ".github/dependabot.yml";

/// The form a `--require` names.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Form {
    MajorTag,
    Sha,
}

impl Form {
    /// The form of the flag's value, none for another.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "major-tag" => Some(Self::MajorTag),
            "sha" => Some(Self::Sha),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::MajorTag => "major-tag",
            Self::Sha => "sha",
        }
    }
}

/// An expression as grep and bash read it in the C locale: bytes, not characters.
fn expression(text: &str) -> Regex {
    RegexBuilder::new(text)
        .unicode(false)
        .build()
        .expect("a valid expression")
}

/// The space class of POSIX, as `[[:space:]]` in an expression.
const SPACE: &str = r"[ \t\n\x0B\x0C\r]";

/// What the run found: the counts of the three forms, the findings for the reviewer, and with a
/// `--require` the references outside the form.
#[derive(Default)]
pub struct Count {
    major: usize,
    full: usize,
    sha: usize,
    findings: Vec<Vec<u8>>,
    excluded: Vec<Vec<u8>>,
}

/// The value of a `uses:` line as the bash's sed strips it: the key and the space after it, a
/// comment after space, one quote at either end, and the trailing space.
fn value(line: &[u8]) -> Vec<u8> {
    let key = expression(&format!(r"^{SPACE}*(- )?uses:{SPACE}*"));
    let mut v = match key.find(line) {
        Some(m) => line[m.end()..].to_vec(),
        None => line.to_vec(),
    };
    let comment = (0..v.len()).find(|&at| {
        let run = v[at..].iter().take_while(|&&b| bash::is_space(b)).count();
        run > 0 && v.get(at + run) == Some(&b'#')
    });
    if let Some(at) = comment {
        v.truncate(at);
    }
    if matches!(v.first(), Some(b'"' | b'\'')) {
        v.remove(0);
    }
    if matches!(v.last(), Some(b'"' | b'\'')) {
        v.pop();
    }
    while v.last().is_some_and(|&b| bash::is_space(b)) {
        v.pop();
    }
    v
}

/// Read one workflow at its path under the root, counting its references and noting what the form
/// excludes.
pub fn read(count: &mut Count, path: &str, text: &[u8], require: Option<Form>) {
    let uses = expression(&format!(r"^{SPACE}*(- )?uses:"));
    let major = expression(r"^v?[0-9]+$");
    let full = expression(r"^v?[0-9]+\.[0-9]+(\.[0-9]+)?$");
    let sha = expression(r"^[0-9a-f]{40}$");
    for (index, line) in bash::awk_lines(text).into_iter().enumerate() {
        if !uses.is_match(line) {
            continue;
        }
        let value = value(line);
        let at = format!("{path}:{}", index + 1);
        // `<at>: '<value>'<tail>`, the value byte for byte.
        let says = |tail: &str| {
            let mut line = format!("{at}: '").into_bytes();
            line.extend_from_slice(&value);
            line.push(b'\'');
            line.extend_from_slice(tail.as_bytes());
            line
        };
        if value.starts_with(b"./") {
            continue;
        }
        if let Some(image) = value.strip_prefix(b"docker://") {
            let last = image
                .iter()
                .rposition(|&b| b == b'/')
                .map_or(image, |at| &image[at + 1..]);
            if !last.contains(&b':') {
                count.findings.push(says(" carries no tag"));
            }
            continue;
        }
        let Some(sign) = value.iter().rposition(|&b| b == b'@') else {
            count.findings.push(says(" names no ref"));
            continue;
        };
        let reference = &value[sign + 1..];
        if major.is_match(reference) {
            count.major += 1;
            if require == Some(Form::Sha) {
                count
                    .excluded
                    .push(says(" is on a major tag, not on a commit SHA"));
            }
        } else if full.is_match(reference) {
            count.full += 1;
            if require.is_some() {
                count.excluded.push(says(
                    " is on a full version tag, which the decision excludes",
                ));
            }
        } else if sha.is_match(reference) {
            count.sha += 1;
            if require == Some(Form::MajorTag) {
                count
                    .excluded
                    .push(says(" is on a commit SHA, not on a major tag"));
            }
        } else {
            let mut line = says(" references '");
            line.extend_from_slice(reference);
            line.extend_from_slice(b"', which is neither a version tag nor a commit SHA");
            count.findings.push(line);
        }
    }
}

/// Whether `.github/dependabot.yml` has a `github-actions` entry, as the bash's grep finds one.
pub fn keeps_actions(dependabot: Option<&[u8]>) -> bool {
    let entry = expression(&format!(
        r"^{SPACE}*-?{SPACE}*package-ecosystem:{SPACE}*.?github-actions"
    ));
    dependabot.is_some_and(|text| bash::awk_lines(text).into_iter().any(|l| entry.is_match(l)))
}

/// The report where there is no workflow.
pub fn no_workflow() -> Vec<u8> {
    format!("Action references sensor: no workflow under {WORKFLOWS} — nothing to report.\n")
        .into_bytes()
}

/// The outcome of a run: the report on the standard output, and with a `--require` that fails, the
/// references outside the form for the standard error, where the bash writes them and exits 1.
pub struct Outcome {
    pub report: Vec<u8>,
    pub failed: Option<Vec<u8>>,
}

/// The outcome of the references counted, with or without a `--require`.
pub fn outcome(count: Count, require: Option<Form>, keeps_actions: bool) -> Outcome {
    let Count {
        major,
        full,
        sha,
        mut findings,
        mut excluded,
    } = count;
    let summary =
        format!("{major} on a major tag, {full} on a full version tag, {sha} on a commit SHA");
    if let Some(form) = require {
        excluded.append(&mut findings);
        if form == Form::MajorTag && !keeps_actions {
            excluded.push(
                format!("{DEPENDABOT}: no 'github-actions' entry — a major tag without Dependabot never learns of the next major")
                    .into_bytes(),
            );
        }
        if excluded.is_empty() {
            return Outcome {
                report: format!(
                    "Action references check ({}): {summary} — every reference is in the decided form.\n",
                    form.name()
                )
                .into_bytes(),
                failed: None,
            };
        }
        let mut failed = format!(
            "Action references check ({}): {} reference(s) outside the decided form:\n",
            form.name(),
            excluded.len()
        )
        .into_bytes();
        for line in &excluded {
            failed.extend_from_slice(b"  - ");
            failed.extend_from_slice(line);
            failed.push(b'\n');
        }
        return Outcome {
            report: Vec::new(),
            failed: Some(failed),
        };
    }
    let forms = [major, full, sha].iter().filter(|&&n| n > 0).count();
    if forms > 1 {
        findings.insert(0, format!("mixed forms — {summary}").into_bytes());
    }
    let report = if findings.is_empty() {
        format!("Action references sensor: {summary} — nothing to report.\n").into_bytes()
    } else {
        let mut out = format!(
            "Action references sensor: {} finding(s) for the reviewer — each is held against the project's decision on action references:\n",
            findings.len()
        )
        .into_bytes();
        for line in &findings {
            out.extend_from_slice(b"  - ");
            out.extend_from_slice(line);
            out.push(b'\n');
        }
        out
    };
    Outcome {
        report,
        failed: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(bytes: &[u8]) -> String {
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[test]
    fn a_value_loses_its_key_comment_quotes_and_trailing_space() {
        assert_eq!(value(b"      - uses: \"a/b@v4\"  # pinned "), b"a/b@v4");
        assert_eq!(value(b"  uses: 'a/b@main'\r"), b"a/b@main'");
        assert_eq!(
            value(b"uses: a/b@v1#not-a-comment"),
            b"a/b@v1#not-a-comment"
        );
    }

    #[test]
    fn each_form_is_counted_and_what_is_no_release_is_found() {
        let workflow = b"steps:\n  - uses: a/b@v4\n  - uses: c/d@1.2.3\n  - uses: e/f@0123456789abcdef0123456789abcdef01234567\n  - uses: g/h@main\n  - uses: ./local\n  - uses: docker://alpine\n  - uses: docker://host:5000/img:1\n  - uses: i/j\n  # - uses: k/l@main\n";
        let mut count = Count::default();
        read(&mut count, "w.yml", workflow, None);
        let out = outcome(count, None, false);
        assert_eq!(
            text(&out.report),
            "Action references sensor: 4 finding(s) for the reviewer — each is held against the project's decision on action references:\n  - mixed forms — 1 on a major tag, 1 on a full version tag, 1 on a commit SHA\n  - w.yml:5: 'g/h@main' references 'main', which is neither a version tag nor a commit SHA\n  - w.yml:7: 'docker://alpine' carries no tag\n  - w.yml:9: 'i/j' names no ref\n"
        );
    }

    #[test]
    fn a_require_fails_on_the_other_forms_and_the_missing_dependabot_entry() {
        let mut count = Count::default();
        read(
            &mut count,
            "w.yml",
            b"- uses: a/b@v4\n",
            Some(Form::MajorTag),
        );
        let out = outcome(count, Some(Form::MajorTag), false);
        assert!(out.report.is_empty());
        assert!(
            text(&out.failed.unwrap())
                .contains(".github/dependabot.yml: no 'github-actions' entry")
        );
        let mut count = Count::default();
        read(
            &mut count,
            "w.yml",
            b"- uses: a/b@v4\n",
            Some(Form::MajorTag),
        );
        let out = outcome(count, Some(Form::MajorTag), true);
        assert_eq!(
            text(&out.report),
            "Action references check (major-tag): 1 on a major tag, 0 on a full version tag, 0 on a commit SHA — every reference is in the decided form.\n"
        );
        assert!(keeps_actions(Some(
            b"updates:\n  - package-ecosystem: \"github-actions\"\n"
        )));
        assert!(!keeps_actions(Some(
            b"updates:\n  - package-ecosystem: cargo\n"
        )));
    }
}
