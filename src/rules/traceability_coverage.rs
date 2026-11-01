//! Parts 2 and 4 of the template's `scripts/check-traceability.sh`: every accepted ADR is cited by
//! a `Verifies:` marker or declared not mechanically decidable with a reason, and the criteria no
//! marker cites are pending work, named in the summary and never a finding (ADR-0023).

use super::{Input, Inputs, Outcome, Rule};
use crate::adr;
use crate::bash;
use crate::finding::Finding;
use crate::traceability;
use std::collections::BTreeSet;

pub const RULE_ID: &str = "traceability/coverage";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Every accepted ADR is cited by at least one 'Verifies:' marker in a file git does \
              not ignore that is not Markdown, unless its Enforcement section carries a paragraph \
              starting '**Not mechanically decidable:**' followed by the reason. A decision no test \
              verifies is one nobody notices breaking; one no test can decide says why, so that \
              the gap is a choice and not an oversight. Proposed ADRs are out of scope, and the \
              criteria of the specification no test cites yet are listed in the summary as \
              pending work, never as a finding.",
    reads: &[Input::Markers, Input::Adr, Input::Specification],
    check: outcome,
};

/// The label of the exemption, at the start of a line.
const EXEMPTION: &[u8] = b"**Not mechanically decidable:**";

/// The findings of the rule, and the criteria no test cites yet as a note, the way the bash's
/// closing line names them; they are pending work and change no exit code (ADR-0023).
fn outcome(inputs: &Inputs) -> Outcome {
    let pending = pending(inputs);
    let notes = if pending.is_empty() {
        Vec::new()
    } else {
        vec![format!(
            "{} criteria without a test: {}",
            pending.len(),
            pending.join(" ")
        )]
    };
    Outcome {
        notes,
        ..check(inputs).into()
    }
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let cited = cited(inputs);
    let mut findings = Vec::new();
    let mut accepted = Vec::new();
    let mut exempt = BTreeSet::new();
    for (name, header) in adr::headers(&inputs.adr) {
        let status = header.field("Status").unwrap_or(b"");
        if !bash::contains(status, "🟢".as_bytes()) {
            continue;
        }
        let id = format!("ADR-{}", adr::key_of(name));
        match exemption(&inputs.adr[name]) {
            Exemption::None => {}
            Exemption::Reasoned => {
                exempt.insert(id.clone());
            }
            Exemption::Unreasoned => findings.push(Finding {
                rule: RULE_ID,
                file: format!("docs/adr/{name}"),
                line: None,
                message: "'Not mechanically decidable' declared without a reason — the \
                          paragraph has to say why no test can decide it"
                    .to_owned(),
            }),
        }
        accepted.push(id);
    }
    for id in accepted {
        if !exempt.contains(&id) && !cited.contains(&id) {
            findings.push(Finding {
                rule: RULE_ID,
                file: "docs/adr".to_owned(),
                line: None,
                message: format!(
                    "{id} is accepted but no test cites it — add a 'Verifies' marker or declare \
                     it not mechanically decidable in its Enforcement section"
                ),
            });
        }
    }
    findings
}

/// The criteria no marker cites, in the order of the specification, each as often as it is
/// listed: the pending work the bash's closing line names.
fn pending(inputs: &Inputs) -> Vec<String> {
    let cited = cited(inputs);
    traceability::criteria(inputs.specification.as_deref())
        .ids
        .into_iter()
        .filter(|id| !cited.contains(id))
        .collect()
}

/// Every identifier a marker cites, whether it names anything or not.
fn cited(inputs: &Inputs) -> BTreeSet<String> {
    traceability::citations(&inputs.markers)
        .into_iter()
        .map(|citation| citation.id)
        .collect()
}

/// What the Enforcement section of an ADR declares about its exemption.
#[derive(Debug, PartialEq, Eq)]
enum Exemption {
    None,
    Reasoned,
    Unreasoned,
}

/// The exemption the ADR declares, read as the bash reads it: the lines from the first one that
/// starts with `## Enforcement` to the end, as `sed -n '/^## Enforcement/,$p'` prints them; a
/// line that starts with the label declares it, and one with a byte that is no space after the
/// label gives the reason.
fn exemption(text: &[u8]) -> Exemption {
    let lines = bash::awk_lines(text);
    let Some(start) = lines.iter().position(|l| l.starts_with(b"## Enforcement")) else {
        return Exemption::None;
    };
    let mut declared = Exemption::None;
    for line in &lines[start..] {
        if let Some(rest) = line.strip_prefix(EXEMPTION) {
            if rest.iter().any(|&b| !bash::is_space(b)) {
                return Exemption::Reasoned;
            }
            declared = Exemption::Unreasoned;
        }
    }
    declared
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The marker for the identifiers, assembled so that this file carries no literal one.
    fn mark(ids: &str) -> String {
        format!("{}: {ids}\n", "Verifies")
    }

    fn adr_id(n: u32) -> String {
        format!("ADR-{n:04}")
    }

    /// An ADR as the bash self-test writes it, with the Enforcement section given.
    fn adr_file(n: u32, status: &str, enforcement: &str) -> (String, Vec<u8>) {
        (
            format!("{n:04}-decision.md"),
            format!(
                "# ADR-{n:04}: decision {n}\n\n- **Status:** {status}\n\n## Decision\n\nWe will.\n\n\
                 ## Enforcement\n\n{enforcement}\n"
            )
            .into_bytes(),
        )
    }

    fn inputs(spec: &str, adrs: &[(u32, &str, &str)], marks: &str) -> Inputs {
        Inputs {
            specification: Some(spec.as_bytes().to_vec()),
            adr: adrs.iter().map(|&(n, s, e)| adr_file(n, s, e)).collect(),
            markers: [("tests/t.txt".to_owned(), marks.as_bytes().to_vec())]
                .into_iter()
                .collect(),
            ..Inputs::default()
        }
    }

    const SPEC: &str = "# Specification\n\n## Goals / Success Criteria\n\n- **G-1** — it works.\n\n\
                        ## Quality Goals\n\n- **Q-1** — fast enough.\n\n## Non-Goals\n\n- none\n";

    fn messages(inputs: &Inputs) -> Vec<String> {
        check(inputs)
            .into_iter()
            .map(|f| format!("{}: {}", f.file, f.message))
            .collect()
    }

    fn uncited(n: u32) -> String {
        format!(
            "docs/adr: {} is accepted but no test cites it — add a 'Verifies' marker or declare \
             it not mechanically decidable in its Enforcement section",
            adr_id(n)
        )
    }

    #[test]
    fn the_cases_of_the_bash_self_test_report_as_it_does() {
        let review = "Review only.";
        let both = format!("{}{}", mark(&adr_id(1)), mark("G-1"));
        assert!(messages(&inputs(SPEC, &[(1, "🟢 accepted", review)], &both)).is_empty());
        assert_eq!(
            messages(&inputs(SPEC, &[(1, "🟢 accepted", review)], &mark("G-1"))),
            [uncited(1)]
        );
        assert!(messages(&inputs(SPEC, &[(1, "🟡 proposed", review)], &mark("G-1"))).is_empty());
        let reasoned = "**Not mechanically decidable:** it is a judgement call.";
        assert!(messages(&inputs(SPEC, &[(1, "🟢 accepted", reasoned)], &mark("G-1"))).is_empty());
        let bare = "**Not mechanically decidable:**";
        assert_eq!(
            messages(&inputs(SPEC, &[(1, "🟢 accepted", bare)], &mark("G-1"))),
            [
                "docs/adr/0001-decision.md: 'Not mechanically decidable' declared without a \
                 reason — the paragraph has to say why no test can decide it"
                    .to_owned(),
                uncited(1)
            ]
        );
    }

    #[test]
    fn criteria_without_a_test_are_pending_in_the_order_of_the_specification() {
        let review = "Review only.";
        let inputs = inputs(SPEC, &[(1, "🟢 accepted", review)], &mark(&adr_id(1)));
        assert!(messages(&inputs).is_empty());
        assert_eq!(pending(&inputs), ["G-1", "Q-1"]);
        let spec = SPEC.replace("- **Q-1** — fast enough.", "- **Q-1** — a.\n- **G-1** — b.");
        let twice = Inputs {
            specification: Some(spec.into_bytes()),
            ..inputs.clone()
        };
        assert_eq!(pending(&twice), ["G-1", "Q-1", "G-1"]);
    }

    #[test]
    fn a_citation_counts_whatever_it_names_and_only_where_a_marker_is() {
        // An accepted ADR named only in a document, or by a citation that also names nothing
        // known, counts as cited only by the marker.
        let review = "Review only.";
        let marks = mark(&format!("{} G-9", adr_id(1)));
        assert!(messages(&inputs(SPEC, &[(1, "🟢 accepted", review)], &marks)).is_empty());
        let named = format!("See {} in the text.\n", adr_id(1));
        assert_eq!(
            messages(&inputs(SPEC, &[(1, "🟢 accepted", review)], &named)),
            [uncited(1)]
        );
    }

    #[test]
    fn the_exemption_is_read_from_the_first_enforcement_heading_to_the_end() {
        assert_eq!(
            exemption(b"**Not mechanically decidable:** early\n"),
            Exemption::None
        );
        assert_eq!(
            exemption(b"## Enforcement of the rule\n**Not mechanically decidable:** \t\r\n"),
            Exemption::Unreasoned
        );
        assert_eq!(
            exemption(
                b"## Enforcement\n**Not mechanically decidable:**\n## Later\n\
                  **Not mechanically decidable:** a reason\n"
            ),
            Exemption::Reasoned
        );
        assert_eq!(
            exemption(b"## Enforcement\n **Not mechanically decidable:** indented\n"),
            Exemption::None
        );
        assert_eq!(
            exemption(b"## Enforcement\n**Not mechanically decidable:**x"),
            Exemption::Reasoned
        );
    }

    #[test]
    fn two_files_of_one_number_are_exempt_together_and_reported_once_each() {
        let mut adrs: adr::Dir = [adr_file(1, "🟢 accepted", "Review only.")]
            .into_iter()
            .collect();
        adrs.insert(
            "0001-twin.md".to_owned(),
            b"# ADR-0001: twin\n\n- **Status:** \xf0\x9f\x9f\xa2 accepted\n\n## Enforcement\n\n\
              **Not mechanically decidable:** a reason.\n"
                .to_vec(),
        );
        let inputs = Inputs {
            specification: Some(SPEC.as_bytes().to_vec()),
            adr: adrs,
            ..Inputs::default()
        };
        assert!(messages(&inputs).is_empty());
    }

    #[test]
    fn two_uncited_files_of_one_number_are_reported_once_each() {
        let mut adrs: adr::Dir = [adr_file(1, "🟢 accepted", "Review only.")]
            .into_iter()
            .collect();
        adrs.insert(
            "0001-twin.md".to_owned(),
            "# ADR-0001: twin\n\n- **Status:** 🟢 accepted\n"
                .as_bytes()
                .to_vec(),
        );
        let inputs = Inputs {
            specification: Some(SPEC.as_bytes().to_vec()),
            adr: adrs,
            ..Inputs::default()
        };
        assert_eq!(messages(&inputs), [uncited(1), uncited(1)]);
    }

    #[test]
    fn an_accepted_adr_only_a_proposed_adr_s_test_cites_is_uncited() {
        let review = "Review only.";
        let inputs = inputs(
            SPEC,
            &[(1, "🟢 accepted", review), (2, "🟡 proposed", review)],
            &mark(&adr_id(2)),
        );
        assert_eq!(messages(&inputs), [uncited(1)]);
    }
}
