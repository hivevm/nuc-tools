//! Part 3 of the template's `scripts/check-traceability.sh`: every identifier a `Verifies:`
//! marker cites names an ADR of `docs/adr/` that is in force, or a criterion of
//! `docs/SPECIFICATION.md` (ADR-0023).

use super::{Input, Inputs, Rule, traceability_criteria};
use crate::adr;
use crate::finding::Finding;
use std::collections::{BTreeMap, BTreeSet};

pub const RULE_ID: &str = "traceability/citations";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Every identifier a 'Verifies:' marker cites, in a file git does not ignore that is \
              not Markdown, names an ADR that exists in docs/adr/ and is neither superseded nor \
              rejected, or a criterion G-n or Q-n of docs/SPECIFICATION.md. A test that cites \
              what does not exist, or a decision no longer in force, verifies nothing the record \
              still asks for.",
    reads: &[Input::Markers, Input::Adr, Input::Specification],
    check,
};

/// The marker as the bash's grep finds it.
const MARKER: &[u8] = b"Verifies:";

/// One identifier one marker cites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    pub file: String,
    pub line: usize,
    pub id: String,
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let criteria = traceability_criteria::read(inputs.specification.as_deref()).ids;
    let (known, inactive) = record(&inputs.adr);
    citations(&inputs.markers)
        .into_iter()
        .filter_map(|citation| {
            let why = if citation.id.starts_with("ADR-") {
                if !known.contains(&citation.id) {
                    "which does not exist in docs/adr/"
                } else if inactive.contains(&citation.id) {
                    "which is superseded or rejected — a test cannot verify a decision no longer \
                     in force"
                } else {
                    return None;
                }
            } else if criteria.contains(&citation.id) {
                return None;
            } else {
                "which is not a criterion of docs/SPECIFICATION.md"
            };
            Some(Finding {
                rule: RULE_ID,
                file: citation.file,
                line: Some(citation.line),
                message: format!("cites '{}', {why}", citation.id),
            })
        })
        .collect()
}

/// The identifiers of the ADR files, and those of the files whose status line carries 🔴 or ⚪
/// but not 🟢, as the bash's `case` sorts them.
pub fn record(dir: &adr::Dir) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut known = BTreeSet::new();
    let mut inactive = BTreeSet::new();
    for (name, header) in adr::headers(dir) {
        let id = format!("ADR-{}", adr::key_of(name));
        let status = header.field("Status").unwrap_or(b"");
        if !adr::contains(status, "🟢".as_bytes())
            && (adr::contains(status, "🔴".as_bytes()) || adr::contains(status, "⚪".as_bytes()))
        {
            inactive.insert(id.clone());
        }
        known.insert(id);
    }
    (known, inactive)
}

/// Every citation of every file, in the order of the files and their lines, each as often as it
/// is written.
pub fn citations(markers: &BTreeMap<String, Vec<u8>>) -> Vec<Citation> {
    let mut found = Vec::new();
    for (file, bytes) in markers {
        for (i, line) in lines(bytes).enumerate() {
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

/// The lines as grep numbers them: split at each newline, the last one kept when it has bytes.
fn lines(text: &[u8]) -> impl Iterator<Item = &[u8]> {
    let body = text.strip_suffix(b"\n").unwrap_or(text);
    (!text.is_empty())
        .then(|| body.split(|&b| b == b'\n'))
        .into_iter()
        .flatten()
}

/// The identifiers the markers of one line cite, as `grep -oE` finds each marker
/// `Verifies:[[:space:]]*(ID)([[:space:],]+(ID))*`, leftmost and longest, the search going on
/// after it, and `grep -oE ID` takes the identifiers out of it.
fn cited(line: &[u8]) -> Vec<&[u8]> {
    let mut ids = Vec::new();
    let mut from = 0;
    while let Some(at) = adr::find(&line[from..], MARKER) {
        let start = from + at + MARKER.len();
        let mut i = start + line[start..].iter().take_while(|&&b| is_space(b)).count();
        let Some(length) = identifier_at(&line[i..]) else {
            from += at + 1;
            continue;
        };
        ids.push(&line[i..i + length]);
        i += length;
        loop {
            let gap = line[i..]
                .iter()
                .take_while(|&&b| is_space(b) || b == b',')
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

/// The space class of the C locale, which the bash matches with `[[:space:]]`.
fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\x0B' | b'\x0C' | b'\r')
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The marker for the identifiers, assembled so that this file carries no literal one.
    fn mark(ids: &str) -> String {
        format!("{}: {ids}", "Verifies")
    }

    fn adr_id(n: u32) -> String {
        format!("ADR-{n:04}")
    }

    fn adr_file(n: u32, status: &str) -> (String, Vec<u8>) {
        (
            format!("{n:04}-decision.md"),
            format!("# ADR-{n:04}: decision\n\n- **Status:** {status}\n").into_bytes(),
        )
    }

    fn inputs(spec: &str, adrs: &[(u32, &str)], files: &[(&str, &str)]) -> Inputs {
        Inputs {
            specification: Some(spec.as_bytes().to_vec()),
            adr: adrs.iter().map(|&(n, s)| adr_file(n, s)).collect(),
            markers: files
                .iter()
                .map(|&(f, text)| (f.to_owned(), text.as_bytes().to_vec()))
                .collect(),
            ..Inputs::default()
        }
    }

    const SPEC: &str = "## Goals / Success Criteria\n\n- **G-1** — it works.\n\n\
                        ## Quality Goals\n\n- **Q-1** — fast enough.\n";

    fn messages(inputs: &Inputs) -> Vec<String> {
        check(inputs)
            .into_iter()
            .map(|f| format!("{}:{}: {}", f.file, f.line.unwrap(), f.message))
            .collect()
    }

    #[test]
    fn the_cases_of_the_bash_self_test_report_as_it_does() {
        let text = format!("{}\n{}\n", mark(&adr_id(1)), mark("G-1"));
        assert!(
            messages(&inputs(
                SPEC,
                &[(1, "🟢 accepted")],
                &[("tests/t.txt", &text)]
            ))
            .is_empty()
        );
        let text = mark(&format!("{}, G-1 Q-1", adr_id(1)));
        assert!(
            messages(&inputs(
                SPEC,
                &[(1, "🟢 accepted")],
                &[("tests/t.txt", &text)]
            ))
            .is_empty()
        );
        assert_eq!(
            messages(&inputs(SPEC, &[], &[("tests/t.txt", &mark("G-9"))])),
            ["tests/t.txt:1: cites 'G-9', which is not a criterion of docs/SPECIFICATION.md"]
        );
        assert_eq!(
            messages(&inputs(SPEC, &[], &[("tests/t.txt", &mark(&adr_id(42)))])),
            [format!(
                "tests/t.txt:1: cites '{}', which does not exist in docs/adr/",
                adr_id(42)
            )]
        );
        let text = format!("x\n{}\n", mark(&adr_id(1)));
        assert_eq!(
            messages(&inputs(
                SPEC,
                &[(1, "⚪ superseded by ADR-0002"), (2, "🟢 accepted")],
                &[("tests/t.txt", &text)]
            )),
            [format!(
                "tests/t.txt:2: cites '{}', which is superseded or rejected — a test cannot \
                 verify a decision no longer in force",
                adr_id(1)
            )]
        );
    }

    #[test]
    fn a_draft_is_cited_by_its_slug() {
        let draft = format!("{}-{}-next", "ADR", "draft");
        let mut inputs = inputs(SPEC, &[], &[("t.rs", &mark(&draft))]);
        inputs.adr.insert(
            "draft-next.md".to_owned(),
            format!("# {draft}: next\n\n- **Status:** 🟡 proposed\n").into_bytes(),
        );
        assert!(messages(&inputs).is_empty());
        inputs.adr.remove("draft-next.md");
        assert_eq!(
            messages(&inputs),
            [format!(
                "t.rs:1: cites '{draft}', which does not exist in docs/adr/"
            )]
        );
    }

    #[test]
    fn a_marker_reads_as_the_bash_pattern_reads_it() {
        // `V:` stands for the marker and `@` for `ADR-`, so that this file carries neither.
        let ids = |text: &str| -> Vec<String> {
            let line = text
                .replace("V:", &format!("{}:", "Verifies"))
                .replace('@', "ADR-");
            cited(line.as_bytes())
                .into_iter()
                .map(|id| String::from_utf8_lossy(id).replace("ADR-", "@"))
                .collect()
        };
        assert_eq!(ids("x V: G-1, Q-2\tG-3 and G-4"), ["G-1", "Q-2", "G-3"]);
        assert_eq!(ids("V:G-1,,G-2"), ["G-1", "G-2"]);
        assert_eq!(ids("V: G-12x G-2"), ["G-12"]);
        assert_eq!(ids("V: @00012 G-1"), ["@0001"]);
        assert!(ids("V: @12 G-1").is_empty());
        assert_eq!(ids("V: none; V: Q-1 V: G-2"), ["Q-1", "G-2"]);
        assert!(ids("V: g-1 verifies: G-1").is_empty());
        assert_eq!(ids("V:\r\x0bG-1\r"), ["G-1"]);
        assert_eq!(ids("V: @draft-a-b- G-1"), ["@draft-a-b"]);
    }

    #[test]
    fn a_status_with_green_is_in_force_and_one_without_a_status_line_is_known() {
        let (known, inactive) = record(
            &[
                adr_file(1, "🟢 accepted, once ⚪"),
                adr_file(2, "🔴 rejected"),
                ("0003-x.md".to_owned(), b"# ADR-0003: x\n".to_vec()),
            ]
            .into_iter()
            .collect(),
        );
        assert_eq!(known.len(), 3);
        assert_eq!(inactive.into_iter().collect::<Vec<_>>(), ["ADR-0002"]);
    }

    #[test]
    fn without_a_specification_every_criterion_cited_is_reported() {
        let mut inputs = inputs(SPEC, &[], &[("t.rs", &mark("G-1"))]);
        inputs.specification = None;
        assert_eq!(
            messages(&inputs),
            ["t.rs:1: cites 'G-1', which is not a criterion of docs/SPECIFICATION.md"]
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
