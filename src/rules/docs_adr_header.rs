//! Check 7 of the template's `scripts/check-docs.sh` at v0.5.0: every ADR writes its header fields
//! as the list item `- **<Name>:** …`, its status line carries exactly one legend emoji with the
//! word that belongs to it, and no ADR keeps the instruction paragraph of `docs/adr/template.md`.

use super::{Input, Inputs, Rule};
use crate::adr::{self, show};
use crate::bash;
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
    reads: &[Input::Adr],
    check: |inputs| check(inputs).into(),
};

/// The word that belongs to each emoji of the legend.
fn word_of(emoji: &str) -> &'static str {
    match emoji {
        "🟢" => "accepted",
        "🟡" => "proposed",
        "🔴" => "rejected",
        _ => "superseded",
    }
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let mut found = Vec::new();
    for (name, header) in adr::headers(&inputs.adr) {
        let path = format!("docs/adr/{name}");
        let at = |line, message: &str| Finding {
            rule: RULE_ID,
            file: path.clone(),
            line,
            message: message.to_owned(),
        };
        for stray in &header.stray {
            // '<line number>:<line>' as the awk prints it; the bash names the part before ':'.
            let before = stray.split(|&b| b == b':').next().unwrap_or(stray);
            let line = show(before).parse().ok();
            found.push(at(line, "a header field not written as the list item '- **<Name>:** …' of template.md — every script reads only that form"));
        }
        let status = header.field("Status").unwrap_or(b"");
        if let (Some(emoji), count) = adr::status_emoji(status) {
            let at_emoji = bash::find(status, emoji.as_bytes()).unwrap_or(0);
            let word = bash::first_word(&status[at_emoji + emoji.len()..]);
            let expected = word_of(emoji);
            if count > 1 {
                found.push(at(
                    None,
                    &format!("status line carries {count} status emoji — keep exactly one"),
                ));
            } else if word != expected.as_bytes() {
                let word = if word.is_empty() {
                    "<nothing>".to_owned()
                } else {
                    show(word)
                };
                found.push(at(
                    None,
                    &format!("status line reads '{emoji} {word}', but {emoji} means '{expected}'"),
                ));
            }
        }
        if header.needle {
            found.push(at(
                None,
                "still carries the instruction paragraph from template.md — delete it",
            ));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adr::Dir;

    const TEMPLATE: &str =
        "# ADR-draft-<slug>: T\n\n*Complete the header and delete what does not apply. More.*\n";

    fn inputs(adr_text: &[u8]) -> Inputs {
        let adr: Dir = [
            ("template.md".to_owned(), TEMPLATE.as_bytes().to_vec()),
            ("0001-a.md".to_owned(), adr_text.to_vec()),
        ]
        .into();
        Inputs {
            adr,
            ..Inputs::default()
        }
    }

    fn messages(adr_text: &str) -> Vec<String> {
        check(&inputs(adr_text.as_bytes()))
            .into_iter()
            .map(|f| f.message)
            .collect()
    }

    #[test]
    fn a_clean_header_passes() {
        let text = "# ADR-0001: A\n\n- **Status:** 🟢 accepted\n- **Date:** 2026-10-31\n\n## Context\n\nThe **Date:** field is named in the body.\n";
        // bash: a header field quoted in the body of an ADR
        assert_eq!(messages(text), Vec::<String>::new());
    }

    #[test]
    fn two_emoji_or_a_wrong_word_fail() {
        assert_eq!(
            messages("- **Status:** 🟡 proposed | 🟢 accepted\n"),
            ["status line carries 2 status emoji — keep exactly one"]
        );
        assert_eq!(
            messages("- **Status:** 🟢 proposed\n"),
            ["status line reads '🟢 proposed', but 🟢 means 'accepted'"]
        );
        assert_eq!(
            messages("- **Status:** ⚪\n"),
            ["status line reads '⚪ <nothing>', but ⚪ means 'superseded'"]
        );
        assert_eq!(
            messages("- **Status:** ⚪ superseded by ADR-0002\n"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn the_word_is_split_as_read_splits_it_so_a_carriage_return_stays() {
        assert_eq!(
            messages("- **Status:** 🟢 accepted\r\n"),
            ["status line reads '🟢 accepted\r', but 🟢 means 'accepted'"]
        );
        assert_eq!(messages("- **Status:** 🟢\u{a0}accepted\n").len(), 1);
        assert_eq!(
            messages("- **Status:** 🟢\taccepted by X\n"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_status_line_without_an_emoji_is_the_index_rule_s() {
        assert_eq!(messages("- **Status:** accepted\n"), Vec::<String>::new());
    }

    // bash: a header field that is no list item
    #[test]
    fn a_field_in_another_form_fails_at_its_line() {
        let found = check(&inputs(
            b"# ADR-0001\n\n**Status:** \xf0\x9f\x9f\xa2 accepted\n",
        ));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].line, Some(3));
        assert!(
            found[0]
                .message
                .starts_with("a header field not written as the list item")
        );
    }

    #[test]
    fn the_instruction_paragraph_fails() {
        let text =
            "- **Status:** 🟢 accepted\n\n*Complete the header and delete what does not apply.*\n";
        assert_eq!(
            messages(text),
            ["still carries the instruction paragraph from template.md — delete it"]
        );
    }
}
