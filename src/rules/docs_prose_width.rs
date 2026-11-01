//! Check 15 of the template's `scripts/check-docs.sh`: a line of Markdown prose is at most 100
//! columns. Fenced code, front matter, table rows, headings, ADR header fields, and a line that
//! cannot break are exempt; superseded and rejected ADRs are not read, nor is a file marked
//! `linguist-generated`. Indented code is read as prose.

use super::{Input, Inputs, Rule};
use crate::adr;
use crate::bash::{awk_lines, is_space};
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/prose-width";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "A line of Markdown prose is at most 100 columns, so a diff shows the sentence that \
              changed. Fenced code, front matter, table rows, headings, and ADR header fields \
              are exempt, and so is a line that cannot break, such as a link standing on a line \
              of its own. Superseded and rejected ADRs are not read, nor is a file \
              .gitattributes marks 'linguist-generated', such as a changelog a tool writes. \
              Indented code blocks are read as prose.",
    reads: &[Input::Adr, Input::TextFiles, Input::Generated],
    check,
};

/// The widest a line of prose may be, in columns.
const WIDTH: usize = 100;

/// The lines longer than 100 bytes outside the front matter and a fence, with their numbers, as
/// the bash's awk pass in the C locale finds them.
pub fn wide_lines(text: &[u8]) -> Vec<(usize, &[u8])> {
    let mut wide = Vec::new();
    let mut front = false;
    let mut fence: Option<&[u8]> = None;
    for (i, line) in awk_lines(text).into_iter().enumerate() {
        if i == 0 && line == b"---" {
            front = true;
            continue;
        }
        if front {
            front = line != b"---";
            continue;
        }
        if let Some(open) = fence {
            if closes(line, open) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = opens(line) {
            fence = Some(open);
            continue;
        }
        if line.len() > WIDTH {
            wide.push((i + 1, line));
        }
    }
    wide
}

/// The run of a fence the line opens, `^[[:space:]]*(```+|~~~+)`, without its indent.
fn opens(line: &[u8]) -> Option<&[u8]> {
    let indent = line.iter().take_while(|&&b| is_space(b)).count();
    let rest = &line[indent..];
    let mark = *rest.first().filter(|&&b| b == b'`' || b == b'~')?;
    let run = rest.iter().take_while(|&&b| b == mark).count();
    (run >= 3).then_some(&rest[..run])
}

/// Whether the line, `^[[:space:]]*[`~]+[[:space:]]*$`, closes the fence opened by `open`: its
/// marks without the spaces start with the opening run.
fn closes(line: &[u8], open: &[u8]) -> bool {
    let indent = line.iter().take_while(|&&b| is_space(b)).count();
    let run = line[indent..]
        .iter()
        .take_while(|&&b| b == b'`' || b == b'~')
        .count();
    let shaped = line[indent + run..].iter().all(|&b| is_space(b)) && run > 0;
    shaped && run >= 3 && line[indent..indent + run].starts_with(open)
}

/// The characters of the line as bash counts them in `C.UTF-8`, for a line [`undecided`] passes.
pub fn columns(line: &[u8]) -> usize {
    std::str::from_utf8(line).map_or(line.len(), |text| text.chars().count())
}

/// Whether a wide line holds what the bash reads otherwise than its characters: a byte of no
/// valid UTF-8, which glibc and Rust count apart and `read` may join to the next line, or the
/// unit separator 0x1F, which the bash splits its record pass on. Such a line is not decided.
pub fn undecided(line: &[u8]) -> bool {
    std::str::from_utf8(line).is_err() || line.contains(&0x1F)
}

/// The wide lines a file the rule reads holds that [`undecided`] refuses, by number.
pub fn undecided_lines(text: &[u8]) -> Vec<usize> {
    wide_lines(text)
        .into_iter()
        .filter(|(_, line)| undecided(line))
        .map(|(n, _)| n)
        .collect()
}

/// Whether a line of prose can break: a space outside the text of its links, after its indent.
fn can_break(line: &[u8]) -> bool {
    let indent = line.iter().take_while(|&&b| b == b' ').count();
    let mut body = line[indent..].to_vec();
    while let Some(at) = link_space(&body) {
        body[at] = b'_';
    }
    body.contains(&b' ')
}

/// The space `(\[[^] ]*) ([^]]*\]\()` finds first: the first space in the text of a link.
fn link_space(body: &[u8]) -> Option<usize> {
    body.iter().enumerate().find_map(|(open, &b)| {
        if b != b'[' {
            return None;
        }
        let text = &body[open + 1..];
        let before = text.iter().take_while(|&&c| c != b']' && c != b' ').count();
        if text.get(before) != Some(&b' ') {
            return None;
        }
        let after = &text[before + 1..];
        let close = after.iter().position(|&c| c == b']')?;
        (after.get(close + 1) == Some(&b'(')).then_some(open + 1 + before)
    })
}

/// Whether the line is a table row, `^[[:space:]]*\|`, or a heading, `^#`.
fn by_syntax(line: &[u8]) -> bool {
    let indent = line.iter().take_while(|&&b| is_space(b)).count();
    line.get(indent) == Some(&b'|') || line.starts_with(b"#")
}

/// Whether the line is an ADR header field, `^- \*\*[A-Za-z ]+:\*\*`.
fn header_field(line: &[u8]) -> bool {
    let Some(rest) = line.strip_prefix(b"- **") else {
        return false;
    };
    let name = rest
        .iter()
        .take_while(|&&b| b.is_ascii_alphabetic() || b == b' ')
        .count();
    name > 0 && rest[name..].starts_with(b":**")
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let inactive = adr::inactive_paths(&inputs.adr);
    let mut found = Vec::new();
    for (file, text) in &inputs.text_files {
        if !file.ends_with(".md") || inactive.contains(file) || inputs.generated.contains(file) {
            continue;
        }
        let in_adr = file.starts_with("docs/adr/");
        for (n, line) in wide_lines(text) {
            let width = columns(line);
            if width <= WIDTH || by_syntax(line) || (in_adr && header_field(line)) {
                continue;
            }
            if can_break(line) {
                found.push(Finding {
                    rule: RULE_ID,
                    file: file.clone(),
                    line: Some(n),
                    message: format!(
                        "{width} columns — wrap the prose at 100 (a link that does not fit stands on a line of its own)"
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

    fn found(path: &str, text: &str) -> Vec<String> {
        let mut inputs = Inputs::default();
        inputs
            .text_files
            .insert(path.to_owned(), text.as_bytes().to_vec());
        check(&inputs)
            .into_iter()
            .map(|f| format!("{}:{}: {}", f.file, f.line.unwrap_or(0), f.message))
            .collect()
    }

    fn prose(n: usize) -> String {
        "word ".repeat(n / 5) + &"x".repeat(n % 5)
    }

    #[test]
    fn a_line_over_100_columns_fails_and_one_at_100_passes() {
        assert_eq!(
            found("README.md", &format!("{}\n", prose(100))),
            Vec::<String>::new()
        );
        assert_eq!(
            found("README.md", &format!("# T\n\n{}\n", prose(101))),
            [
                "README.md:3: 101 columns — wrap the prose at 100 (a link that does not fit stands on a line of its own)"
            ]
        );
    }

    #[test]
    fn fences_front_matter_tables_headings_and_header_fields_are_exempt() {
        let long = prose(120);
        let text = format!(
            "---\n{long}\n---\n```\n{long}\n```\n  ~~~~\n{long}\n~~~\n{long}\n~~~~\n| {long}\n# {long}\n"
        );
        assert_eq!(found("README.md", &text), Vec::<String>::new());
        let header = format!("- **Applies to:** {long}\n");
        assert_eq!(found("docs/adr/0001-a.md", &header), Vec::<String>::new());
        assert_eq!(found("docs/x.md", &header).len(), 1);
    }

    #[test]
    fn a_line_that_cannot_break_is_exempt_and_link_text_is_one_word() {
        let url = format!("https://example.invalid/{}", "a".repeat(100));
        assert_eq!(
            found("README.md", &format!("{url}\n")),
            Vec::<String>::new()
        );
        assert_eq!(
            found("README.md", &format!("  [a long link text]({url})\n")),
            Vec::<String>::new()
        );
        assert_eq!(found("README.md", &format!("See [x]({url})\n")).len(), 1);
    }

    #[test]
    fn columns_count_characters_as_bash_does_in_c_utf8() {
        assert_eq!(columns("—".repeat(101).as_bytes()), 101);
        let text = format!("{}\n", "é ".repeat(51));
        assert_eq!(text.len(), 154);
        assert_eq!(
            found("README.md", &text),
            [
                "README.md:1: 102 columns — wrap the prose at 100 (a link that does not fit stands on a line of its own)"
            ]
        );
        assert_eq!(
            found("README.md", &format!("{}\n", "é ".repeat(40))),
            Vec::<String>::new()
        );
        assert!(undecided(b"a\xff b"));
        assert!(undecided(b"\xe2\x80"));
        assert!(undecided(b"\xf4\x90\x80\x80"));
        assert!(undecided(b"a\x1f"));
        assert!(!undecided("é —".as_bytes()));
    }

    #[test]
    fn inactive_adrs_and_generated_files_are_not_read() {
        let long = prose(120);
        let mut inputs = Inputs::default();
        let superseded = format!("- **Status:** ⚪ superseded by ADR-0002\n\n{long}\n");
        for (path, text) in [
            ("docs/adr/0001-a.md", superseded.as_str()),
            ("CHANGELOG.md", long.as_str()),
        ] {
            inputs
                .text_files
                .insert(path.to_owned(), text.as_bytes().to_vec());
        }
        inputs
            .adr
            .insert("0001-a.md".to_owned(), superseded.as_bytes().to_vec());
        inputs.generated.insert("CHANGELOG.md".to_owned());
        assert_eq!(check(&inputs), []);
    }

    /// The cases of check 15 in the template's `scripts/test-check-docs.sh` the others leave out.
    #[test]
    fn the_remaining_cases_of_the_bash_self_test() {
        let words = |n: usize| "word ".repeat(n).chars().take(n).collect::<String>();
        assert_eq!(
            found("README.md", &format!("{} — {}\n", words(48), words(49))),
            Vec::<String>::new()
        );
        assert_eq!(
            found(
                "README.md",
                &format!("````\n```\n{}\n```\n````\n", words(120))
            ),
            Vec::<String>::new()
        );
        let url = words(100).replace(' ', "x");
        assert_eq!(
            found(
                "README.md",
                &format!("[a link](https://example.invalid/{url}) and prose that could wrap\n")
            )
            .len(),
            1
        );
        assert_eq!(
            found("README.md", &format!("- **Label:** {}\n", words(110))).len(),
            1
        );
    }
}
