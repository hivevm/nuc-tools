//! The decision record as the template's `scripts/check-docs.sh` and `scripts/lib/adr.sh` read
//! it: which files are ADRs, the header fields of each, its `# ADR-` heading, a header field in
//! another form than the list item, and the template's instruction paragraph left in it. The rules
//! of the record share this reading, so that each reads an ADR the one way the bash does.

use std::collections::BTreeMap;

/// The status emoji of the record's legend.
pub const STATUS_EMOJI: [char; 4] = ['🟢', '🟡', '🔴', '⚪'];

/// The header fields of `docs/adr/template.md`.
const FIELDS: [&str; 5] = ["Status", "Date", "Deciders", "Applies to", "Supersedes"];

/// A file directly in `docs/adr/`, as the shell read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    /// The text, a byte that is no UTF-8 standing for itself.
    pub text: String,
    /// Whether it is a regular file of its own, not a link: what `find -type f` lists.
    pub listed: bool,
}

/// The files directly in `docs/adr/`, by name; empty when the directory does not exist.
pub type Dir = BTreeMap<String, File>;

/// Whether the name is that of an ADR file: `NNNN-*.md` or `draft-*.md`.
pub fn is_adr_name(name: &str) -> bool {
    let numbered = name.len() > 8
        && name.as_bytes()[..4].iter().all(u8::is_ascii_digit)
        && name.as_bytes()[4] == b'-';
    (numbered || name.starts_with("draft-")) && name.ends_with(".md")
}

/// The ADR files of the directory, the numbered ones in order, then the drafts.
pub fn adr_names(dir: &Dir) -> Vec<&str> {
    dir.iter()
        .filter(|(name, file)| file.listed && is_adr_name(name))
        .map(|(name, _)| name.as_str())
        .collect()
}

/// What follows `ADR-` in the identifier of the ADR file: its number, or `draft-<slug>`.
pub fn key_of(name: &str) -> &str {
    if name.starts_with("draft-") {
        name.strip_suffix(".md").unwrap_or(name)
    } else {
        name.split('-').next().unwrap_or(name)
    }
}

/// The header of one ADR, read as the bash's awk pass reads it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Header {
    /// The first line of each field written as the list item `- **<Name>:** …`, anywhere in the
    /// file, by name; the Supersedes and Deciders fields with their continuation lines.
    pub fields: BTreeMap<String, Vec<String>>,
    /// The first line that starts with `# ADR-`.
    pub heading: Option<String>,
    /// Each header field in another form, before the first `## ` heading, with its line number.
    pub stray: Vec<(usize, String)>,
    /// Whether the file still carries the template's instruction paragraph.
    pub needle: bool,
}

impl Header {
    /// The first line of the field, as `grep -m1 '^- **<name>:**'` finds it.
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .get(name)
            .and_then(|lines| lines.first())
            .map(String::as_str)
    }
}

/// The header of the text, with the instruction needle of the template, if it has one.
pub fn header(text: &str, needle: Option<&str>) -> Header {
    let mut header = Header::default();
    let mut head = true;
    let mut cont: Option<String> = None;
    for (i, line) in text.split('\n').enumerate() {
        if head && line.starts_with("## ") {
            head = false;
        }
        if head && names_a_field(line) && !line.starts_with("- **") {
            header.stray.push((i + 1, line.to_owned()));
        }
        if needle.is_some_and(|needle| !needle.is_empty() && line.contains(needle)) {
            header.needle = true;
        }
        if header.heading.is_none() && line.starts_with("# ADR-") {
            header.heading = Some(line.to_owned());
        }
        if let Some(name) = cont.as_ref().filter(|_| continues(line)) {
            header
                .fields
                .entry(name.clone())
                .or_default()
                .push(line.to_owned());
            continue;
        }
        cont = None;
        if let Some(name) = list_item_field(line).filter(|name| !header.fields.contains_key(*name))
        {
            header.fields.insert(name.to_owned(), vec![line.to_owned()]);
            if name == "Supersedes" || name == "Deciders" {
                cont = Some(name.to_owned());
            }
        }
    }
    header
}

/// Whether the line holds `**<field>:**` for a field of the template, anywhere on it.
fn names_a_field(line: &str) -> bool {
    FIELDS
        .iter()
        .any(|field| line.contains(&format!("**{field}:**")))
}

/// The name of a list item field `- **<Name>:**` at the start of the line, letters and spaces.
fn list_item_field(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("- **")?;
    let end = rest.find(":**")?;
    let name = &rest[..end];
    (!name.is_empty() && name.chars().all(|c| c.is_ascii_alphabetic() || c == ' ')).then_some(name)
}

/// Whether the line continues a field: whitespace, then something else.
fn continues(line: &str) -> bool {
    let rest = line.trim_start_matches(is_space);
    rest.len() < line.len() && !rest.is_empty()
}

/// The needle of the template's instruction paragraph: the 40 characters after the `*` of its
/// first line that starts with `*` and a character other than `*`.
pub fn instruction_needle(template: &str) -> Option<String> {
    let line = template.split('\n').find(|line| {
        let mut chars = line.chars();
        chars.next() == Some('*') && chars.next().is_some_and(|c| c != '*')
    })?;
    Some(line.chars().skip(1).take(40).collect())
}

/// The first status emoji of the text, by position, and how many the text carries in all.
pub fn status_emoji(text: &str) -> (Option<char>, usize) {
    let first = text.chars().find(|c| STATUS_EMOJI.contains(c));
    let count = text.chars().filter(|c| STATUS_EMOJI.contains(c)).count();
    (first, count)
}

/// The space class of POSIX, which the bash matches with `[[:space:]]`.
pub fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\x0B' | '\x0C' | '\r')
}

/// The text without leading and trailing POSIX space.
pub fn trim(text: &str) -> &str {
    text.trim_matches(is_space)
}

/// The headers of every ADR of the directory, by name, read with the template's needle.
pub fn headers(dir: &Dir) -> BTreeMap<&str, Header> {
    let needle = dir
        .get("template.md")
        .and_then(|t| instruction_needle(&t.text));
    adr_names(dir)
        .into_iter()
        .map(|name| (name, header(&dir[name].text, needle.as_deref())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adr_names_are_the_numbered_files_then_the_drafts() {
        let file = |listed| File {
            text: String::new(),
            listed,
        };
        let dir: Dir = [
            ("0002-b.md", file(true)),
            ("0001-a.md", file(true)),
            ("draft-x.md", file(true)),
            ("draft-link.md", file(false)),
            ("README.md", file(true)),
            ("template.md", file(true)),
            ("0003.md", file(true)),
            ("12345-x.md", file(true)),
        ]
        .into_iter()
        .map(|(n, f)| (n.to_owned(), f))
        .collect();
        assert_eq!(adr_names(&dir), ["0001-a.md", "0002-b.md", "draft-x.md"]);
        assert_eq!(
            (key_of("0001-a.md"), key_of("draft-x.md")),
            ("0001", "draft-x")
        );
    }

    #[test]
    fn a_header_is_read_as_the_awk_pass_reads_it() {
        let text = "# ADR-0001: Title\n\n- **Status:** 🟢 accepted\n- **Deciders:** A,\n  B\n**Date:** 2026\n\n## Context\n\n- **Status:** 🟡 later\n**Applies to:** body\n";
        let header = header(text, Some("Complete the header"));
        assert_eq!(header.heading.as_deref(), Some("# ADR-0001: Title"));
        assert_eq!(header.field("Status"), Some("- **Status:** 🟢 accepted"));
        assert_eq!(header.fields["Deciders"], ["- **Deciders:** A,", "  B"]);
        assert_eq!(header.stray, [(6, "**Date:** 2026".to_owned())]);
        assert!(!header.needle);
    }

    #[test]
    fn the_needle_is_forty_characters_after_the_star() {
        let template =
            "# T\n\n**Bold** line\n*Complete the header and delete what does not apply. More*\n";
        assert_eq!(
            instruction_needle(template).as_deref(),
            Some("Complete the header and delete what does")
        );
        assert!(
            header(
                "x\n*Complete the header and delete what does it*\n",
                Some("Complete the header")
            )
            .needle
        );
    }

    #[test]
    fn the_first_emoji_by_position_and_the_count_of_all() {
        assert_eq!(
            status_emoji("- **Status:** 🟡 proposed | 🟢 accepted"),
            (Some('🟡'), 2)
        );
        assert_eq!(status_emoji("no emoji"), (None, 0));
    }
}
