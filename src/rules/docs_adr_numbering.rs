//! Check 2 of the template's `scripts/check-docs.sh` at v0.5.0: the ADR files run 0001..N without
//! gaps, each `# ADR-NNNN` heading matches its file name, and a draft's slug and heading follow
//! its file name.

use super::{Input, Inputs, Rule};
use crate::adr;
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/adr-numbering";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "The ADR files run 0001..N without gaps, and each file's '# ADR-NNNN' heading \
              matches its file name. A draft stands outside the count: its slug is lower case \
              letters and digits joined by single hyphens, and its heading reads \
              '# ADR-draft-<slug>' for the file 'draft-<slug>.md'. Superseded ADRs stay on disk, \
              so a gap can only mean a deleted ADR or a botched rename.",
    reads: &[Input::Adr],
    check: |inputs| check(inputs).into(),
};

fn finding(file: String, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        file,
        line: None,
        message,
    }
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let headers = adr::headers(&inputs.adr);
    let mut found = Vec::new();
    let numbered = headers
        .iter()
        .filter(|(name, _)| !name.starts_with("draft-"));
    for (n, (name, header)) in numbered.enumerate() {
        let number = adr::key_of(name);
        let expected = format!("{:04}", n + 1);
        if number != expected {
            found.push(finding("docs/adr".to_owned(), format!(
                "ADR numbering: expected '{expected}-*.md' at position {}, found '{name}' — numbers must run 0001..N without gaps",
                n + 1
            )));
            break;
        }
        let text = header.heading.as_deref().map(adr::show).unwrap_or_default();
        let heading = number_heading(&text).unwrap_or("");
        if heading != format!("# ADR-{number}") {
            found.push(finding(
                format!("docs/adr/{name}"),
                format!(
                    "heading says '{}', but the filename says ADR-{number}",
                    if heading.is_empty() {
                        "<none>"
                    } else {
                        heading
                    }
                ),
            ));
        }
    }
    for (name, header) in headers
        .iter()
        .filter(|(name, _)| name.starts_with("draft-"))
    {
        let key = adr::key_of(name);
        if !key.strip_prefix("draft-").is_some_and(is_slug) {
            found.push(finding(
                format!("docs/adr/{name}"),
                "a draft's slug is lower case letters and digits joined by single hyphens"
                    .to_owned(),
            ));
            continue;
        }
        let text = header.heading.as_deref().map(adr::show).unwrap_or_default();
        let heading = reference_heading(&text).unwrap_or("");
        if heading != format!("# ADR-{key}") {
            found.push(finding(
                format!("docs/adr/{name}"),
                format!(
                    "heading says '{}', but the filename says ADR-{key}",
                    if heading.is_empty() {
                        "<none>"
                    } else {
                        heading
                    }
                ),
            ));
        }
    }
    found
}

/// The `# ADR-NNNN` the heading starts with, four digits.
fn number_heading(heading: &str) -> Option<&str> {
    let digits = heading.strip_prefix("# ADR-")?;
    (digits.len() >= 4 && digits.as_bytes()[..4].iter().all(u8::is_ascii_digit))
        .then(|| &heading[.."# ADR-".len() + 4])
}

/// The `# ADR-<key>` the heading starts with, the key a number or the longest draft slug.
fn reference_heading(heading: &str) -> Option<&str> {
    let rest = heading.strip_prefix("# ADR-")?;
    let prefix = "# ADR-".len();
    if rest.len() >= 4 && rest.as_bytes()[..4].iter().all(u8::is_ascii_digit) {
        return Some(&heading[..prefix + 4]);
    }
    let slug = rest.strip_prefix("draft-")?;
    let length = adr::slug_length(slug.as_bytes());
    (length > 0).then(|| &heading[..prefix + "draft-".len() + length])
}

/// Whether the text is a slug: lower case letters and digits joined by single hyphens.
fn is_slug(text: &str) -> bool {
    !text.is_empty() && adr::slug_length(text.as_bytes()) == text.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adr::Dir;
    use crate::testing::draft_id;

    fn inputs(files: &[(&str, &str)]) -> Inputs {
        let adr: Dir = files
            .iter()
            .map(|(n, t)| ((*n).to_owned(), t.as_bytes().to_vec()))
            .collect();
        Inputs {
            adr,
            ..Inputs::default()
        }
    }

    fn messages(files: &[(&str, &str)]) -> Vec<String> {
        check(&inputs(files))
            .into_iter()
            .map(|f| f.message)
            .collect()
    }

    #[test]
    fn numbers_without_gaps_and_matching_headings_pass() {
        assert_eq!(
            messages(&[
                ("0001-a.md", "# ADR-0001: A"),
                ("0002-b.md", "# ADR-0002: B"),
                ("draft-c-d.md", &format!("# {}", draft_id("c-d: C")))
            ]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_gap_fails_once_at_its_position() {
        let found = messages(&[
            ("0001-a.md", "# ADR-0001"),
            ("0003-c.md", "# ADR-0003"),
            ("0004-d.md", "# ADR-0004"),
        ]);
        assert_eq!(
            found,
            [
                "ADR numbering: expected '0002-*.md' at position 2, found '0003-c.md' — numbers must run 0001..N without gaps"
            ]
        );
    }

    #[test]
    fn a_heading_that_names_another_number_or_none_fails() {
        assert_eq!(
            messages(&[("0001-a.md", "# ADR-0002: A")]),
            ["heading says '# ADR-0002', but the filename says ADR-0001"]
        );
        assert_eq!(
            messages(&[("0001-a.md", "# A")]),
            ["heading says '<none>', but the filename says ADR-0001"]
        );
    }

    #[test]
    fn a_draft_with_a_malformed_slug_or_another_heading_fails() {
        assert_eq!(
            messages(&[("draft-Bad_slug.md", &format!("# {}", draft_id("Bad_slug")))]),
            ["a draft's slug is lower case letters and digits joined by single hyphens"]
        );
        assert_eq!(
            messages(&[("draft-a--b.md", &format!("# {}", draft_id("a--b")))]).len(),
            1
        );
        assert_eq!(
            messages(&[("draft-x.md", &format!("# {}", draft_id("y: Y")))]),
            [format!(
                "heading says '# {}', but the filename says {}",
                draft_id("y"),
                draft_id("x")
            )]
        );
        assert_eq!(
            messages(&[("draft-x-y.md", &format!("# {}", draft_id("x-y-: T")))]),
            Vec::<String>::new()
        );
    }
}
