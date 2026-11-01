//! Part 1 of the template's `scripts/check-traceability.sh`: every list item under the two
//! criteria sections of `docs/SPECIFICATION.md` starts with a bold identifier of its section,
//! and no identifier appears twice (ADR-0023).

use super::{Input, Inputs, Rule};
use crate::finding::Finding;
use crate::traceability::{self, SPECIFICATION};

pub const RULE_ID: &str = "traceability/criteria";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Every list item under '## Goals / Success Criteria' and '## Quality Goals' of \
              docs/SPECIFICATION.md starts with a bold identifier, **G-n** under Goals and **Q-n** \
              under Quality Goals, and no identifier appears twice. A test cites a criterion by \
              that identifier, so an item without one can never be verified, and an identifier \
              used twice makes a citation name two criteria.",
    reads: &[Input::Specification],
    check,
};

/// The criteria the specification lists, and what is wrong with the list.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Criteria {
    /// Every identifier, in the order of the specification, each as often as it appears.
    pub ids: Vec<String>,
    pub findings: Vec<Finding>,
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    read(inputs.specification.as_deref()).findings
}

/// The criteria of the specification and their findings, as the bash reports them.
pub fn read(specification: Option<&[u8]>) -> Criteria {
    let read = traceability::criteria(specification);
    Criteria {
        ids: read.ids,
        findings: read
            .problems
            .into_iter()
            .map(|(line, message)| finding(line, message))
            .collect(),
    }
}

fn finding(line: Option<usize>, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        file: SPECIFICATION.to_owned(),
        line,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The specification of the bash self-test's fixtures: the goals given, one quality goal
    /// unless another is given, and a section after them.
    fn spec(goals: &[&str], quality: &str) -> Vec<u8> {
        let mut text = String::from("# Specification\n\n## Goals / Success Criteria\n\n");
        for goal in goals {
            text.push_str(&format!("- {goal}\n"));
        }
        text.push_str(&format!(
            "\n## Quality Goals\n\n{quality}\n\n## Non-Goals\n\n- none\n"
        ));
        text.into_bytes()
    }

    const Q1: &str = "- **Q-1** — fast enough.";

    fn messages(text: &[u8]) -> Vec<String> {
        read(Some(text))
            .findings
            .into_iter()
            .map(|f| f.message)
            .collect()
    }

    #[test]
    fn the_cases_of_the_bash_self_test_report_as_it_does() {
        assert!(messages(&spec(&["**G-1** — it works."], Q1)).is_empty());
        assert!(
            messages(&spec(
                &["**G-1** — it works.", "**G-2** — it also works."],
                Q1
            ))
            .is_empty()
        );
        // A wrapped criterion: the continuation is no list item.
        assert!(
            messages(&spec(
                &["**G-1** — a criterion whose text\n  wraps onto a second line."],
                Q1
            ))
            .is_empty()
        );
        assert_eq!(
            messages(&spec(&["it works, but carries no identifier."], Q1)),
            ["list item without a bold G-n identifier: 'it works, but carries no identifier.'"]
        );
        assert_eq!(
            messages(&spec(
                &["**G-1** — it works.", "**G-1** — it works twice."],
                Q1
            )),
            ["identifier 'G-1' appears twice — identifiers are never reused"]
        );
        assert_eq!(
            messages(&spec(
                &["**G-1** — it works."],
                "- **G-2** — a goal filed under quality."
            )),
            [
                "'G-2' is listed under the wrong section — G-n belongs under Goals / Success \
              Criteria, Q-n under Quality Goals"
            ]
        );
    }

    #[test]
    fn the_identifiers_are_collected_in_order_each_as_often_as_listed() {
        let criteria = read(Some(&spec(
            &["**G-1** — a.", "**G-01** — b.", "**G-1** — c."],
            Q1,
        )));
        assert_eq!(criteria.ids, ["G-1", "G-01", "G-1", "Q-1"]);
        assert_eq!(criteria.findings.len(), 1);
        assert_eq!(criteria.findings[0].line, Some(7));
    }

    #[test]
    fn a_missing_specification_is_reported_without_a_line() {
        let criteria = read(None);
        assert!(criteria.ids.is_empty());
        assert_eq!(criteria.findings[0].file, SPECIFICATION);
        assert_eq!(criteria.findings[0].line, None);
        assert_eq!(
            criteria.findings[0].message,
            "not found — the criteria cannot be read"
        );
    }

    #[test]
    fn list_items_are_read_as_the_bash_pattern_reads_them() {
        let text = b"## Goals / Success Criteria\n\
            1. **G-1** - numbered.\n\
            \t*\t**G-2** - tabs.\n\
            + **G-3** - plus.\n\
            -**G-4** - no space after the marker, no item.\n\
            ---\n\
            12) **G-5** - not a marker.\n\
            - \n";
        let criteria = read(Some(text));
        assert_eq!(criteria.ids, ["G-1", "G-2", "G-3"]);
        assert_eq!(
            criteria
                .findings
                .iter()
                .map(|f| (f.line, f.message.as_str()))
                .collect::<Vec<_>>(),
            [(Some(8), "list item without a bold G-n identifier: ''")]
        );
    }

    #[test]
    fn a_heading_of_two_hashes_and_a_space_opens_or_closes_a_section() {
        let text = b"- **Q-9** \xe2\x80\x94 before any section.\n\
            ## Quality Goals (in priority order)\n\
            - **Q-1** \xe2\x80\x94 kept.\n\
            ### A subsection stays in the section\n\
            - **Q-2** \xe2\x80\x94 kept.\n\
            ##No space: no heading, still the section\n\
            - **Q-3** \xe2\x80\x94 kept.\n\
            ##\tOther\n\
            - left out\n\
            ## Goals / Success Criteria\r\n\
            - **G-1** \xe2\x80\x94 a carriage return is space.\r\n";
        let criteria = read(Some(text));
        assert_eq!(criteria.ids, ["Q-1", "Q-2", "Q-3", "G-1"]);
        assert!(criteria.findings.is_empty());
    }

    #[test]
    fn the_excerpt_is_sixty_bytes_and_a_cut_character_is_shown_replaced() {
        let body = format!("{}é and more", "x".repeat(59));
        let text = format!("## Goals / Success Criteria\n- {body}\n");
        let message = &messages(text.as_bytes())[0];
        assert_eq!(
            message,
            &format!(
                "list item without a bold G-n identifier: '{}\u{FFFD}'",
                "x".repeat(59)
            )
        );
    }

    #[test]
    fn the_last_line_counts_without_a_newline_and_an_empty_file_lists_nothing() {
        let criteria = read(Some(b"## Quality Goals\n- **Q-1** \xe2\x80\x94 last"));
        assert_eq!(criteria.ids, ["Q-1"]);
        assert_eq!(read(Some(b"")), Criteria::default());
    }
}
