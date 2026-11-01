//! Check 6 of the template's `scripts/check-docs.sh` at v0.5.0: a Markdown link whose text cites
//! an ADR points at that ADR's own file. Only links to ADR files are compared; the template and
//! the superseded and rejected ADRs are not read.

use super::{Input, Inputs, Rule};
use crate::adr::{self, show};
use crate::finding::Finding;
use crate::markdown;

pub const RULE_ID: &str = "docs/adr-link-targets";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "A Markdown link whose text cites an ADR points at that ADR's own file: the \
              identifier and the file it links name the same decision. Both halves resolve on \
              their own, so a renumbering, a copied line, or a consolidated decision set can leave \
              them naming different ADRs. Only links into ADR files are compared.",
    reads: &[Input::Adr, Input::TextFiles],
    check: |inputs| check(inputs).into(),
};

fn check(inputs: &Inputs) -> Vec<Finding> {
    let inactive = adr::inactive_paths(&inputs.adr);
    let mut found = Vec::new();
    for (path, text) in &inputs.text_files {
        if !path.ends_with(".md")
            || path == "docs/adr/template.md"
            || inactive.contains(path.as_str())
        {
            continue;
        }
        for (line, link) in markdown::links(text) {
            let Some(&(_, reference)) = adr::references(markdown::link_text(link)).first() else {
                continue;
            };
            let number = show(adr::reference_key(reference));
            let target = markdown::link_target(link);
            let target = target.split(|&b| b == b'#').next().unwrap_or(target);
            let base = show(target.rsplit(|&b| b == b'/').next().unwrap_or(target));
            if !adr::is_adr_name(&base) {
                continue;
            }
            let agrees = if number.starts_with("draft-") {
                base == format!("{number}.md")
            } else {
                base.starts_with(&format!("{number}-"))
            };
            if !agrees {
                found.push(Finding {
                    rule: RULE_ID,
                    file: path.clone(),
                    line: Some(line),
                    message: format!(
                        "link '{}' cites ADR-{number} but points at '{base}'",
                        show(link)
                    ),
                });
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::id;

    fn messages(files: &[(&str, &str)]) -> Vec<String> {
        let mut inputs = Inputs::default();
        for (path, text) in files {
            inputs
                .text_files
                .insert((*path).to_owned(), text.as_bytes().to_vec());
            if let Some(name) = path.strip_prefix("docs/adr/") {
                inputs.adr.insert(name.to_owned(), text.as_bytes().to_vec());
            }
        }
        check(&inputs).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn a_link_to_the_cited_adr_passes() {
        let text = format!(
            "[{}](docs/adr/0001-a.md#context), [{}](docs/adr/draft-next.md), [{}](docs/adr/README.md)\n",
            id("0001"),
            id("draft-next"),
            id("0002")
        );
        assert_eq!(messages(&[("README.md", &text)]), Vec::<String>::new());
    }

    #[test]
    fn a_link_to_another_adr_fails() {
        let numbered = format!("See [{}](adr/0002-b.md).\n", id("0001"));
        assert_eq!(
            messages(&[("docs/x.md", &numbered)]),
            [format!(
                "link '[{}](adr/0002-b.md)' cites {} but points at '0002-b.md'",
                id("0001"),
                id("0001")
            )]
        );
        let draft = format!("[{}](0001-decision.md)\n", id("draft-next"));
        assert_eq!(
            messages(&[("docs/adr/README.md", &draft)]),
            [format!(
                "link '[{}](0001-decision.md)' cites {} but points at '0001-decision.md'",
                id("draft-next"),
                id("draft-next")
            )]
        );
    }

    #[test]
    fn links_in_fences_other_files_the_template_and_inactive_adrs_are_not_read() {
        let wrong = format!("[{}](0002-b.md)\n", id("0001"));
        let fenced = format!("```\n{wrong}```\n");
        let superseded = format!("- **Status:** ⚪ superseded by {}\n{wrong}", id("0003"));
        assert_eq!(
            messages(&[
                ("README.md", &fenced),
                ("scripts/x.sh", &wrong),
                ("docs/adr/template.md", &wrong),
                ("docs/adr/0001-a.md", &superseded),
            ]),
            Vec::<String>::new()
        );
    }
}
