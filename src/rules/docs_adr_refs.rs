//! Check 5 of the template's `scripts/check-docs.sh` at v0.5.0: every `ADR-NNNN` and
//! `ADR-draft-<slug>` reference in a text file of the repository names an ADR file that exists in
//! `docs/adr/`. The template and the superseded and rejected ADRs are not read.

use super::{Input, Inputs, Rule};
use crate::adr::{self, show};
use crate::bash;
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/adr-refs";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Every 'ADR-NNNN' and 'ADR-draft-<slug>' reference in a text file names an ADR file \
              that exists in docs/adr/. A follow-up that does not exist yet is described by \
              topic, never by a number. The template and superseded or rejected ADRs are not \
              read: they describe a state of the repository that does not exist.",
    reads: &[Input::Adr, Input::TextFiles],
    check: |inputs| check(inputs).into(),
};

fn check(inputs: &Inputs) -> Vec<Finding> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::id;

    /// The inputs of ADR files and text files, each ADR a text file too, as the shell reads them.
    fn inputs(adrs: &[(&str, &str)], others: &[(&str, &str)]) -> Inputs {
        let mut inputs = Inputs::default();
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
