//! Check 14 of the template's `scripts/check-docs.sh`: `README.md` carries a
//! `**Template release:**` line naming a `vX.Y.Z` tag or `unreleased` (ADR-0005 of the template).

use super::{Inputs, Rule};
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/template-release";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "README.md names the release of the template the repository carries on a \
              '**Template release:**' line: a 'vX.Y.Z' tag, or 'unreleased'. Tags do not travel \
              with GitHub's template mechanism, so the line is the one thing that tells a derived \
              project which release it carries; a release moves it, and a project that takes a \
              release up moves it.",
    check,
};

const README: &str = "README.md";
const LABEL: &str = "**Template release:**";

/// The finding for the README, if it lacks the line or the line names no release. Without a
/// README there is nothing to report: the Project Layout rule reports the missing file.
fn check(inputs: &Inputs) -> Vec<Finding> {
    let Some(readme) = &inputs.readme else {
        return Vec::new();
    };
    let found = readme
        .split('\n')
        .enumerate()
        .find(|(_, line)| is_release_line(line));
    let (line, message) = match found {
        None => (
            None,
            "no '**Template release:**' line — the Template section names the release of the \
             template this repository carries, or 'unreleased' (ADR-0005)",
        ),
        Some((_, text)) if names_release(after_label(text)) => return Vec::new(),
        Some((i, _)) => (
            Some(i + 1),
            "the Template release line names neither a 'vX.Y.Z' tag nor 'unreleased' — a \
             release moves it to the tag it cuts (ADR-0005)",
        ),
    };
    vec![Finding {
        rule: RULE_ID,
        file: README.to_owned(),
        line,
        message: message.to_owned(),
    }]
}

/// The space class of POSIX, which the bash check matches with `[[:space:]]`.
fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\x0B' | '\x0C' | '\r')
}

/// Whether the line is the label, after leading space and at most one list marker.
fn is_release_line(line: &str) -> bool {
    let rest = line.trim_start_matches(is_space);
    let rest = rest
        .strip_prefix("- ")
        .or_else(|| rest.strip_prefix("* "))
        .unwrap_or(rest);
    rest.starts_with(LABEL)
}

/// The text after the first label on the line, without its leading space.
fn after_label(line: &str) -> &str {
    line.split_once(LABEL)
        .map_or("", |(_, rest)| rest)
        .trim_start_matches(is_space)
}

/// Whether the value opens with a release, in a link's brackets or backticks or bare, followed
/// by the end, a space, a punctuation mark, or a full stop that ends the sentence.
fn names_release(value: &str) -> bool {
    let value = value.strip_prefix(['[', '`']).unwrap_or(value);
    let rest = value.strip_prefix("unreleased").or_else(|| tag(value));
    let Some(rest) = rest else {
        return false;
    };
    let mut chars = rest.chars();
    match chars.next() {
        None => true,
        Some(c) if is_space(c) || "[],;:)`".contains(c) => true,
        Some('.') => chars.next().is_none_or(is_space),
        Some(_) => false,
    }
}

/// The text after a `vX.Y.Z` tag at the start of the value, each number without a leading zero.
fn tag(value: &str) -> Option<&str> {
    let mut rest = value.strip_prefix('v')?;
    for i in 0..3 {
        if i > 0 {
            rest = rest.strip_prefix('.')?;
        }
        let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        if digits == 0 || (digits > 1 && rest.starts_with('0')) {
            return None;
        }
        rest = &rest[digits..];
    }
    Some(rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn findings(readme: &str) -> Vec<Finding> {
        check(&Inputs {
            readme: Some(readme.to_owned()),
        })
    }

    fn readme(line: &str) -> String {
        format!("# Project\n\n## Template\n\n{line}\n")
    }

    /// The cases of check 14 in the template's `scripts/test-check-docs.sh`, and the forms its
    /// pattern admits beside them.
    #[test]
    fn a_release_or_unreleased_passes() {
        for line in [
            "- **Template release:** unreleased",
            "- **Template release:** v1.4.0 of the template ([tags](https://example.invalid/tags)).",
            "**Template release:** v0.2.0",
            "* **Template release:** [v1.4.0](https://example.invalid/releases/tag/v1.4.0), taken up in full.",
            "- **Template release:** `v1.4.0`",
            "  - **Template release:** v10.0.0.",
            "- **Template release:**v0.5.0;",
            "- **Template release:** v0.5.0\r",
        ] {
            assert_eq!(findings(&readme(line)), [], "{line}");
        }
    }

    #[test]
    fn a_line_without_a_release_fails_at_its_line() {
        for line in [
            "- **Template release:** v01.4.0",
            "- **Template release:** 1.4",
            "- **Template release:** v1.4.0-rc1",
            "- **Template release:** latest",
            "- **Template release:** v1.4.0.1",
            "- **Template release:** v1.4.0.x",
            "- **Template release:**",
        ] {
            let found = findings(&readme(line));
            assert_eq!(found.len(), 1, "{line}");
            assert_eq!(
                (found[0].file.as_str(), found[0].line),
                (README, Some(5)),
                "{line}"
            );
        }
    }

    #[test]
    fn no_line_fails_without_a_line_number() {
        for text in [
            readme("Created from the template."),
            readme("- - **Template release:** v1.0.0"),
        ] {
            let found = findings(&text);
            assert_eq!(found.len(), 1);
            assert_eq!(found[0].line, None);
            assert!(
                found[0]
                    .message
                    .starts_with("no '**Template release:**' line")
            );
        }
    }

    #[test]
    fn only_the_first_release_line_counts() {
        let text = readme("- **Template release:** latest\n- **Template release:** v1.0.0");
        assert_eq!(findings(&text).len(), 1);
    }

    #[test]
    fn without_a_readme_there_is_nothing_to_report() {
        assert_eq!(check(&Inputs::default()), []);
    }
}
