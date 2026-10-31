//! The decision record as the template's `scripts/check-docs.sh` and `scripts/lib/adr.sh` read
//! it: which files are ADRs, the header fields of each, its `# ADR-` heading, a header field in
//! another form than the list item, and the template's instruction paragraph left in it. The rules
//! of the record share this reading, so that each reads an ADR the one way the bash does: byte by
//! byte, as awk and bash do under `LC_ALL=C`.

use std::collections::{BTreeMap, BTreeSet};

/// The status emoji of the record's legend, as UTF-8 bytes.
pub const STATUS_EMOJI: [&str; 4] = ["🟢", "🟡", "🔴", "⚪"];

/// The header fields of `docs/adr/template.md`.
const FIELDS: [&str; 5] = ["Status", "Date", "Deciders", "Applies to", "Supersedes"];

/// The files directly in `docs/adr/` that the bash reads: the ADRs, the index `README.md`, and
/// `template.md`, by name, with their bytes; empty when the directory does not exist.
pub type Dir = BTreeMap<String, Vec<u8>>;

/// Whether the name is that of an ADR file, as `find -name` matches `[0-9][0-9][0-9][0-9]-*.md`
/// or `draft-*.md`, the `*` empty as well.
pub fn is_adr_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    let numbered =
        bytes.len() >= 8 && bytes[..4].iter().all(u8::is_ascii_digit) && bytes[4] == b'-';
    (numbered || name.starts_with("draft-")) && name.ends_with(".md")
}

/// The ADR files of the directory, the numbered ones in order, then the drafts.
pub fn adr_names(dir: &Dir) -> Vec<&str> {
    dir.keys()
        .map(String::as_str)
        .filter(|name| is_adr_name(name))
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
    /// file, by name.
    pub fields: BTreeMap<String, Vec<u8>>,
    /// The first line that starts with `# ADR-`, or the field `- **heading:**`, which the awk
    /// files under the same key.
    pub heading: Option<Vec<u8>>,
    /// Each header field in another form, before the first `## ` heading, as the awk prints it:
    /// `<line number>:<line>`, or the line of a field `- **stray:**`.
    pub stray: Vec<Vec<u8>>,
    /// Whether the file still carries the template's instruction paragraph, or a field
    /// `- **needle:**`.
    pub needle: bool,
}

impl Header {
    /// The first line of the field, as `grep -m1 '^- **<name>:**'` finds it.
    pub fn field(&self, name: &str) -> Option<&[u8]> {
        self.fields.get(name).map(Vec::as_slice)
    }
}

/// The lines of the text as awk reads them: split at newlines, an unterminated last line kept.
pub fn awk_lines(text: &[u8]) -> Vec<&[u8]> {
    let mut lines: Vec<&[u8]> = text.split(|&b| b == b'\n').collect();
    if text.ends_with(b"\n") || text.is_empty() {
        lines.pop();
    }
    lines
}

/// The lines of the text as `while IFS= read -r` reads them: an unterminated last line dropped.
pub fn read_lines(text: &[u8]) -> Vec<&[u8]> {
    let mut lines: Vec<&[u8]> = text.split(|&b| b == b'\n').collect();
    lines.pop();
    lines
}

/// The header of the text, with the instruction needle of the template, if it has one. Like the
/// awk, it keeps one set of what it has seen for the field names and its own keys, so that a field
/// named `heading`, `needle`, or `stray` lands where the awk's output puts it.
pub fn header(text: &[u8], needle: Option<&[u8]>) -> Header {
    let mut header = Header::default();
    let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
    let mut head = true;
    for (i, line) in awk_lines(text).into_iter().enumerate() {
        if head && line.starts_with(b"## ") {
            head = false;
        }
        if head && names_a_field(line) && !line.starts_with(b"- **") {
            header
                .stray
                .push([format!("{}:", i + 1).as_bytes(), line].concat());
        }
        if needle.is_some_and(|n| !n.is_empty() && contains(line, n))
            && seen.insert(b"needle".to_vec())
        {
            header.needle = true;
        }
        if line.starts_with(b"# ADR-") && seen.insert(b"heading".to_vec()) {
            header.heading = Some(line.to_vec());
        }
        if let Some(name) = list_item_field(line).filter(|name| seen.insert(name.to_vec())) {
            match name {
                b"heading" => header.heading = Some(line.to_vec()),
                b"needle" => header.needle = true,
                b"stray" => header.stray.push(line.to_vec()),
                _ => {
                    let name = String::from_utf8_lossy(name).into_owned();
                    header.fields.insert(name, line.to_vec());
                }
            }
        }
    }
    header
}

/// The position of the first occurrence of the needle in the bytes.
pub fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len().max(1))
        .position(|window| window == needle)
}

/// Whether the needle occurs in the bytes.
pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty()
        || haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

/// Whether the line holds `**<field>:**` for a field of the template, anywhere on it.
fn names_a_field(line: &[u8]) -> bool {
    FIELDS
        .iter()
        .any(|field| contains(line, format!("**{field}:**").as_bytes()))
}

/// The name of a list item field `- **<Name>:**` at the start of the line, letters and spaces.
fn list_item_field(line: &[u8]) -> Option<&[u8]> {
    let rest = line.strip_prefix(b"- **")?;
    let end = rest
        .iter()
        .position(|&b| !(b.is_ascii_alphabetic() || b == b' '))?;
    (end > 0 && rest[end..].starts_with(b":**")).then(|| &rest[..end])
}

/// The needle of the template's instruction paragraph: the 40 bytes after the `*` of its first
/// line that starts with `*` and a byte other than `*`, as bash cuts `${line:1:40}` under
/// `LC_ALL=C`, with the backslash escapes awk reads in a `-v` value applied.
pub fn instruction_needle(template: &[u8]) -> Option<Vec<u8>> {
    let line = read_lines(template)
        .into_iter()
        .find(|line| line.first() == Some(&b'*') && line.get(1).is_some_and(|&b| b != b'*'))?;
    let cut = &line[1..line.len().min(41)];
    Some(awk_unescape(cut))
}

/// The value as awk reads a `-v` assignment: `\\`, `\"`, `\/`, `\a`, `\b`, `\f`, `\n`, `\r`,
/// `\t`, `\v`, and up to three octal digits are escapes; any other backslash stays.
fn awk_unescape(value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(value.len());
    let mut i = 0;
    while i < value.len() {
        if value[i] != b'\\' || i + 1 == value.len() {
            out.push(value[i]);
            i += 1;
            continue;
        }
        let next = value[i + 1];
        let simple = match next {
            b'\\' | b'"' | b'/' => Some(next),
            b'a' => Some(0x07),
            b'b' => Some(0x08),
            b'f' => Some(0x0C),
            b'n' => Some(b'\n'),
            b'r' => Some(b'\r'),
            b't' => Some(b'\t'),
            b'v' => Some(0x0B),
            _ => None,
        };
        if let Some(byte) = simple {
            out.push(byte);
            i += 2;
        } else if (b'0'..=b'7').contains(&next) {
            let digits = value[i + 1..]
                .iter()
                .take(3)
                .take_while(|b| (b'0'..=b'7').contains(*b))
                .count();
            let octal = value[i + 1..i + 1 + digits]
                .iter()
                .fold(0u32, |n, b| n * 8 + u32::from(b - b'0'));
            out.push(u8::try_from(octal & 0xFF).unwrap_or(0));
            i += 1 + digits;
        } else {
            out.push(b'\\');
            i += 1;
        }
    }
    out
}

/// The first status emoji of the bytes, by position, and how many they carry in all.
pub fn status_emoji(text: &[u8]) -> (Option<&'static str>, usize) {
    let mut first: Option<(usize, &'static str)> = None;
    let mut count = 0;
    for emoji in STATUS_EMOJI {
        let e = emoji.as_bytes();
        let positions: Vec<usize> = (0..text.len().saturating_sub(e.len() - 1))
            .filter(|&i| text[i..].starts_with(e))
            .collect();
        count += positions.len();
        if let Some(&at) = positions
            .first()
            .filter(|&&at| first.is_none_or(|(p, _)| at < p))
        {
            first = Some((at, emoji));
        }
    }
    (first.map(|(_, emoji)| emoji), count)
}

/// Whether the byte is in the space class of POSIX, which the bash matches with `[[:space:]]`.
pub fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r')
}

/// The bytes without leading and trailing POSIX space.
pub fn trim(text: &[u8]) -> &[u8] {
    let start = text
        .iter()
        .position(|&b| !is_space(b))
        .unwrap_or(text.len());
    let end = text
        .iter()
        .rposition(|&b| !is_space(b))
        .map_or(start, |i| i + 1);
    &text[start..end]
}

/// The first word of the bytes as `read -r word _` splits them with the default IFS: space, tab,
/// and newline, and nothing else.
pub fn first_word(text: &[u8]) -> &[u8] {
    let ifs = |b: &u8| matches!(b, b' ' | b'\t' | b'\n');
    let start = text.iter().position(|b| !ifs(b)).unwrap_or(text.len());
    let end = text[start..]
        .iter()
        .position(ifs)
        .map_or(text.len(), |i| start + i);
    &text[start..end]
}

/// The bytes as text for a message, a byte that is no UTF-8 shown as U+FFFD.
pub fn show(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The headers of every ADR of the directory, by name, read with the template's needle.
pub fn headers(dir: &Dir) -> BTreeMap<&str, Header> {
    let needle = dir.get("template.md").and_then(|t| instruction_needle(t));
    adr_names(dir)
        .into_iter()
        .map(|name| (name, header(&dir[name], needle.as_deref())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adr_names_are_the_numbered_files_then_the_drafts_the_star_empty_too() {
        let dir: Dir = [
            "0002-b.md",
            "0001-a.md",
            "draft-x.md",
            "README.md",
            "template.md",
            "0003.md",
            "0004-.md",
            "draft-.md",
        ]
        .into_iter()
        .map(|n| (n.to_owned(), Vec::new()))
        .collect();
        assert_eq!(
            adr_names(&dir),
            [
                "0001-a.md",
                "0002-b.md",
                "0004-.md",
                "draft-.md",
                "draft-x.md"
            ]
        );
        assert_eq!(
            (key_of("0001-a.md"), key_of("draft-x.md")),
            ("0001", "draft-x")
        );
    }

    #[test]
    fn a_header_is_read_as_the_awk_pass_reads_it() {
        let text = "# ADR-0001: Title\n\n- **Status:** 🟢 accepted\n**Date:** 2026\n\n## Context\n\n- **Status:** 🟡 later\n**Applies to:** body\n";
        let header = header(text.as_bytes(), Some(b"Complete the header"));
        assert_eq!(header.heading.as_deref(), Some(&b"# ADR-0001: Title"[..]));
        assert_eq!(
            header.field("Status"),
            Some("- **Status:** 🟢 accepted".as_bytes())
        );
        assert_eq!(header.stray, [b"4:**Date:** 2026".to_vec()]);
        assert!(!header.needle);
    }

    #[test]
    fn a_field_named_after_a_key_of_the_awk_lands_under_that_key() {
        let header = header(
            b"- **heading:** x\n# ADR-0001: T\n- **needle:** y\n- **stray:** z\n",
            None,
        );
        assert_eq!(header.heading.as_deref(), Some(&b"- **heading:** x"[..]));
        assert!(header.needle);
        assert_eq!(header.stray, [b"- **stray:** z".to_vec()]);
    }

    #[test]
    fn the_needle_is_forty_bytes_after_the_star_with_awk_s_escapes() {
        let template =
            "# T\n\n**Bold** line\n*Complete the header and delete what does not apply. More*\n";
        assert_eq!(
            instruction_needle(template.as_bytes()).as_deref(),
            Some(&b"Complete the header and delete what does"[..])
        );
        let umlauts = "*Füllen Sie den Kopf aus und löschen Sie, was nicht passt.*\n";
        assert_eq!(
            instruction_needle(umlauts.as_bytes()).map(|n| n.len()),
            Some(40)
        );
        assert_eq!(
            instruction_needle(b"*a\\tb \\\\ c\\101\\q*\n").as_deref(),
            Some(&b"a\tb \\ cA\\q*"[..])
        );
    }

    #[test]
    fn the_first_emoji_by_position_and_the_count_of_all() {
        assert_eq!(
            status_emoji("- **Status:** 🟡 proposed | 🟢 accepted".as_bytes()),
            (Some("🟡"), 2)
        );
        assert_eq!(status_emoji(b"no emoji"), (None, 0));
    }

    #[test]
    fn read_drops_an_unterminated_last_line_and_awk_keeps_it() {
        assert_eq!(read_lines(b"a\nb"), [&b"a"[..]]);
        assert_eq!(awk_lines(b"a\nb"), [&b"a"[..], &b"b"[..]]);
        assert_eq!(awk_lines(b"a\n"), [&b"a"[..]]);
    }

    #[test]
    fn the_first_word_splits_at_space_tab_and_newline_only() {
        assert_eq!(first_word(b" accepted\r"), b"accepted\r");
        assert_eq!(
            first_word("\u{a0}accepted".as_bytes()),
            "\u{a0}accepted".as_bytes()
        );
        assert_eq!(first_word(b"\x0Baccepted"), b"\x0Baccepted");
        assert_eq!(first_word(b"\taccepted by X"), b"accepted");
    }
}
