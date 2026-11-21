//! `nuc sense revision-due`: whether a design revision is due, as the template's
//! `scripts/sensor-revision-due.sh` of the tree's release reports it (NUC-design-is-revised).
//!
//! The sensor reads the `Last design revision` line of `docs/ARCHITECTURE.md`; the shell asks git
//! for the commit that set its date, the count of the changes outside `docs/` since, and, where
//! the revision is due, the top-level entries those changes touched. The report is the bash's,
//! byte for byte: the first line, and where it is due one line per entry that still exists. It is
//! the bash's report under `LC_ALL=C`, as the parity harness runs it: the bash sets no locale, and
//! in another its `sort` orders tied entries, and `[[:space:]]` matches bytes, otherwise.

use std::collections::BTreeMap;

use super::{Refusal, Sensor};
use crate::bash;
use crate::config::Version;
use crate::rules::CATEGORY_IDENTIFIERS;

/// The sensor, which the template has carried from its first release, alike up to `ORDINALS`.
pub const SENSOR: Sensor = Sensor {
    name: "revision-due",
    since: Version(0, 1, 0),
    changes: &[ORDINALS, CATEGORY_IDENTIFIERS],
    reasons: &[
        "overview",
        "file",
        "line",
        "unreadable",
        "number",
        "git",
        super::DRIVER,
        super::ENVIRONMENT,
        super::RELEASE,
    ],
};

/// The release from which the line may carry an ordinal after its date, `(2)`, and the commit that
/// set the date is found by the date and its ordinal up to the comma.
pub const ORDINALS: Version = Version(0, 6, 2);

/// The file that holds the line.
pub const OVERVIEW: &str = "docs/ARCHITECTURE.md";

/// The text that starts the line, which `grep -m1 -F` looks for.
const MARK: &[u8] = b"**Last design revision:**";

/// What the line says: the date and its ordinal as written, none for `none yet`; and the number of
/// changes after which the next revision is due, as written and as a number.
#[derive(Debug, PartialEq, Eq)]
pub struct Line {
    pub date: Option<Vec<u8>>,
    pub number: Vec<u8>,
    pub due_after: u64,
}

/// The line of the overview, read as the bash of `release` reads it: the first line that holds the
/// mark, matched against the line's pattern. The overview is none where it is no file.
pub fn read(overview: Option<&[u8]>, release: Version) -> Result<Line, Refusal> {
    let refuse = |reason, message: &str| Refusal {
        sensor: SENSOR.name,
        reason,
        message: format!("{OVERVIEW}: {message}"),
    };
    let overview = overview.ok_or_else(|| {
        refuse(
            "overview",
            "not found — the 'Last design revision' line lives there",
        )
    })?;
    let line = bash::awk_lines(overview)
        .into_iter()
        .find(|line| bash::contains(line, MARK))
        .ok_or_else(|| refuse("line", "has no '**Last design revision:**' line"))?;
    let (date, number) = matched(line, release >= ORDINALS).ok_or_else(|| {
        refuse(
            "unreadable",
            "the 'Last design revision' line is unreadable",
        )
    })?;
    // The bash compares the count with the number in its arithmetic, which a number past the
    // largest it holds wraps around: what it reports then is no comparison nuc can make.
    let due_after = std::str::from_utf8(&number)
        .ok()
        .and_then(|n| n.parse::<i64>().ok())
        .ok_or_else(|| {
            refuse(
                "number",
                "the number of changes is past the largest the bash's arithmetic holds, where it wraps around",
            )
        })?;
    Ok(Line {
        date: (date != b"none yet").then_some(date),
        number,
        due_after: due_after as u64,
    })
}

/// The date and the number of the leftmost match of the line's pattern in the line, as `=~`
/// matches it in the C locale: the mark, POSIX space, the date, its ordinal from `ordinals` on,
/// or `none yet`, a comma, space, `due after`, space, a number without a leading zero, space, and
/// `changes.`.
fn matched(line: &[u8], ordinals: bool) -> Option<(Vec<u8>, Vec<u8>)> {
    (0..line.len())
        .filter(|&at| line[at..].starts_with(MARK))
        .find_map(|at| matched_after(&line[at + MARK.len()..], ordinals))
}

/// The date and the number where the rest of the pattern matches at the start of `rest`.
fn matched_after(rest: &[u8], ordinals: bool) -> Option<(Vec<u8>, Vec<u8>)> {
    let rest = spaces(rest)?;
    let (date, rest) = if let Some(rest) = rest.strip_prefix(b"none yet") {
        (b"none yet".to_vec(), rest)
    } else {
        let after = digits(rest, 4)
            .and_then(|r| r.strip_prefix(b"-"))
            .and_then(|r| digits(r, 2))
            .and_then(|r| r.strip_prefix(b"-"))
            .and_then(|r| digits(r, 2))?;
        let after = match ordinal(after).filter(|_| ordinals) {
            Some(after) if after.starts_with(b",") => after,
            _ => after,
        };
        (rest[..rest.len() - after.len()].to_vec(), after)
    };
    let rest = rest.strip_prefix(b",")?;
    let rest = spaces(rest)?.strip_prefix(b"due after")?;
    let rest = spaces(rest)?;
    let len = rest.iter().take_while(|b| b.is_ascii_digit()).count();
    if len == 0 || rest[0] == b'0' {
        return None;
    }
    let number = rest[..len].to_vec();
    spaces(&rest[len..])?
        .starts_with(b"changes.")
        .then_some((date, number))
}

/// The bytes after `n` ASCII digits at the start, none where there are fewer.
fn digits(text: &[u8], n: usize) -> Option<&[u8]> {
    (text.len() >= n && text[..n].iter().all(u8::is_ascii_digit)).then(|| &text[n..])
}

/// The bytes after an ordinal ` (<n>)` at the start, `n` a number without a leading zero.
fn ordinal(text: &[u8]) -> Option<&[u8]> {
    let rest = text.strip_prefix(b" (")?;
    let len = rest.iter().take_while(|b| b.is_ascii_digit()).count();
    if len == 0 || rest[0] == b'0' {
        return None;
    }
    rest[len..].strip_prefix(b")")
}

/// The bytes after one or more POSIX spaces at the start, none where there is none.
fn spaces(text: &[u8]) -> Option<&[u8]> {
    let n = text.iter().take_while(|&&b| bash::is_space(b)).count();
    (n > 0).then(|| &text[n..])
}

/// The pattern of `git log -G` that finds the commits whose diff of the overview adds or removes
/// the line with this date: up to the comma, its brackets as classes, from `ORDINALS` on; the date
/// alone before.
pub fn anchor_pattern(date: &[u8], release: Version) -> Vec<u8> {
    let mut pattern = b"Last design revision:.. *".to_vec();
    if release >= ORDINALS {
        for &b in date {
            match b {
                b'(' => pattern.extend_from_slice(b"[(]"),
                b')' => pattern.extend_from_slice(b"[)]"),
                _ => pattern.push(b),
            }
        }
        pattern.push(b',');
    } else {
        pattern.extend_from_slice(date);
    }
    pattern
}

/// The top-level entries the changes touched, each with the number of changes, from the output of
/// `git log --no-merges --name-only --format=%x01`: an entry once per commit, the directory of a
/// path with its slash; ordered as `sort | uniq -c | sort -rn` orders them, the most changes
/// first, ties by the line descending, and each read back as `read -r n entry` reads it. The shell
/// leaves out each that no longer exists.
pub fn scopes(log: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut counts: BTreeMap<Vec<u8>, u64> = BTreeMap::new();
    let mut seen: Vec<Vec<u8>> = Vec::new();
    for line in bash::awk_lines(log) {
        if line == b"\x01" {
            seen.clear();
            continue;
        }
        if line.is_empty() {
            continue;
        }
        let entry = match line.iter().position(|&b| b == b'/') {
            Some(at) => line[..=at].to_vec(),
            None => line.to_vec(),
        };
        if !seen.contains(&entry) {
            *counts.entry(entry.clone()).or_default() += 1;
            seen.push(entry);
        }
    }
    let mut lines: Vec<(u64, Vec<u8>)> = counts
        .into_iter()
        .map(|(entry, n)| {
            let mut line = format!("{n:>7} ").into_bytes();
            line.extend_from_slice(&entry);
            (n, line)
        })
        .collect();
    lines.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    lines
        .into_iter()
        .map(|(_, line)| bash::read_pair(&line))
        .collect()
}

/// Whether the revision is due: the count has reached the number.
pub fn is_due(line: &Line, count: u64) -> bool {
    count >= line.due_after
}

/// The path the bash tests for existence for an entry: the entry without one trailing slash.
pub fn scope_path(entry: &[u8]) -> &[u8] {
    entry.strip_suffix(b"/").unwrap_or(entry)
}

/// The report: the count against the number, and where the revision is due, the entries that
/// still exist with the number of changes each, as `scopes` gives them.
pub fn report(line: &Line, count: u64, scopes: &[(Vec<u8>, Vec<u8>)], release: Version) -> Vec<u8> {
    let from = match &line.date {
        Some(date) => String::from_utf8_lossy(date).into_owned(),
        None => "the first commit".to_owned(),
    };
    let number = String::from_utf8_lossy(&line.number);
    if !is_due(line, count) {
        return format!(
            "Design revision sensor: {count} of {number} changes outside docs/ since {from} — not yet due.\n"
        )
        .into_bytes();
    }
    let cited = if release >= CATEGORY_IDENTIFIERS {
        "NUC-design-is-revised §5"
    } else {
        "ADR-0006"
    };
    let mut out = format!(
        "Design revision sensor: {count} changes outside docs/ since {from}, due after {number} — DUE: run the design-revision skill over the directories they touched, then move the line ({cited}).\n"
    )
    .into_bytes();
    for (n, entry) in scopes {
        out.extend_from_slice(b"  - ");
        out.extend_from_slice(entry);
        out.extend_from_slice(b" (");
        out.extend_from_slice(n);
        out.extend_from_slice(if n == b"1" {
            b" change)\n"
        } else {
            b" changes)\n"
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEWEST: Version = Version(0, 12, 0);

    fn line(text: &str, release: Version) -> Result<Line, &'static str> {
        read(Some(text.as_bytes()), release).map_err(|refusal| refusal.reason)
    }

    #[test]
    fn the_line_reads_its_date_ordinal_and_number() {
        let read = line(
            "x\n> **Last design revision:** 2026-11-18 (2), due after 20 changes. The\n",
            NEWEST,
        )
        .unwrap();
        assert_eq!(read.date.as_deref(), Some(&b"2026-11-18 (2)"[..]));
        assert_eq!(read.number, b"20");
        assert_eq!(read.due_after, 20);
        let none = line(
            "**Last design revision:**\tnone yet,  due after 50 changes.",
            NEWEST,
        );
        assert_eq!(none.unwrap().date, None);
    }

    #[test]
    fn an_ordinal_is_unreadable_before_it_was_written() {
        let text = "**Last design revision:** 2026-11-18 (2), due after 20 changes.";
        assert_eq!(line(text, Version(0, 6, 1)), Err("unreadable"));
        assert!(line(text, ORDINALS).is_ok());
    }

    #[test]
    fn a_missing_overview_line_or_shape_is_refused_by_its_reason() {
        assert_eq!(read(None, NEWEST).map_err(|r| r.reason), Err("overview"));
        assert_eq!(line("no line here\n", NEWEST), Err("line"));
        // The first line with the mark is the one read, even where a later one would match.
        assert_eq!(
            line(
                "**Last design revision:** soon\n**Last design revision:** none yet, due after 5 changes.\n",
                NEWEST
            ),
            Err("unreadable")
        );
        assert_eq!(
            line(
                "**Last design revision:** none yet, due after 05 changes.",
                NEWEST
            ),
            Err("unreadable")
        );
        assert_eq!(
            line(
                "**Last design revision:** none yet, due after 99999999999999999999 changes.",
                NEWEST
            ),
            Err("number")
        );
    }

    #[test]
    fn a_later_mark_on_the_line_matches_where_the_first_does_not() {
        let read = line(
            "**Last design revision:** x **Last design revision:** none yet, due after 3 changes.",
            NEWEST,
        )
        .unwrap();
        assert_eq!(read.number, b"3");
    }

    #[test]
    fn the_anchor_takes_the_ordinal_up_to_the_comma_from_its_release() {
        assert_eq!(
            anchor_pattern(b"2026-11-18 (2)", NEWEST),
            b"Last design revision:.. *2026-11-18 [(]2[)],".to_vec()
        );
        assert_eq!(
            anchor_pattern(b"2026-11-18", Version(0, 6, 1)),
            b"Last design revision:.. *2026-11-18".to_vec()
        );
    }

    #[test]
    fn scopes_count_each_entry_once_a_commit_and_order_as_sort_rn() {
        let log =
            b"\x01\n\nsrc/a.rs\nsrc/b.rs\nREADME.md\n\x01\n\nsrc/c.rs\nscripts/x.sh\n\x01\n\nb\n";
        let got: Vec<(String, String)> = scopes(log)
            .into_iter()
            .map(|(n, e)| (String::from_utf8(n).unwrap(), String::from_utf8(e).unwrap()))
            .collect();
        let want = [
            ("2", "src/"),
            ("1", "scripts/"),
            ("1", "b"),
            ("1", "README.md"),
        ];
        let want: Vec<(String, String)> = want
            .iter()
            .map(|(n, e)| (n.to_string(), e.to_string()))
            .collect();
        assert_eq!(got, want);
    }

    #[test]
    fn the_report_cites_the_adr_of_its_release_and_counts_changes() {
        let due = Line {
            date: Some(b"2026-01-01".to_vec()),
            number: b"2".to_vec(),
            due_after: 2,
        };
        let scopes = vec![
            (b"2".to_vec(), b"src/".to_vec()),
            (b"1".to_vec(), b"x".to_vec()),
        ];
        let out = String::from_utf8(report(&due, 3, &scopes, NEWEST)).unwrap();
        assert_eq!(
            out,
            "Design revision sensor: 3 changes outside docs/ since 2026-01-01, due after 2 — DUE: run the design-revision skill over the directories they touched, then move the line (NUC-design-is-revised §5).\n  - src/ (2 changes)\n  - x (1 change)\n"
        );
        let old = String::from_utf8(report(&due, 3, &[], Version(0, 8, 1))).unwrap();
        assert!(old.ends_with("(ADR-0006).\n"));
        let not_yet = Line {
            date: None,
            number: b"50".to_vec(),
            due_after: 50,
        };
        assert_eq!(
            String::from_utf8(report(&not_yet, 7, &[], NEWEST)).unwrap(),
            "Design revision sensor: 7 of 50 changes outside docs/ since the first commit — not yet due.\n"
        );
    }
}
