//! Check 7 of the template's `scripts/check-docs.sh` at v0.5.0: every ADR writes its header fields
//! as the list item `- **<Name>:** …`, its status line carries exactly one legend emoji with the
//! word that belongs to it, and no ADR keeps the instruction paragraph of `docs/adr/template.md`.

use super::{Inputs, Rule};
use crate::adr;
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/adr-header";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Every ADR's '**Status:**' line carries exactly one legend emoji and the word that \
              belongs to it, every header field is the list item '- **<Name>:** …' of \
              docs/adr/template.md, and no ADR still contains the template's instruction \
              paragraph. A leftover list of alternatives or a word contradicting its emoji \
              survives the index check, and the word is what a reader believes; a field in \
              another form is skipped by every script that reads the header.",
    check,
};

/// The word that belongs to each emoji of the legend.
fn word_of(emoji: char) -> &'static str {
    match emoji {
        '🟢' => "accepted",
        '🟡' => "proposed",
        '🔴' => "rejected",
        _ => "superseded",
    }
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let mut found = Vec::new();
    for (name, header) in adr::headers(&inputs.adr) {
        let path = format!("docs/adr/{name}");
        let at = |line, message| Finding {
            rule: RULE_ID,
            file: path.clone(),
            line,
            message,
        };
        for (n, _) in &header.stray {
            found.push(at(Some(*n), format!(
                "{name}:{n}: a header field not written as the list item '- **<Name>:** …' of template.md — every script reads only that form"
            )));
        }
        let status = header.field("Status").unwrap_or("");
        if let (Some(emoji), count) = adr::status_emoji(status) {
            let after = status.split_once(emoji).map_or("", |(_, rest)| rest);
            let word = after.split_whitespace().next().unwrap_or("");
            let expected = word_of(emoji);
            if count > 1 {
                found.push(at(
                    None,
                    format!("{name}: status line carries {count} status emoji — keep exactly one"),
                ));
            } else if word != expected {
                found.push(at(
                    None,
                    format!(
                        "{name}: status line reads '{emoji} {}', but {emoji} means '{expected}'",
                        if word.is_empty() { "<nothing>" } else { word }
                    ),
                ));
            }
        }
        if header.needle {
            found.push(at(
                None,
                format!(
                    "{name}: still carries the instruction paragraph from template.md — delete it"
                ),
            ));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adr::{Dir, File};

    const TEMPLATE: &str =
        "# ADR-draft-<slug>: T\n\n*Complete the header and delete what does not apply. More.*\n";

    fn messages(adr_text: &str) -> Vec<String> {
        let adr: Dir = [("template.md", TEMPLATE), ("0001-a.md", adr_text)]
            .into_iter()
            .map(|(n, t)| {
                (
                    n.to_owned(),
                    File {
                        text: t.to_owned(),
                        listed: true,
                    },
                )
            })
            .collect();
        check(&Inputs {
            adr,
            ..Inputs::default()
        })
        .into_iter()
        .map(|f| f.message)
        .collect()
    }

    #[test]
    fn a_clean_header_passes() {
        assert_eq!(
            messages(
                "# ADR-0001: A\n\n- **Status:** 🟢 accepted\n- **Date:** 2026-10-31\n\n## Context\n\nThe **Date:** field is named in the body.\n"
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn two_emoji_or_a_wrong_word_fail() {
        assert_eq!(
            messages("- **Status:** 🟡 proposed | 🟢 accepted\n"),
            ["0001-a.md: status line carries 2 status emoji — keep exactly one"]
        );
        assert_eq!(
            messages("- **Status:** 🟢 proposed\n"),
            ["0001-a.md: status line reads '🟢 proposed', but 🟢 means 'accepted'"]
        );
        assert_eq!(
            messages("- **Status:** ⚪\n"),
            ["0001-a.md: status line reads '⚪ <nothing>', but ⚪ means 'superseded'"]
        );
        assert_eq!(
            messages("- **Status:** ⚪ superseded by ADR-0002\n"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_status_line_without_an_emoji_is_the_index_rule_s() {
        assert_eq!(messages("- **Status:** accepted\n"), Vec::<String>::new());
    }

    #[test]
    fn a_field_in_another_form_fails_at_its_line() {
        let found = check_at("# ADR-0001\n\n**Status:** 🟢 accepted\n");
        assert_eq!(found.line, Some(3));
        assert!(
            found
                .message
                .starts_with("0001-a.md:3: a header field not written as the list item")
        );
    }

    #[test]
    fn the_instruction_paragraph_fails() {
        assert_eq!(
            messages(
                "- **Status:** 🟢 accepted\n\n*Complete the header and delete what does not apply.*\n"
            ),
            ["0001-a.md: still carries the instruction paragraph from template.md — delete it"]
        );
    }

    fn check_at(text: &str) -> Finding {
        let adr: Dir = [(
            "0001-a.md".to_owned(),
            File {
                text: text.to_owned(),
                listed: true,
            },
        )]
        .into();
        check(&Inputs {
            adr,
            ..Inputs::default()
        })
        .remove(0)
    }
}
