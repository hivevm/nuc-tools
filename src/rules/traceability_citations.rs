//! Part 3 of the template's `scripts/check-traceability.sh`: every identifier a `Verifies:`
//! marker cites names an ADR of `docs/adr/` that is in force, or a criterion of
//! `docs/SPECIFICATION.md` (ADR-0023).

use super::{Input, Inputs, Rule};
use crate::adr;
use crate::finding::Finding;
use crate::traceability;
use std::collections::BTreeSet;

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

fn check(inputs: &Inputs) -> Vec<Finding> {
    let criteria = traceability::criteria(inputs.specification.as_deref()).ids;
    let (known, inactive) = record(&inputs.adr);
    traceability::citations(&inputs.markers)
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
fn record(dir: &adr::Dir) -> (BTreeSet<String>, BTreeSet<String>) {
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

    /// The specification of the bash self-test's fixtures, with the goals given and the quality
    /// goal given.
    fn spec(goals: &[&str], quality: &str) -> String {
        let goals: String = goals.iter().map(|g| format!("- {g}\n")).collect();
        format!(
            "# Specification\n\n## Goals / Success Criteria\n\n{goals}\n## Quality Goals\n\n\
             {quality}\n\n## Non-Goals\n\n- none\n"
        )
    }

    const Q1: &str = "- **Q-1** — fast enough.";

    #[test]
    fn the_cases_of_the_bash_self_test_report_as_it_does() {
        let g1 = spec(&["**G-1** — it works."], Q1);
        let g2 = spec(&["**G-1** — it works.", "**G-2** — it also works."], Q1);
        let text = format!("{}\n{}\n", mark(&adr_id(1)), mark("G-1"));
        assert!(
            messages(&inputs(
                &g1,
                &[(1, "🟢 accepted")],
                &[("tests/t.txt", &text)]
            ))
            .is_empty()
        );
        let text = mark(&format!("{}, G-1 G-2", adr_id(1)));
        assert!(
            messages(&inputs(
                &g2,
                &[(1, "🟢 accepted")],
                &[("tests/t.txt", &text)]
            ))
            .is_empty()
        );
        // A G-n filed under Quality Goals is still a criterion a marker may cite.
        let wrong = spec(
            &["**G-1** — it works."],
            "- **G-2** — a goal filed under quality.",
        );
        let text = mark("G-1 G-2");
        assert!(
            messages(&inputs(
                &wrong,
                &[(1, "🟡 proposed")],
                &[("tests/t.txt", &text)]
            ))
            .is_empty()
        );
        assert_eq!(
            messages(&inputs(
                &g1,
                &[(1, "🟡 proposed")],
                &[("tests/t.txt", &mark("G-1 G-9"))]
            )),
            ["tests/t.txt:1: cites 'G-9', which is not a criterion of docs/SPECIFICATION.md"]
        );
        let text = mark(&format!("G-1 {}", adr_id(42)));
        assert_eq!(
            messages(&inputs(
                &g1,
                &[(1, "🟡 proposed")],
                &[("tests/t.txt", &text)]
            )),
            [format!(
                "tests/t.txt:1: cites '{}', which does not exist in docs/adr/",
                adr_id(42)
            )]
        );
        let superseded = format!("⚪ superseded by {}", adr_id(2));
        let text = mark(&format!("G-1 {} {}", adr_id(1), adr_id(2)));
        assert_eq!(
            messages(&inputs(
                &g1,
                &[(1, &superseded), (2, "🟢 accepted")],
                &[("tests/t.txt", &text)]
            )),
            [format!(
                "tests/t.txt:1: cites '{}', which is superseded or rejected — a test cannot \
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
}
