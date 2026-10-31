//! Check 8 of the template's `scripts/check-docs.sh` at v0.5.0: the `**Supersedes:**` field of an
//! ADR and the `⚪ superseded by ADR-NNNN` status line of every ADR it names agree, both ways. A
//! name that matches no file is the reference rule's.

use super::{Inputs, Rule};
use crate::adr::{self, Header, show};
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/adr-supersessions";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "The '**Supersedes:**' field of a new ADR and the '⚪ superseded by ADR-NNNN' \
              status line of every old one it names, one or several it consolidates, name each \
              other. Both are written in the same pull request but in two files, and the index \
              check sees neither half missing. Every name of a Supersedes field counts, on \
              whichever line it sits.",
    check,
};

/// The key a `superseded by [ADR-<key>` status line names, the bracket optional.
fn successor(status: &[u8]) -> Option<String> {
    let label: &[u8] = b"superseded by ";
    let mut i = 0;
    while let Some(at) = adr::find(&status[i..], label) {
        let after = &status[i + at + label.len()..];
        let after = after.strip_prefix(b"[").unwrap_or(after);
        if let Some(length) = adr::reference_at(after) {
            return Some(show(adr::reference_key(&after[..length])));
        }
        i += at + 1;
    }
    None
}

/// The keys the Supersedes field names, in order.
fn superseded(header: &Header) -> Vec<String> {
    adr::references(&header.supersedes)
        .into_iter()
        .map(|(_, reference)| show(adr::reference_key(reference)))
        .collect()
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let headers = adr::headers(&inputs.adr);
    let by_key = adr::by_key(&inputs.adr);
    let at = |name: &str, message: String| Finding {
        rule: RULE_ID,
        file: format!("docs/adr/{name}"),
        line: None,
        message,
    };
    let mut found = Vec::new();
    for (name, header) in &headers {
        let key = adr::key_of(name);
        for target in superseded(header) {
            let Some(&target_name) = by_key.get(target.as_str()) else {
                continue;
            };
            let status = headers[target_name].field("Status").unwrap_or(b"");
            if successor(status).as_deref() != Some(key) {
                found.push(at(target_name, format!(
                    "{name} supersedes it, but its status line does not say 'superseded by ADR-{key}'"
                )));
            }
        }
        let status = header.field("Status").unwrap_or(b"");
        match successor(status) {
            None if adr::status_emoji(status).0 == Some("⚪") => {
                found.push(at(
                    name,
                    "status is ⚪ but the line names no superseding ADR".to_owned(),
                ));
            }
            None => {}
            Some(next) => {
                let Some(&next_name) = by_key.get(next.as_str()) else {
                    continue;
                };
                if !superseded(&headers[next_name]).iter().any(|k| k == key) {
                    found.push(at(next_name, format!(
                        "{name} says it supersedes, but it carries no '**Supersedes:**' field naming ADR-{key}"
                    )));
                }
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An identifier `ADR-<key>`, assembled so that this repository's own reference check, which
    /// reads this file, finds no reference it lacks.
    fn id(key: &str) -> String {
        format!("{}-{key}", "ADR")
    }

    fn found(adrs: &[(&str, String)]) -> Vec<(String, String)> {
        let mut inputs = Inputs::default();
        for (name, text) in adrs {
            inputs
                .adr
                .insert((*name).to_owned(), text.as_bytes().to_vec());
        }
        check(&inputs)
            .into_iter()
            .map(|f| (f.file, f.message))
            .collect()
    }

    fn adr(status: &str, supersedes: &str) -> String {
        format!("# T\n\n- **Status:** {status}\n{supersedes}- **Applies to:** x\n")
    }

    #[test]
    fn a_consolidation_both_flipped_and_named_on_a_continuation_line_passes() {
        let by3 = format!("⚪ superseded by [{}](0003-c.md)", id("0003"));
        let field = format!(
            "- **Supersedes:** [{}](0001-a.md),\n  [{}](0002-b.md)\n",
            id("0001"),
            id("0002")
        );
        let adrs = [
            ("0001-a.md", adr(&by3, "")),
            ("0002-b.md", adr(&by3, "")),
            ("0003-c.md", adr("🟢 accepted", &field)),
        ];
        assert_eq!(found(&adrs), []);
    }

    #[test]
    fn a_name_whose_status_was_not_flipped_fails_at_it() {
        let field = format!("- **Supersedes:** {}, {}\n", id("0001"), id("0002"));
        let adrs = [
            (
                "0001-a.md",
                adr(&format!("⚪ superseded by {}", id("0003")), ""),
            ),
            ("0002-b.md", adr("🟡 proposed", "")),
            ("0003-c.md", adr("🟢 accepted", &field)),
        ];
        assert_eq!(
            found(&adrs),
            [(
                "docs/adr/0002-b.md".to_owned(),
                format!(
                    "0003-c.md supersedes it, but its status line does not say 'superseded by {}'",
                    id("0003")
                )
            )]
        );
    }

    #[test]
    fn a_superseded_status_without_a_successor_fails() {
        assert_eq!(
            found(&[("0001-a.md", adr("⚪ superseded", ""))]),
            [(
                "docs/adr/0001-a.md".to_owned(),
                "status is ⚪ but the line names no superseding ADR".to_owned()
            )]
        );
    }

    #[test]
    fn a_successor_that_does_not_name_it_back_fails_at_the_successor() {
        let adrs = [
            (
                "0001-a.md",
                adr(&format!("⚪ superseded by {}", id("draft-next")), ""),
            ),
            ("draft-next.md", adr("🟡 proposed", "")),
        ];
        assert_eq!(
            found(&adrs),
            [(
                "docs/adr/draft-next.md".to_owned(),
                format!(
                    "0001-a.md says it supersedes, but it carries no '**Supersedes:**' field naming {}",
                    id("0001")
                )
            )]
        );
    }

    #[test]
    fn a_name_that_matches_no_file_is_left_to_the_reference_rule() {
        let field = format!("- **Supersedes:** {}\n", id("0009"));
        let adrs = [
            (
                "0001-a.md",
                adr(&format!("⚪ superseded by {}", id("0008")), ""),
            ),
            ("0002-b.md", adr("🟢 accepted", &field)),
        ];
        assert_eq!(found(&adrs), []);
    }
}
