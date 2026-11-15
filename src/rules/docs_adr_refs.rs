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
    reasons: &["unmatched", "unlisted"],
    explain: "Every 'ADR-NNNN' and 'ADR-draft-<slug>' reference in a text file names an ADR file \
              that exists in docs/adr/. A follow-up that does not exist yet is described by \
              topic, never by a number. The template and superseded or rejected ADRs are not \
              read: they describe a state of the repository that does not exist. From template \
              release v0.9.0 on, a reference is '<CAT>-<slug>' of a category of the record, with \
              no letter, digit, '-', or '_' beside it, and a code span of Markdown is not read. \
              From v0.10.0 on, each rule key of scripts/check-docs.sh the Applies to of an ADR \
              in force names is one the script's header lists, an old key included; a tree \
              without the script has no header to hold it to.",
    reads: &[
        Input::Adr,
        Input::TextFiles,
        Input::Release,
        Input::RuleKeys,
    ],
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

/// The rule keys of `scripts/check-docs.sh` that the *Applies to* of an ADR in force names and the
/// script's header does not list, from template release v0.10.0 on; a tree without the script has
/// no header, as the bash, which reads its own, has one wherever it runs, and none is reported.
fn unlisted_keys(inputs: &Inputs) -> Vec<Finding> {
    let Some(keys) = &inputs.rule_keys else {
        return Vec::new();
    };
    let listed = |key: &str| keys.contains(key);
    let mut found = Vec::new();
    for (name, header) in adr::named_headers(&inputs.adr) {
        let applies = header.field("Applies to").unwrap_or(b"");
        let status = header.field("Status").unwrap_or(b"");
        if !bash::contains(applies, b"the rule")
            || bash::contains(status, "⚪".as_bytes())
            || bash::contains(status, "🔴".as_bytes())
        {
            continue;
        }
        for (path, keys) in adr::rule_paths(applies) {
            if path != "scripts/check-docs.sh" {
                continue;
            }
            // The bash splits the keys at spaces, as an unquoted `$keys` splits.
            let words = keys.iter().flat_map(|key| key.split([' ', '\t', '\n']));
            for key in words.filter(|key| !key.is_empty() && !listed(key)) {
                found.push(Finding {
                    rule: RULE_ID,
                    reason: "unlisted",
                    file: format!("docs/adr/{name}"),
                    line: None,
                    message: format!(
                        "Applies to names the rule '{key}' of scripts/check-docs.sh, which its \
                         header does not list"
                    ),
                });
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
    if inputs.carries_at_least(super::TEMPLATE_FINDINGS) {
        found.extend(unlisted_keys(inputs));
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

    #[test]
    fn from_v0_10_0_a_rule_key_of_check_docs_is_one_its_header_lists() {
        use crate::testing::cat_id;
        let adr = |status: &str, applies: &str| {
            format!("# x\n\n- **Status:** {status}\n- **Applies to:** {applies}\n").into_bytes()
        };
        let keyed = "the rules `docs/links` and `docs/no-such-key` of `scripts/check-docs.sh`, \
                     the rule `build/x` of `scripts/check-build.sh`";
        let run = |keys: Option<&[&str]>, release| -> Vec<String> {
            let inputs = Inputs {
                adr: [
                    (
                        format!("{}.md", cat_id("SEC", "a-rule")),
                        adr("🟢 accepted", keyed),
                    ),
                    (
                        format!("{}.md", cat_id("SEC", "b-rule")),
                        adr("🔴 rejected", keyed),
                    ),
                ]
                .into(),
                rule_keys: keys.map(|k| k.iter().map(|k| (*k).to_owned()).collect()),
                release,
                ..Inputs::default()
            };
            check(&inputs)
                .into_iter()
                .map(|f| format!("{} {}: {}", f.reason, f.file, f.message))
                .collect()
        };
        let missing = format!(
            "unlisted docs/adr/{}.md: Applies to names the rule 'docs/no-such-key' of \
             scripts/check-docs.sh, which its header does not list",
            cat_id("SEC", "a-rule")
        );
        assert!(run(None, None).is_empty());
        assert_eq!(run(Some(&["docs/links"]), None), [missing.as_str()]);
        assert!(run(Some(&["docs/links", "docs/no-such-key"]), None).is_empty());
        assert!(run(None, Some(crate::config::Version(0, 9, 1))).is_empty());
    }
}
