//! The decision record as the template's `scripts/check-docs.sh` and `scripts/lib/adr.sh` read
//! it: which files are ADRs, the header fields of each, its `# ADR-` heading, a header field in
//! another form than the list item, and the template's instruction paragraph left in it. The rules
//! of the record share this reading, so that each reads an ADR the one way the bash does: byte by
//! byte, as awk and bash do under `LC_ALL=C`.

use crate::bash::{awk_lines, contains, find, is_space, read_lines};
use std::collections::{BTreeMap, BTreeSet};

/// The status emoji of the record's legend, as UTF-8 bytes.
pub const STATUS_EMOJI: [&str; 4] = ["🟢", "🟡", "🔴", "⚪"];

/// The header fields of `docs/adr/template.md`.
const FIELDS: &[&str] = &["Status", "Date", "Deciders", "Applies to", "Supersedes"];

/// The header fields of `docs/adr/template.md` from template release v0.9.0 on, `ADR_FIELDS`.
const NAMED_FIELDS: &[&str] = &[
    "Status",
    "Date",
    "Deciders",
    "Applies to",
    "Supersedes",
    "Extends",
    "Narrows",
    "Role",
];

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

/// Whether the name is that of an ADR file from template release v0.9.0 on, as `find -name`
/// matches `[A-Za-z]*-*.md`: a letter of ASCII first, and a hyphen before the `.md` it ends in.
pub fn is_category_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
        && name
            .strip_suffix(".md")
            .is_some_and(|stem| stem.contains('-'))
}

/// Whether the name is that of a numbered file as `find -name` matches `[0-9]*-*.md`, which the
/// record from template release v0.9.0 on refuses.
pub fn is_numbered_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_digit)
        && name
            .strip_suffix(".md")
            .is_some_and(|stem| stem.contains('-'))
}

/// The ADR files of the directory from template release v0.9.0 on, in byte order, as
/// `adr_files` lists them.
pub fn category_names(dir: &Dir) -> Vec<&str> {
    dir.keys()
        .map(String::as_str)
        .filter(|name| is_category_name(name))
        .collect()
}

/// The categories every record has, from template release v0.9.0 on.
pub const FIXED_CATEGORIES: [&str; 4] = ["NUC", "SEC", "OPS", "API"];

/// The categories of the record, as `adr_categories` reads them: the fixed ones, then the key of
/// each item `- **<KEY>**` under a heading `Categories` of the index, up to the next heading.
pub fn categories(index: Option<&[u8]>) -> Vec<String> {
    let mut found: Vec<String> = FIXED_CATEGORIES.iter().map(|c| (*c).to_owned()).collect();
    let mut inside = false;
    for line in index.map(awk_lines).unwrap_or_default() {
        let hashes = line.iter().take_while(|&&b| b == b'#').count();
        if hashes > 0 && line.get(hashes) == Some(&b' ') {
            let rest = &line[hashes + 1..];
            let end = rest.len() - rest.iter().rev().take_while(|&&b| is_space(b)).count();
            inside = &rest[..end] == b"Categories";
            continue;
        }
        if !inside {
            continue;
        }
        if let Some(rest) = line.strip_prefix(b"- **") {
            let key = rest.iter().take_while(|b| b.is_ascii_uppercase()).count();
            if key > 0 && rest[key..].starts_with(b"**") {
                found.push(String::from_utf8_lossy(&rest[..key]).into_owned());
            }
        }
    }
    found
}

/// Whether the identifier has the form `<CAT>-<slug>` of `ADR_NEW_RE`: two to five capital
/// letters, a hyphen, and two to five words of lower case letters and digits joined by hyphens.
pub fn is_identifier(id: &str) -> bool {
    let Some((category, slug)) = id.split_once('-') else {
        return false;
    };
    let words: Vec<&str> = slug.split('-').collect();
    (2..=5).contains(&category.len())
        && category.bytes().all(|b| b.is_ascii_uppercase())
        && (2..=5).contains(&words.len())
        && words.iter().all(|w| {
            !w.is_empty()
                && w.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

/// The first line of the file that starts with `# `, as the bash's awk pass takes the heading
/// from template release v0.9.0 on.
pub fn title(text: &[u8]) -> Option<&[u8]> {
    awk_lines(text)
        .into_iter()
        .find(|line| line.starts_with(b"# "))
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
    /// The Supersedes field with its continuation lines, each ended by a newline, as the bash
    /// collects it.
    pub supersedes: Vec<u8>,
    /// The Deciders field with its continuation lines, each ended by a newline, as the bash
    /// collects it.
    pub deciders: Vec<u8>,
    /// The Extends field of a record from template release v0.9.0 on, with its continuation
    /// lines, each ended by a newline, as the bash collects it.
    pub extends: Vec<u8>,
    /// The Narrows field, likewise.
    pub narrows: Vec<u8>,
}

impl Header {
    /// The first line of the field, as `grep -m1 '^- **<name>:**'` finds it.
    pub fn field(&self, name: &str) -> Option<&[u8]> {
        self.fields.get(name).map(Vec::as_slice)
    }

    /// Adds a line of a field the awk continues, ended by a newline.
    fn continued(&mut self, field: Continued, line: &[u8]) {
        let text = match field {
            Continued::Supersedes => &mut self.supersedes,
            Continued::Deciders => &mut self.deciders,
            Continued::Extends => &mut self.extends,
            Continued::Narrows => &mut self.narrows,
        };
        text.extend_from_slice(line);
        text.push(b'\n');
    }
}

/// A field whose continuation lines the awk reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Continued {
    Supersedes,
    Deciders,
    Extends,
    Narrows,
}

/// The header of the text, with the instruction needle of the template, if it has one. Like the
/// awk, it keeps one set of what it has seen for the field names and its own keys, so that a field
/// named `heading`, `needle`, or `stray` lands where the awk's output puts it.
pub fn header(text: &[u8], needle: Option<&[u8]>) -> Header {
    header_of(text, needle, FIELDS, b"# ADR-")
}

/// The header as the awk pass of a record named by category reads it, from template release
/// v0.9.0 on: a field in another form is any of `ADR_FIELDS`, Extends, Narrows, and Role included,
/// and the heading is the first line that starts with `# `.
pub fn named_header(text: &[u8], needle: Option<&[u8]>) -> Header {
    header_of(text, needle, NAMED_FIELDS, b"# ")
}

/// The header, a field in another form being one of `fields`, the heading the first line that
/// starts with `heading`.
fn header_of(text: &[u8], needle: Option<&[u8]>, fields: &[&str], heading: &[u8]) -> Header {
    let mut header = Header::default();
    let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
    let mut head = true;
    let mut continuing: Option<Continued> = None;
    for (i, line) in awk_lines(text).into_iter().enumerate() {
        if head && line.starts_with(b"## ") {
            head = false;
        }
        if head && names_a_field(line, fields) && !line.starts_with(b"- **") {
            header
                .stray
                .push([format!("{}:", i + 1).as_bytes(), line].concat());
        }
        if needle.is_some_and(|n| !n.is_empty() && contains(line, n))
            && seen.insert(b"needle".to_vec())
        {
            header.needle = true;
        }
        if line.starts_with(heading) && seen.insert(b"heading".to_vec()) {
            header.heading = Some(line.to_vec());
        }
        // A field the awk continues, Supersedes or Deciders, on lines that open with space.
        if let Some(field) = continuing.filter(|_| continues(line)) {
            header.continued(field, line);
            continue;
        }
        continuing = None;
        if let Some(name) = list_item_field(line).filter(|name| seen.insert(name.to_vec())) {
            match name {
                b"heading" => header.heading = Some(line.to_vec()),
                b"needle" => header.needle = true,
                b"stray" => header.stray.push(line.to_vec()),
                _ => {
                    continuing = match name {
                        b"Supersedes" => Some(Continued::Supersedes),
                        b"Deciders" => Some(Continued::Deciders),
                        b"Extends" => Some(Continued::Extends),
                        b"Narrows" => Some(Continued::Narrows),
                        _ => None,
                    };
                    if let Some(field) = continuing {
                        header.continued(field, line);
                    }
                    let name = String::from_utf8_lossy(name).into_owned();
                    header.fields.insert(name, line.to_vec());
                }
            }
        }
    }
    header
}

/// Whether the line holds `**<field>:**` for a field of the template, anywhere on it.
fn names_a_field(line: &[u8], fields: &[&str]) -> bool {
    fields
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

/// Whether the line continues a field: POSIX space, then something else.
fn continues(line: &[u8]) -> bool {
    let spaces = line.iter().take_while(|&&b| is_space(b)).count();
    spaces > 0 && spaces < line.len()
}

/// The length of the ADR reference that starts at the bytes, as `ADR-([0-9]{4}|draft-<slug>)`
/// matches there, longest first: four digits, or `draft-` and the longest slug.
pub fn reference_at(text: &[u8]) -> Option<usize> {
    let rest = text.strip_prefix(b"ADR-")?;
    if rest.len() >= 4 && rest[..4].iter().all(u8::is_ascii_digit) {
        return Some(8);
    }
    let slug = rest.strip_prefix(b"draft-")?;
    let length = slug_length(slug);
    (length > 0).then_some(b"ADR-draft-".len() + length)
}

/// The length of the longest slug at the start of the bytes: lower case letters and digits in
/// runs joined by single hyphens.
pub fn slug_length(text: &[u8]) -> usize {
    let word = |b: &u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    let mut end = text.iter().take_while(|b| word(b)).count();
    if end == 0 {
        return 0;
    }
    while text.get(end) == Some(&b'-') && text.get(end + 1).is_some_and(word) {
        end += 1 + text[end + 1..].iter().take_while(|b| word(b)).count();
    }
    end
}

/// Every ADR reference of the bytes, in order, as `grep -o` and a loop over `=~` find them:
/// leftmost first, each the longest, the search going on after it. Each is the position and the
/// reference, `ADR-NNNN` or `ADR-draft-<slug>`.
pub fn references(text: &[u8]) -> Vec<(usize, &[u8])> {
    let mut found = Vec::new();
    let mut i = 0;
    while let Some(at) = find(&text[i..], b"ADR-") {
        let start = i + at;
        match reference_at(&text[start..]) {
            Some(length) => {
                found.push((start, &text[start..start + length]));
                i = start + length;
            }
            None => i = start + 1,
        }
    }
    found
}

/// The key after `ADR-` of a reference: its number, or `draft-<slug>`.
pub fn reference_key(reference: &[u8]) -> &[u8] {
    reference.strip_prefix(b"ADR-").unwrap_or(reference)
}

/// The paths of the ADRs whose status line carries ⚪ or 🔴, which the bash's reference checks
/// pass over.
pub fn inactive_paths(dir: &Dir) -> BTreeSet<String> {
    headers(dir)
        .into_iter()
        .filter(|(_, header)| {
            let status = header.field("Status").unwrap_or(b"");
            contains(status, "⚪".as_bytes()) || contains(status, "🔴".as_bytes())
        })
        .map(|(name, _)| format!("docs/adr/{name}"))
        .collect()
}

/// The first file of the record for each key, as the bash fills ADR_BY_KEY.
pub fn by_key(dir: &Dir) -> BTreeMap<&str, &str> {
    let mut keys = BTreeMap::new();
    for name in adr_names(dir) {
        keys.entry(key_of(name)).or_insert(name);
    }
    keys
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

/// The ADRs of a record named by category, from template release v0.9.0 on, with their headers.
pub fn named_headers(dir: &Dir) -> BTreeMap<&str, Header> {
    let needle = dir.get("template.md").and_then(|t| instruction_needle(t));
    category_names(dir)
        .into_iter()
        .map(|name| (name, named_header(&dir[name], needle.as_deref())))
        .collect()
}

/// The paths of the superseded and rejected ADRs of a record named by category, as
/// `is_inactive_adr` reads their status.
pub fn named_inactive_paths(dir: &Dir) -> BTreeSet<String> {
    named_headers(dir)
        .into_iter()
        .filter(|(_, header)| {
            let status = header.field("Status").unwrap_or(b"");
            contains(status, "⚪".as_bytes()) || contains(status, "🔴".as_bytes())
        })
        .map(|(name, _)| format!("docs/adr/{name}"))
        .collect()
}

/// Whether the byte is one of `ADR_WORD_CHARS`, `A-Za-z0-9_-`.
fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

/// The length of the longest `<CAT>-<slug>` at the start of the text, one of the categories and a
/// slug of two to five words, such that `bounded` asks the byte after it to be none of the word
/// characters; as `ADR_REF_RE` matches, leftmost and longest.
fn reference_length(text: &[u8], categories: &[String], bounded: bool) -> Option<usize> {
    let mut best = None;
    for category in categories {
        let Some(rest) = text.strip_prefix(category.as_bytes()) else {
            continue;
        };
        let Some(slug) = rest.strip_prefix(b"-") else {
            continue;
        };
        let mut words = 0;
        let mut at = 0;
        loop {
            let len = slug[at..]
                .iter()
                .take_while(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                .count();
            if len == 0 {
                break;
            }
            words += 1;
            at += len;
            let end = category.len() + 1 + at;
            if words >= 2 && (!bounded || text.get(end).is_none_or(|&b| !is_word(b))) {
                best = best.max(Some(end));
            }
            if words == 5 || slug.get(at) != Some(&b'-') {
                break;
            }
            at += 1;
        }
    }
    best
}

/// The references `<CAT>-<slug>` of the line, as the bash's loop over `ADR_REF_WORD_RE` finds them
/// from template release v0.9.0 on: each with none of the word characters `A-Za-z0-9_-` before or
/// after it, the leftmost and longest first, the search going on after it.
pub fn named_references<'a>(line: &'a [u8], categories: &[String]) -> Vec<&'a [u8]> {
    let mut found = Vec::new();
    let mut i = 0;
    while i < line.len() {
        if (i == 0 || !is_word(line[i - 1]))
            && let Some(len) = reference_length(&line[i..], categories, true)
        {
            found.push(&line[i..i + len]);
            i += len;
            continue;
        }
        i += 1;
    }
    found
}

/// Whether the line holds `ADR_REF_RE` anywhere, as the bash's `grep -E` picks the lines it reads.
pub fn holds_reference(line: &[u8], categories: &[String]) -> bool {
    (0..line.len()).any(|i| reference_length(&line[i..], categories, false).is_some())
}

/// The line with each code span between two backticks taken out, as `code_spans_out` does.
pub fn code_spans_out(line: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.iter().position(|&b| b == b'`') {
        let Some(close) = rest[open + 1..].iter().position(|&b| b == b'`') else {
            break;
        };
        out.extend_from_slice(&rest[..open]);
        rest = &rest[open + 1 + close + 1..];
    }
    out.extend_from_slice(rest);
    out
}

/// The lines of the section `## <name>` of an ADR, from its heading to the next `## ` heading,
/// neither read inside a fence, as `adr_section` prints them with the fences of `ADR_FENCE_AWK`.
pub fn section<'a>(text: &'a [u8], name: &str) -> Vec<&'a [u8]> {
    let heading = format!("## {name}");
    let mut fence: Option<(u8, usize)> = None;
    let mut inside = false;
    let mut found = Vec::new();
    for line in awk_lines(text) {
        let trimmed = &line[line.iter().take_while(|&&b| is_space(b)).count()..];
        let run = |c: u8| trimmed.iter().take_while(|&&b| b == c).count();
        let fenced = match fence {
            None => {
                let c = trimmed.first().copied().unwrap_or(0);
                let n = if c == b'`' || c == b'~' { run(c) } else { 0 };
                if n >= 3 && !(c == b'`' && trimmed[n..].contains(&b'`')) {
                    fence = Some((c, n));
                    true
                } else {
                    false
                }
            }
            Some((c, n)) => {
                let m = run(c);
                if m >= n && trimmed[m..].iter().all(|&b| is_space(b)) {
                    fence = None;
                }
                true
            }
        };
        if !fenced && line == heading.as_bytes() {
            inside = true;
            found.push(line);
            continue;
        }
        if inside && !fenced && line.starts_with(b"## ") {
            break;
        }
        if inside {
            found.push(line);
        }
    }
    found
}

/// The successor a status line names, as `superseding_id` reads it: `superseded by`, an optional
/// `[`, and a `<CAT>-<slug>` with no word character after it.
pub fn named_successor(status: &[u8], categories: &[String]) -> Option<String> {
    let needle = b"superseded by ";
    let mut from = 0;
    while let Some(at) = find(&status[from..], needle) {
        let start = from + at + needle.len();
        let rest = &status[start..];
        let rest = rest.strip_prefix(b"[").unwrap_or(rest);
        if let Some(len) = reference_length(rest, categories, true) {
            return Some(show(&rest[..len]));
        }
        from = start;
    }
    None
}

/// The clauses an Extends or Narrows field names, `NUC-<slug>` and the number of `(NUC-<slug>) ?§
/// ?([0-9]+)`, each leftmost and longest, the search going on after it.
pub fn clauses(field: &[u8]) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut i = 0;
    'scan: while i < field.len() {
        let rest = &field[i..];
        if rest.starts_with(b"NUC-") {
            // The longest slug after which the clause follows.
            let mut best: Option<(usize, usize)> = None;
            let mut slug_end = Vec::new();
            let mut at = 4;
            for _ in 0..5 {
                let len = rest[at..]
                    .iter()
                    .take_while(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                    .count();
                if len == 0 {
                    break;
                }
                at += len;
                slug_end.push(at);
                if rest.get(at) != Some(&b'-') {
                    break;
                }
                at += 1;
            }
            for &end in slug_end.iter().skip(1) {
                let mut j = end;
                if rest.get(j) == Some(&b' ') {
                    j += 1;
                }
                if !rest[j..].starts_with("§".as_bytes()) {
                    continue;
                }
                j += "§".len();
                if rest.get(j) == Some(&b' ') {
                    j += 1;
                }
                let digits = rest[j..].iter().take_while(|b| b.is_ascii_digit()).count();
                if digits > 0 && best.is_none_or(|(_, e)| j + digits > e) {
                    best = Some((end, j + digits));
                }
            }
            if let Some((end, total)) = best {
                let number = rest[total
                    - rest[..total]
                        .iter()
                        .rev()
                        .take_while(|b| b.is_ascii_digit())
                        .count()..total]
                    .to_vec();
                found.push((show(&rest[..end]), show(&number)));
                i += total;
                continue 'scan;
            }
        }
        i += 1;
    }
    found
}

/// The length of `(<REF>) §[0-9]+((, |,? and |–)§[0-9]+)*` at the start of the text, the longest
/// a slug of two to five words allows, as `clauses_out` matches it.
fn clause_citation_length(text: &[u8], categories: &[String]) -> Option<usize> {
    let sign = "§".as_bytes();
    let number = |at: usize| -> Option<usize> {
        let rest = text.get(at..)?.strip_prefix(sign)?;
        let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
        (digits > 0).then_some(at + sign.len() + digits)
    };
    let mut best = None;
    for category in categories {
        let Some(slug) = text
            .strip_prefix(category.as_bytes())
            .and_then(|r| r.strip_prefix(b"-"))
        else {
            continue;
        };
        let mut at = 0;
        for words in 1..=5 {
            let len = slug[at..]
                .iter()
                .take_while(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                .count();
            if len == 0 {
                break;
            }
            at += len;
            let end = category.len() + 1 + at;
            if words >= 2
                && text.get(end) == Some(&b' ')
                && let Some(mut stop) = number(end + 1)
            {
                loop {
                    let next = [&b", "[..], b", and ", b" and ", "–".as_bytes()]
                        .iter()
                        .filter(|sep| text[stop..].starts_with(sep))
                        .filter_map(|sep| number(stop + sep.len()))
                        .max();
                    match next {
                        Some(n) => stop = n,
                        None => break,
                    }
                }
                best = best.max(Some(stop));
            }
            if slug.get(at) != Some(&b'-') {
                break;
            }
            at += 1;
        }
    }
    best
}

/// The line with each clause citation of an ADR, as `<CAT>-<slug> §4, §5` or `§2–§5`, taken out,
/// as `clauses_out` takes them out from template release v0.9.0 on.
pub fn clauses_out(line: &[u8], categories: &[String]) -> Vec<u8> {
    let mut out = line.to_vec();
    'again: loop {
        for i in 0..out.len() {
            if let Some(len) = clause_citation_length(&out[i..], categories) {
                out.drain(i..i + len);
                continue 'again;
            }
        }
        return out;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::adr_id;

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
        let heading = format!("# {}: Title", adr_id(1));
        let text = format!(
            "{heading}\n\n- **Status:** 🟢 accepted\n**Date:** 2026\n\n## Context\n\n- **Status:** 🟡 later\n**Applies to:** body\n"
        );
        let header = header(text.as_bytes(), Some(b"Complete the header"));
        assert_eq!(header.heading.as_deref(), Some(heading.as_bytes()));
        assert_eq!(
            header.field("Status"),
            Some("- **Status:** 🟢 accepted".as_bytes())
        );
        assert_eq!(header.stray, [b"4:**Date:** 2026".to_vec()]);
        assert!(!header.needle);
    }

    #[test]
    fn a_field_named_after_a_key_of_the_awk_lands_under_that_key() {
        let text = format!(
            "- **heading:** x\n# {}: T\n- **needle:** y\n- **stray:** z\n",
            adr_id(1)
        );
        let header = header(text.as_bytes(), None);
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
    fn a_named_reference_stands_alone_and_is_the_longest() {
        let cats = ["SEC".to_owned(), "UX".to_owned()];
        let id = crate::testing::cat_id;
        let line = format!(
            "a {} b {} c x{} {} {}",
            id("SEC", "a-b"),
            id("UX", "c-d-e"),
            id("SEC", "q-r"),
            id("SEC", "one-two-three-four-five-six"),
            id("API", "a-b")
        );
        let found: Vec<String> = named_references(line.as_bytes(), &cats)
            .into_iter()
            .map(show)
            .collect();
        assert_eq!(found, [id("SEC", "a-b"), id("UX", "c-d-e")]);
        assert!(holds_reference(
            format!("x{}", id("SEC", "q-r")).as_bytes(),
            &cats
        ));
        assert!(!holds_reference(b"SEC-once", &cats));
    }

    #[test]
    fn code_spans_go_and_a_lone_backtick_stays() {
        assert_eq!(code_spans_out(b"a `x` b `y` c"), b"a  b  c");
        assert_eq!(code_spans_out(b"a `x"), b"a `x");
    }

    #[test]
    fn a_section_ends_at_the_next_heading_outside_a_fence() {
        let text = b"# T\n## Decision\n1. One.\n```\n## not\n```\n2. Two.\n## Next\n3. No.\n";
        let lines: Vec<String> = section(text, "Decision").into_iter().map(show).collect();
        assert_eq!(
            lines,
            ["## Decision", "1. One.", "```", "## not", "```", "2. Two."]
        );
    }

    #[test]
    fn a_successor_and_the_clauses_are_read_as_the_bash_reads_them() {
        let cats = ["SEC".to_owned()];
        let id = crate::testing::cat_id;
        let line = format!(
            "- **Status:** x superseded by [{}](y.md)",
            id("SEC", "new-rule")
        );
        assert_eq!(
            named_successor(line.as_bytes(), &cats),
            Some(id("SEC", "new-rule"))
        );
        assert_eq!(named_successor(b"superseded by nothing", &cats), None);
        let field = format!(
            "[{} §2](x), {} § 13",
            id("NUC", "base-rule"),
            id("NUC", "other-one")
        );
        assert_eq!(
            clauses(field.as_bytes()),
            [
                (id("NUC", "base-rule"), "2".to_owned()),
                (id("NUC", "other-one"), "13".to_owned())
            ]
        );
        assert!(clauses(format!("{} \u{a7}1", id("SEC", "x-y")).as_bytes()).is_empty());
    }

    #[test]
    fn clause_citations_go_and_a_section_sign_alone_stays() {
        let cats = ["NUC".to_owned(), "SEC".to_owned()];
        let id = crate::testing::cat_id;
        let line = format!(
            "see {} §4, §5 and §6, {} §2–§3, and AGENTS.md §9",
            id("NUC", "base-rule"),
            id("SEC", "a-rule")
        );
        assert_eq!(
            show(&clauses_out(line.as_bytes(), &cats)),
            "see , , and AGENTS.md §9"
        );
        let unknown = format!("{} §4", id("UX", "dark-mode"));
        assert_eq!(show(&clauses_out(unknown.as_bytes(), &cats)), unknown);
    }
}
