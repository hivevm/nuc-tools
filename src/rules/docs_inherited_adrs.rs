//! Check 12 of the template's `scripts/check-docs.sh`: once `TEMPLATE-SETUP.md` is gone, every ADR
//! in force whose Deciders field names the NUC maintainer also names a decider of the project and
//! is not proposed. Superseded and rejected ADRs are not read.

use super::{Input, Inputs, Rule};
use crate::adr;
use crate::bash;
use crate::bash::contains;
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/inherited-adrs";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Once TEMPLATE-SETUP.md is gone, every ADR in force whose '**Deciders:**' field \
              names the NUC maintainer also names a decider of this project and is not \
              proposed. An inherited ADR binds a derived project once its own maintainer has \
              added their name and accepted it, or it is superseded: an inherited ADR nobody \
              here decided is a rule nobody decided. While the setup file exists the decision \
              is pending by design.",
    reads: &[Input::Adr, Input::SetupFile],
    check,
};

/// The name the template's ADRs carry as their decider.
const TEMPLATE_DECIDER: &[u8] = b"NUC maintainer";

fn check(inputs: &Inputs) -> Vec<Finding> {
    if inputs.setup_file {
        return Vec::new();
    }
    let mut found = Vec::new();
    for (name, header) in adr::headers(&inputs.adr) {
        if !contains(&header.deciders, TEMPLATE_DECIDER) {
            continue;
        }
        let status = adr::status_emoji(header.field("Status").unwrap_or(b"")).0;
        if matches!(status, Some("⚪" | "🔴")) {
            continue;
        }
        let message = if others(&header.deciders).is_empty() {
            "inherited from the template and not decided by this project — add your name to Deciders and accept it, or supersede it (docs/adr/README.md)"
        } else if status == Some("🟡") {
            "inherited from the template and still proposed after setup — flip the status to accepted, or supersede it (docs/adr/README.md)"
        } else {
            continue;
        };
        found.push(Finding {
            rule: RULE_ID,
            file: format!("docs/adr/{name}"),
            line: None,
            message: message.to_owned(),
        });
    }
    found
}

/// Whether the byte is one of `[[:space:],]`.
fn is_separator(b: u8) -> bool {
    bash::is_space(b) || b == b','
}

/// What is left of the Deciders field once the bash's sed has taken out the label, the template's
/// name, the word `and`, and every space and comma, line by line.
fn others(deciders: &[u8]) -> Vec<u8> {
    let mut left = Vec::new();
    for line in deciders.split(|&b| b == b'\n') {
        let line = line.strip_prefix(b"- **Deciders:**").unwrap_or(line);
        let line = remove_all(line, TEMPLATE_DECIDER);
        let line = remove_and(&line);
        left.extend(line.into_iter().filter(|&b| !is_separator(b)));
    }
    left
}

/// The line with every occurrence of the needle taken out, left to right, as `s/x//g` does.
fn remove_all(line: &[u8], needle: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < line.len() {
        if line[i..].starts_with(needle) {
            i += needle.len();
        } else {
            out.push(line[i]);
            i += 1;
        }
    }
    out
}

/// The line after `s/(^|[[:space:],])and([[:space:],]|$)/ /g`: each match, the longest at the
/// leftmost place, becomes a space, and the search goes on after it, where `^` no longer holds.
fn remove_and(line: &[u8]) -> Vec<u8> {
    let after = |at: usize| -> Option<usize> {
        if !line[at..].starts_with(b"and") {
            return None;
        }
        let end = at + 3;
        match line.get(end) {
            None => Some(end),
            Some(&b) if is_separator(b) => Some(end + 1),
            Some(_) => None,
        }
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < line.len() {
        let separated = is_separator(line[i]).then(|| after(i + 1)).flatten();
        let at_start = (i == 0).then(|| after(0)).flatten();
        match separated.max(at_start) {
            Some(end) => {
                out.push(b' ');
                i = end;
            }
            None => {
                out.push(line[i]);
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(status: &str, deciders: &str, setup: bool) -> Vec<String> {
        let mut inputs = Inputs {
            setup_file: setup,
            ..Inputs::default()
        };
        let text = format!(
            "# T\n\n- **Status:** {status}\n- **Deciders:** {deciders}\n- **Applies to:** x\n"
        );
        inputs
            .adr
            .insert("0001-decision.md".to_owned(), text.into_bytes());
        check(&inputs).into_iter().map(|f| f.message).collect()
    }

    const UNDECIDED: &str = "inherited from the template and not decided by this project — add your name to Deciders and accept it, or supersede it (docs/adr/README.md)";
    const PROPOSED: &str = "inherited from the template and still proposed after setup — flip the status to accepted, or supersede it (docs/adr/README.md)";

    /// The cases of check 12 in the template's `scripts/test-check-docs.sh`.
    #[test]
    fn the_cases_of_the_bash_self_test_report_as_it_does() {
        assert_eq!(found("🟡 proposed", "NUC maintainer", false), [UNDECIDED]);
        assert_eq!(
            found("🟡 proposed", "NUC maintainer", true),
            Vec::<String>::new()
        );
        assert_eq!(
            found("🟢 accepted", "NUC maintainer, project maintainer", false),
            Vec::<String>::new()
        );
        assert_eq!(found("🟢 accepted", "NUC maintainer", false), [UNDECIDED]);
        assert_eq!(
            found(
                "🟢 accepted",
                "NUC maintainer and project maintainer",
                false
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            found("🔴 rejected", "NUC maintainer", false),
            Vec::<String>::new()
        );
        assert_eq!(
            found("🟡 proposed", "project maintainer", false),
            Vec::<String>::new()
        );
        assert_eq!(
            found(
                "🟡 proposed",
                "project maintainer (reviewing),\n  NUC maintainer",
                false
            ),
            [PROPOSED]
        );
    }

    #[test]
    fn the_word_and_goes_as_the_bash_s_sed_takes_it() {
        assert_eq!(others(b"- **Deciders:** NUC maintainer and\n"), b"");
        assert_eq!(others(b"- **Deciders:** NUC maintainer,and,and\n"), b"and");
        assert_eq!(others(b"and NUC maintainer\n"), b"");
        assert_eq!(
            others(b"- **Deciders:** NUC maintainer, Andrea\n"),
            b"Andrea"
        );
        assert_eq!(others(b"- **Deciders:** NUC maintainer, sandy\n"), b"sandy");
        assert_eq!(others(b"x and and y\n"), b"xandy");
        assert_eq!(others(b"and,and\n"), b"and");
        assert_eq!(others(b",and\n"), b"");
    }
}
