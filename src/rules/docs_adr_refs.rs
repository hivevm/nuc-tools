//! Check 5 of the template's `scripts/check-docs.sh` at v0.5.0: every `ADR-NNNN` and
//! `ADR-draft-<slug>` reference in a text file of the repository names an ADR file that exists in
//! `docs/adr/`. The template and the superseded and rejected ADRs are not read.

use super::{CATEGORY_IDENTIFIERS, Input, Inputs, Rule};
use crate::adr::{self, show};
use crate::bash;
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/adr-refs";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["unmatched"],
    explain: "Every 'ADR-NNNN' and 'ADR-draft-<slug>' reference in a text file names an ADR file \
              that exists in docs/adr/. A follow-up that does not exist yet is described by \
              topic, never by a number. The template and superseded or rejected ADRs are not \
              read: they describe a state of the repository that does not exist. From template \
              release v0.9.0 on, a reference is '<CAT>-<slug>' of a category of the record, with \
              no letter, digit, '-', or '_' beside it, and a code span of Markdown is not read.",
    reads: &[Input::Adr, Input::TextFiles, Input::Release],
    check: |inputs| check(inputs).into(),
};

fn check(inputs: &Inputs) -> Vec<Finding> {
    if inputs.carries_at_least(CATEGORY_IDENTIFIERS) {
        return named(inputs);
    }
    let known = adr::by_key(&inputs.adr);
    let inactive = adr::inactive_paths(&inputs.adr);
    let mut found = Vec::new();
    for (path, text) in &inputs.text_files {
        if path == "docs/adr/template.md" || inactive.contains(path.as_str()) {
            continue;
        }
        for (n, line) in bash::awk_lines(text).into_iter().enumerate() {
            for (_, reference) in adr::references(line) {
                if !known.contains_key(show(adr::reference_key(reference)).as_str()) {
                    found.push(Finding {
                        rule: RULE_ID,
                        reason: "unmatched",
                        file: path.clone(),
                        line: Some(n + 1),
                        message: format!(
                            "reference '{}' matches no ADR file in docs/adr/",
                            show(reference)
                        ),
                    });
                }
            }
        }
    }
    found
}

/// The check from template release v0.9.0 on: each `<CAT>-<slug>` of a category of the record,
/// with no word character beside it and outside the code spans of a Markdown line, names an ADR
/// file of the record.
fn named(inputs: &Inputs) -> Vec<Finding> {
    let known: std::collections::BTreeSet<&str> = adr::category_names(&inputs.adr)
        .into_iter()
        .map(|name| name.strip_suffix(".md").unwrap_or(name))
        .collect();
    let categories = adr::categories(inputs.adr.get("README.md").map(Vec::as_slice));
    let inactive = adr::named_inactive_paths(&inputs.adr);
    let mut found = Vec::new();
    for (path, text) in &inputs.text_files {
        if path == "docs/adr/template.md" || inactive.contains(path.as_str()) {
            continue;
        }
        for (n, raw) in bash::awk_lines(text).into_iter().enumerate() {
            if !adr::holds_reference(raw, &categories) {
                continue;
            }
            let line = if path.ends_with(".md") {
                adr::code_spans_out(raw)
            } else {
                raw.to_vec()
            };
            for reference in adr::named_references(&line, &categories) {
                let reference = show(reference);
                if !known.contains(reference.as_str()) {
                    found.push(Finding {
                        rule: RULE_ID,
                        reason: "unmatched",
                        file: path.clone(),
                        line: Some(n + 1),
                        message: format!(
                            "reference '{reference}' matches no ADR file in docs/adr/"
                        ),
                    });
                }
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::id;

    /// The inputs of ADR files and text files, each ADR a text file too, as the shell reads them.
    fn inputs(adrs: &[(&str, &str)], others: &[(&str, &str)]) -> Inputs {
        let mut inputs = Inputs {
            release: Some(crate::config::Version(0, 8, 1)),
            ..Inputs::default()
        };
        for (name, text) in adrs {
            inputs
                .adr
                .insert((*name).to_owned(), text.as_bytes().to_vec());
            inputs
                .text_files
                .insert(format!("docs/adr/{name}"), text.as_bytes().to_vec());
        }
        for (path, text) in others {
            inputs
                .text_files
                .insert((*path).to_owned(), text.as_bytes().to_vec());
        }
        inputs
    }

    fn found(inputs: &Inputs) -> Vec<(String, Option<usize>, String)> {
        check(inputs)
            .into_iter()
            .map(|f| (f.file, f.line, f.message))
            .collect()
    }

    const ACCEPTED: &str = "# T\n\n- **Status:** 🟢 accepted\n";

    #[test]
    fn references_to_existing_adrs_pass() {
        let text = format!("See {} and {}.\n", id("0001"), id("draft-next"));
        let inputs = inputs(
            &[("0001-a.md", ACCEPTED), ("draft-next.md", ACCEPTED)],
            &[("README.md", &text)],
        );
        assert_eq!(found(&inputs), []);
    }

    #[test]
    fn a_reference_to_a_missing_adr_fails_at_its_line() {
        let text = format!(
            "one\nSee {}, {} and {}.\n",
            id("0001"),
            id("0007"),
            id("draft-gone")
        );
        let found = found(&inputs(
            &[("0001-a.md", ACCEPTED)],
            &[("scripts/x.sh", &text)],
        ));
        // bash: a reference to an ADR that does not exist
        // bash: a draft identifier with no draft file
        assert_eq!(
            found,
            [
                (
                    "scripts/x.sh".to_owned(),
                    Some(2),
                    format!(
                        "reference '{}' matches no ADR file in docs/adr/",
                        id("0007")
                    )
                ),
                (
                    "scripts/x.sh".to_owned(),
                    Some(2),
                    format!(
                        "reference '{}' matches no ADR file in docs/adr/",
                        id("draft-gone")
                    )
                ),
            ]
        );
    }

    #[test]
    fn a_reference_is_the_longest_match_and_needs_no_word_boundary() {
        let text = format!("x{}5 {}-y- {}\n", id("0001"), id("draft-a-b"), id("12"));
        let found = found(&inputs(&[("0001-a.md", ACCEPTED)], &[("a.md", &text)]));
        let messages: Vec<&str> = found.iter().map(|(_, _, m)| m.as_str()).collect();
        assert_eq!(
            messages,
            [format!(
                "reference '{}' matches no ADR file in docs/adr/",
                id("draft-a-b-y")
            )]
        );
    }

    #[test]
    fn the_template_and_inactive_adrs_are_not_read() {
        let missing = format!("{}\n", id("0099"));
        let superseded = format!("- **Status:** ⚪ superseded by {}\n{missing}", id("0002"));
        let rejected = format!("- **Status:** 🔴 rejected\n{missing}");
        let inputs = inputs(
            &[
                ("0001-a.md", &superseded),
                ("0002-b.md", &rejected),
                ("template.md", &missing),
            ],
            &[],
        );
        assert_eq!(found(&inputs), []);
    }
}
