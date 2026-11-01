//! The traceability of the record, as the template's `scripts/check-traceability.sh` reads it: the
//! criteria `docs/SPECIFICATION.md` lists under its two criteria sections, and the identifiers the
//! `Verifies:` markers cite. The rules of the area `traceability` share these readings, as the
//! rules of the record share `crate::adr` (ADR-0023).

use crate::adr;
use crate::bash;
use std::collections::BTreeMap;

pub const SPECIFICATION: &str = "docs/SPECIFICATION.md";

/// The criteria the specification lists, and what is wrong with the list, each by its line.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Criteria {
    /// Every identifier, in the order of the specification, each as often as it appears.
    pub ids: Vec<String>,
    /// Each problem of the list, by its line when it has one, as the bash words it.
    pub problems: Vec<(Option<usize>, String)>,
}

/// The criteria of the specification, read as the bash reads it: line by line, a heading of two
/// hashes opening or closing a section, each list item of a section checked for its identifier.
pub fn criteria(specification: Option<&[u8]>) -> Criteria {
    let mut criteria = Criteria::default();
    let Some(text) = specification else {
        criteria
            .problems
            .push((None, "not found — the criteria cannot be read".to_owned()));
        return criteria;
    };
    let mut section = None;
    for (i, line) in bash::awk_lines(text).into_iter().enumerate() {
        if let Some(rest) = line.strip_prefix(b"##")
            && rest.first().copied().is_some_and(bash::is_space)
        {
            section = if line.starts_with(b"## Goals / Success Criteria") {
                Some(b'G')
            } else if line.starts_with(b"## Quality Goals") {
                Some(b'Q')
            } else {
                None
            };
            continue;
        }
        let Some(section) = section else { continue };
        let Some(body) = item_body(line) else {
            continue;
        };
        let lineno = Some(i + 1);
        match identifier(body) {
            Some((kind, id)) => {
                if kind != section {
                    criteria.problems.push((
                        lineno,
                        format!(
                            "'{id}' is listed under the wrong section — G-n belongs under Goals / \
                             Success Criteria, Q-n under Quality Goals"
                        ),
                    ));
                }
                if criteria.ids.contains(&id) {
                    criteria.problems.push((
                        lineno,
                        format!("identifier '{id}' appears twice — identifiers are never reused"),
                    ));
                }
                criteria.ids.push(id);
            }
            None => {
                // The bash cuts the body at 60 characters of the C locale, which are bytes.
                let excerpt = String::from_utf8_lossy(&body[..body.len().min(60)]);
                criteria.problems.push((
                    lineno,
                    format!(
                        "list item without a bold {}-n identifier: '{excerpt}'",
                        char::from(section)
                    ),
                ));
            }
        }
    }
    criteria
}

/// The text of a list item after its marker and the space that follows it, as
/// `^[[:space:]]*([-*+]|[0-9]+\.)[[:space:]]+(.*)$` captures it; none for any other line.
fn item_body(line: &[u8]) -> Option<&[u8]> {
    let start = line.iter().position(|&b| !bash::is_space(b))?;
    let rest = &line[start..];
    let rest = match rest.first()? {
        b'-' | b'*' | b'+' => &rest[1..],
        b'0'..=b'9' => {
            let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
            rest[digits..].strip_prefix(b".")?
        }
        _ => return None,
    };
    let spaces = rest.iter().take_while(|&&b| bash::is_space(b)).count();
    (spaces > 0).then(|| &rest[spaces..])
}

/// The section letter and the identifier a body opens with, as `^\*\*([GQ])-([0-9]+)\*\*`
/// matches it; the digits as written.
fn identifier(body: &[u8]) -> Option<(u8, String)> {
    let rest = body.strip_prefix(b"**")?;
    let (&kind, rest) = rest.split_first()?;
    if kind != b'G' && kind != b'Q' {
        return None;
    }
    let rest = rest.strip_prefix(b"-")?;
    let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
    if digits == 0 || !rest[digits..].starts_with(b"**") {
        return None;
    }
    let number = std::str::from_utf8(&rest[..digits]).expect("ASCII digits");
    Some((kind, format!("{}-{number}", char::from(kind))))
}

/// The marker as the bash's grep finds it.
const MARKER: &[u8] = b"Verifies:";

/// One identifier one marker cites.
#[derive(Debug, PartialEq, Eq)]
pub struct Citation {
    pub file: String,
    pub line: usize,
    pub id: String,
}

/// Every citation of every file, in the order of the files and their lines, each as often as it
/// is written.
pub fn citations(markers: &BTreeMap<String, Vec<u8>>) -> Vec<Citation> {
    let mut found = Vec::new();
    for (file, bytes) in markers {
        for (i, line) in bash::awk_lines(bytes).into_iter().enumerate() {
            for id in cited(line) {
                found.push(Citation {
                    file: file.clone(),
                    line: i + 1,
                    id: String::from_utf8_lossy(id).into_owned(),
                });
            }
        }
    }
    found
}

/// The identifiers the markers of one line cite, as `grep -oE` finds each marker
/// `Verifies:[[:space:]]*(ID)([[:space:],]+(ID))*`, leftmost and longest, the search going on
/// after it, and `grep -oE ID` takes the identifiers out of it.
fn cited(line: &[u8]) -> Vec<&[u8]> {
    let mut ids = Vec::new();
    let mut from = 0;
    while let Some(at) = bash::find(&line[from..], MARKER) {
        let start = from + at + MARKER.len();
        let mut i = start
            + line[start..]
                .iter()
                .take_while(|&&b| bash::is_space(b))
                .count();
        let Some(length) = identifier_at(&line[i..]) else {
            from += at + 1;
            continue;
        };
        ids.push(&line[i..i + length]);
        i += length;
        loop {
            let gap = line[i..]
                .iter()
                .take_while(|&&b| bash::is_space(b) || b == b',')
                .count();
            match (gap > 0).then(|| identifier_at(&line[i + gap..])).flatten() {
                Some(length) => {
                    ids.push(&line[i + gap..i + gap + length]);
                    i += gap + length;
                }
                None => break,
            }
        }
        from = i;
    }
    ids
}

/// The length of the identifier at the start of the bytes, as `ADR-([0-9]{4}|draft-<slug>)` or
/// `[GQ]-[0-9]+` match there, longest.
fn identifier_at(text: &[u8]) -> Option<usize> {
    if let Some(length) = adr::reference_at(text) {
        return Some(length);
    }
    match text {
        [b'G' | b'Q', b'-', rest @ ..] => {
            let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
            (digits > 0).then_some(2 + digits)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{adr_id, id, mark, verifies};

    #[test]
    fn a_marker_reads_as_the_bash_pattern_reads_it() {
        let ids = |line: String| -> Vec<String> {
            cited(line.as_bytes())
                .into_iter()
                .map(|id| String::from_utf8_lossy(id).into_owned())
                .collect()
        };
        let v = verifies();
        assert_eq!(
            ids(format!("x {v} G-1, Q-2\tG-3 and G-4")),
            ["G-1", "Q-2", "G-3"]
        );
        assert_eq!(ids(format!("{v}G-1,,G-2")), ["G-1", "G-2"]);
        assert_eq!(ids(format!("{v} G-12x G-2")), ["G-12"]);
        assert_eq!(ids(format!("{v} {}2 G-1", adr_id(1))), [adr_id(1)]);
        assert!(ids(format!("{v} {} G-1", id("12"))).is_empty());
        assert_eq!(ids(format!("{v} none; {v} Q-1 {v} G-2")), ["Q-1", "G-2"]);
        assert!(ids(format!("{v} g-1 verifies: G-1")).is_empty());
        assert_eq!(ids(format!("{v}\r\x0bG-1\r")), ["G-1"]);
        assert_eq!(
            ids(format!("{v} {}- G-1", id("draft-a-b"))),
            [id("draft-a-b")]
        );
    }

    #[test]
    fn lines_are_numbered_as_grep_numbers_them() {
        let text = format!("a\n\n{}", mark("G-9"));
        assert_eq!(
            citations(&[("f".to_owned(), text.into_bytes())].into_iter().collect()),
            [Citation {
                file: "f".to_owned(),
                line: 3,
                id: "G-9".to_owned()
            }]
        );
    }
}
