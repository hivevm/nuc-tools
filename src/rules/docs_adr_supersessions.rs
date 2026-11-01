//! Check 8 of the template's `scripts/check-docs.sh`: the `**Supersedes:**` field of an ADR and
//! the `⚪ superseded by ADR-NNNN` status line of every ADR it names agree, both ways. A name that
//! matches no file is the reference rule's. From v0.6.0 the old line flips when a human accepts
//! the new ADR: a proposed or rejected one names old ADRs that keep their status, and an old ADR
//! names no successor that is not accepted. One superseded in its turn counts as accepted, so a
//! chain stays whole.

use super::{Input, Inputs, Rule};
use crate::adr::{self, Header, show};
use crate::bash;
use crate::config::Version;
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/adr-supersessions";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "The '**Supersedes:**' field of a new ADR and the '⚪ superseded by ADR-NNNN' \
              status line of every old one it names, one or several it consolidates, name each \
              other. The two halves stand in two files, and the index check sees neither one \
              missing. Every name of a Supersedes field counts, on whichever line it sits. From \
              template release v0.6.0 the old line flips only when a human accepts the new ADR, \
              so a proposed one leaves the old ones in force.",
    reads: &[Input::Adr, Input::Release],
    check: |inputs| check(inputs).into(),
};

/// The release from which a human flips the old status lines when accepting the new ADR.
const FLIP_ON_ACCEPTANCE: Version = Version(0, 6, 0);

/// Whether the status counts as accepted for a supersession: accepted, or superseded in turn.
fn accepted(status: &[u8]) -> bool {
    matches!(adr::status_emoji(status).0, Some("🟢" | "⚪"))
}

/// The key a `superseded by [ADR-<key>` status line names, the bracket optional.
fn successor(status: &[u8]) -> Option<String> {
    let label: &[u8] = b"superseded by ";
    let mut i = 0;
    while let Some(at) = bash::find(&status[i..], label) {
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
    let on_acceptance = inputs.carries_at_least(FLIP_ON_ACCEPTANCE);
    let mut found = Vec::new();
    for (name, header) in &headers {
        let key = adr::key_of(name);
        let flips = !on_acceptance || accepted(header.field("Status").unwrap_or(b""));
        for target in superseded(header).into_iter().filter(|_| flips) {
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
                if on_acceptance && !accepted(headers[next_name].field("Status").unwrap_or(b"")) {
                    found.push(at(name, format!(
                        "superseded by ADR-{next}, which is not accepted — the status line flips when the human accepts ADR-{next}"
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

    /// The findings on a tree of template release v0.5.0, whose cases these tests copy.
    fn found(adrs: &[(&str, String)]) -> Vec<(String, String)> {
        found_at(Some(Version(0, 5, 0)), adrs)
    }

    fn found_at(release: Option<Version>, adrs: &[(&str, String)]) -> Vec<(String, String)> {
        let mut inputs = Inputs {
            release,
            ..Inputs::default()
        };
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
    #[test]
    fn a_consolidation_on_one_line_passes() {
        let by3 = format!("⚪ superseded by {}", id("0003"));
        let field = format!(
            "- **Supersedes:** [{}](0001-a.md), [{}](0002-b.md)\n",
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
    fn an_unflipped_name_on_the_continuation_line_fails_at_it() {
        let field = format!(
            "- **Supersedes:** [{}](0001-a.md),\n  [{}](0002-b.md)\n",
            id("0001"),
            id("0002")
        );
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
    fn a_draft_successor_linked_from_the_status_line_passes() {
        let status = format!("⚪ superseded by [{}](draft-next.md)", id("draft-next"));
        let field = format!("- **Supersedes:** [{}](0001-a.md)\n", id("0001"));
        let adrs = [
            ("0001-a.md", adr(&status, "")),
            ("draft-next.md", adr("🟡 proposed", &field)),
        ];
        assert_eq!(found(&adrs), []);
    }

    /// ADR 2, proposed, superseding ADR 1, whose status line reads as given.
    fn proposed_supersession(old: &str) -> [(&'static str, String); 2] {
        let field = format!("- **Supersedes:** [{}](0001-a.md)\n", id("0001"));
        [
            ("0001-a.md", adr(old, "")),
            ("0002-b.md", adr("🟡 proposed", &field)),
        ]
    }

    /// Verifies: ADR-0021
    #[test]
    fn from_v0_6_0_a_proposed_supersession_leaves_the_old_adr_in_force() {
        let v0_6 = Some(FLIP_ON_ACCEPTANCE);
        assert_eq!(found_at(v0_6, &proposed_supersession("🟢 accepted")), []);
        let early = proposed_supersession(&format!("⚪ superseded by {}", id("0002")));
        assert_eq!(
            found_at(v0_6, &early),
            [(
                "docs/adr/0001-a.md".to_owned(),
                format!(
                    "superseded by {}, which is not accepted — the status line flips when the human accepts {}",
                    id("0002"),
                    id("0002")
                )
            )]
        );
        // The same trees on v0.5.x and on 'unreleased', the newest release.
        assert_eq!(found(&early), []);
        assert_eq!(found(&proposed_supersession("🟢 accepted")).len(), 1);
        assert_eq!(found_at(None, &early).len(), 1);
    }

    /// Verifies: ADR-0021
    #[test]
    fn from_v0_6_0_a_chain_of_supersessions_passes() {
        let adrs = [
            (
                "0001-a.md",
                adr(&format!("⚪ superseded by {}", id("0002")), ""),
            ),
            (
                "0002-b.md",
                adr(
                    &format!("⚪ superseded by {}", id("0003")),
                    &format!("- **Supersedes:** [{}](0001-a.md)\n", id("0001")),
                ),
            ),
            (
                "0003-c.md",
                adr(
                    "🟢 accepted",
                    &format!("- **Supersedes:** [{}](0002-b.md)\n", id("0002")),
                ),
            ),
        ];
        assert_eq!(found_at(Some(FLIP_ON_ACCEPTANCE), &adrs), []);
    }

    /// Verifies: ADR-0021
    #[test]
    fn from_v0_6_0_an_old_adr_flipped_for_a_draft_still_proposed_fails() {
        let status = format!("⚪ superseded by [{}](draft-next.md)", id("draft-next"));
        let field = format!("- **Supersedes:** [{}](0001-a.md)\n", id("0001"));
        let adrs = [
            ("0001-a.md", adr(&status, "")),
            ("draft-next.md", adr("🟡 proposed", &field)),
        ];
        assert_eq!(found_at(Some(Version(0, 7, 2)), &adrs).len(), 1);
    }

    /// Verifies: ADR-0021
    #[test]
    fn from_v0_6_0_a_rejected_successor_leaves_the_old_adr_in_force() {
        let field = format!("- **Supersedes:** [{}](0001-a.md)\n", id("0001"));
        let kept = [
            ("0001-a.md", adr("🟢 accepted", "")),
            ("0002-b.md", adr("🔴 rejected", &field)),
        ];
        assert_eq!(found_at(Some(FLIP_ON_ACCEPTANCE), &kept), []);
        let flipped = [
            (
                "0001-a.md",
                adr(&format!("⚪ superseded by {}", id("0002")), ""),
            ),
            ("0002-b.md", adr("🔴 rejected", &field)),
        ];
        assert_eq!(found_at(Some(FLIP_ON_ACCEPTANCE), &flipped).len(), 1);
    }
}
