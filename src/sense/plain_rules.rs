//! `nuc sense plain-rules`: the rules a change adds to, read as the template's
//! `scripts/sensor-plain-rules.sh` and `scripts/lib/rules.sh` of the tree's release read them:
//! each sentence over 25 words that holds an added line, each soft modal verb, and each word the
//! glossary lists under _Avoid:_ on an added line; with `--require`, a soft modal verb or an
//! avoided word in a project ADR fails. The shell resolves the change, lists the files it touches,
//! and reads each with its added lines, the glossary, and the ADR index as the change has them;
//! the sensor reads them as the bash's awk does, byte by byte under `LC_ALL=C`, as mawk reads them.

use std::collections::{BTreeMap, BTreeSet};

use super::{Refusal, Sensor};
use crate::adr;
use crate::bash::{self, awk_lines, contains, find, is_space};
use crate::config::Version;

/// The sensor, which reports from v0.8.0, the release that brings it.
pub const SENSOR: Sensor = Sensor {
    name: "plain-rules",
    since: Version(0, 8, 0),
    changes: &[HELD, REQUIRE, KEYED, ARGUMENTS, TOKENS],
    reasons: &[
        "file",
        "filter",
        "git",
        "name",
        "ref",
        super::DRIVER,
        super::ENVIRONMENT,
        super::HUNK,
        super::RELEASE,
    ],
};

/// The label that opens each line of the report.
pub const LABEL: &str = "Plain rules sensor";

/// The release from which a run of prose is held until it ends, so that a code span or an italic
/// may wrap; fenced code ends as `adr_section` reads it to end; a listed word counts across the
/// line break of a paragraph; and a sub-list ends an _Avoid:_ list.
pub const HELD: Version = Version(0, 8, 1);

/// The release from which `--require` fails on a project ADR; a renamed file adds the lines its
/// rename changes; an ADR is a file named by category; the glossary's items wrap, carry a
/// qualifier, and may be unreadable; and a word inside a longer term of the glossary does not
/// count.
pub const REQUIRE: Version = Version(0, 9, 0);

/// The release from which each finding ends in its key, and a token without a letter or a digit
/// ends a sentence wherever it holds a stop.
pub const KEYED: Version = Version(0, 9, 1);

/// The release from which an argument past the base and the head is refused, where the releases
/// before ignore it.
pub const ARGUMENTS: Version = Version(0, 10, 0);

/// The release from which an ADR identifier of a category the index defines reads as one token.
pub const TOKENS: Version = Version(0, 11, 1);

/// The longest sentence the rules may hold, `MAX_WORDS`.
const MAX_WORDS: usize = 25;

/// The soft modal verbs, in the order the bash looks for them.
const SOFT: [&[u8]; 4] = [b"should", b"could", b"might", b"ideally"];

/// Whether the change touches the path as a file of rules: `AGENTS.md`, or an ADR file directly in
/// `docs/adr/`, as `adr_is_file` of the release names it.
pub fn is_rule_file(path: &[u8], release: Version) -> bool {
    if path == b"AGENTS.md" {
        return true;
    }
    let Some(name) = path.strip_prefix(b"docs/adr/") else {
        return false;
    };
    if name.contains(&b'/') {
        return false;
    }
    let Some(stem) = name.strip_suffix(b".md") else {
        return false;
    };
    if release >= REQUIRE {
        // `[A-Za-z]+-[^/]*\.md`
        let letters = stem.iter().take_while(|b| b.is_ascii_alphabetic()).count();
        letters > 0 && stem.get(letters) == Some(&b'-')
    } else {
        // `([0-9]{4}-[^/]*|draft-[^/]*)\.md`
        (stem.len() >= 5 && stem[..4].iter().all(u8::is_ascii_digit) && stem[4] == b'-')
            || stem.starts_with(b"draft-")
    }
}

/// The files the change touches as `rule_stream` lists them: the last path of each line of
/// `git diff --name-status`, with the path a rename came from, from `REQUIRE` (before it the lines
/// of `--name-only`, each a path alone), and each untracked file; sorted by their bytes and each
/// once, as `LC_ALL=C sort -u` sorts the lines `<path><TAB><from>`.
pub fn listed(diff: &[u8], untracked: &[u8], release: Version) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut lines = BTreeSet::new();
    for line in awk_lines(diff) {
        let fields: Vec<&[u8]> = line.split(|&b| b == b'\t').collect();
        let (path, from): (&[u8], &[u8]) = if release >= REQUIRE {
            let last = fields.last().copied().unwrap_or_default();
            let renamed = fields[0].starts_with(b"R");
            (
                last,
                if renamed {
                    fields.get(1).copied().unwrap_or_default()
                } else {
                    b""
                },
            )
        } else {
            (line, b"")
        };
        lines.insert([path, b"\t", from].concat());
    }
    for line in awk_lines(untracked) {
        lines.insert([line, b"\t"].concat());
    }
    lines
        .into_iter()
        .filter_map(|line| {
            let tab = line.iter().position(|&b| b == b'\t')?;
            let path = line[..tab].to_vec();
            (!path.is_empty()).then(|| (path, line[tab + 1..].to_vec()))
        })
        .collect()
}

/// What a listing of `git diff --name-status` holds for rename detection: whether it names an old
/// side, a path deleted or renamed, and the paths on the new side of an addition or a rename, each
/// as its bytes, the quoting git gives a name undone.
pub struct NewSides {
    pub sources: bool,
    pub added: Vec<Vec<u8>>,
}

/// The new sides of the listing, as [`NewSides`] holds them.
pub fn new_sides(diff: &[u8]) -> NewSides {
    let mut sides = NewSides {
        sources: false,
        added: Vec::new(),
    };
    for line in awk_lines(diff) {
        let fields: Vec<&[u8]> = line.split(|&b| b == b'\t').collect();
        let status = fields[0];
        sides.sources |= status.starts_with(b"D") || status.starts_with(b"R");
        if status.starts_with(b"A") || status.starts_with(b"R") {
            sides
                .added
                .push(unquoted(fields.last().copied().unwrap_or_default()));
        }
    }
    sides
}

/// The name as git writes it unquoted: one in double quotes, as git writes a name with a `"`, a
/// `\`, or a control byte even under `core.quotePath=false`, with the escapes of C read as awk
/// reads them, each of which git's quoting writes; any other name as it stands.
pub fn unquoted(name: &[u8]) -> Vec<u8> {
    match name
        .strip_prefix(b"\"")
        .and_then(|rest| rest.strip_suffix(b"\""))
    {
        Some(inner) => adr::awk_unescape(inner),
        None => name.to_vec(),
    }
}

/// The active filter drivers of the configuration, from the output of
/// `git config -z --get-regexp '^filter\.'`, `<key> LF <value> NUL` a key: those that any entry
/// gives a `clean` or `process` command that is not empty, which git may run. An entry that empties
/// one later leaves it active, the side that never passes.
pub fn filter_drivers(config: &[u8]) -> BTreeSet<Vec<u8>> {
    let mut active = BTreeSet::new();
    for entry in config.split(|&b| b == 0).filter(|e| !e.is_empty()) {
        let (key, value) = match entry.iter().position(|&b| b == b'\n') {
            Some(at) => (&entry[..at], &entry[at + 1..]),
            None => (entry, &b""[..]),
        };
        // `filter.<name>.<variable>`, the name between the first and the last dot.
        let Some(rest) = key.strip_prefix(b"filter.") else {
            continue;
        };
        let Some(dot) = rest.iter().rposition(|&b| b == b'.') else {
            continue;
        };
        let (name, variable) = (&rest[..dot], &rest[dot + 1..]);
        if (variable == b"clean" || variable == b"process") && !value.is_empty() {
            active.insert(name.to_vec());
        }
    }
    active
}

/// The lines a diff of `git diff -U0` adds, by the new side of each hunk header as the bash's awk
/// reads its third field; not decided on a number awk reads inexactly.
pub fn added_lines(diff: &[u8]) -> Result<BTreeSet<usize>, Refusal> {
    let overflow = super::hunk_refusal(SENSOR.name);
    let mut added = BTreeSet::new();
    for line in awk_lines(diff) {
        let Some((start, count)) = bash::hunk_third_field(line).map_err(&overflow)? else {
            continue;
        };
        // `start + count`, the bound of awk's loop, one past the last line it adds: kept within
        // what awk holds exactly, one value early at worst.
        let end = bash::awk_sum(&[start, count]).map_err(&overflow)?;
        let usize_of = |n: u64| usize::try_from(n).map_err(|_| overflow(bash::Overflow));
        added.extend(usize_of(start)?..usize_of(end)?);
    }
    Ok(added)
}

/// Every line of the file, as an untracked file adds them.
pub fn every_line(text: &[u8]) -> BTreeSet<usize> {
    (1..=awk_lines(text).len()).collect()
}

/// A file of rules the change touches: its path, its text as the change has it, and the lines the
/// change adds.
pub struct RuleFile {
    pub path: Vec<u8>,
    pub text: Vec<u8>,
    pub added: BTreeSet<usize>,
}

/// What the run of the sensor gives: the report, and whether `--require` fails on it.
pub struct Outcome {
    pub report: Vec<u8>,
    pub failed: bool,
}

/// The report on the files of rules, in the order `listed` gives, with the glossary and the ADR
/// index as the change has them, empty where it has none; `range` names the change.
pub fn report(
    release: Version,
    range: &str,
    files: &[RuleFile],
    glossary: &[u8],
    index: &[u8],
    require: bool,
) -> Outcome {
    let mut rows = Vec::new();
    for file in files {
        rows.push(Row::File(file.path.clone()));
        rows.extend(read_rules(release, file));
    }
    let avoid = avoid_terms(glossary, release);
    let categories = if release >= TOKENS {
        adr::categories(Some(index))
    } else {
        Vec::new()
    };
    let findings = Findings::new(release, &avoid, categories).run(&rows);
    let mut failing = 0;
    let mut lines = Vec::new();
    for (fails, mut line) in findings {
        if fails && require {
            failing += 1;
            if release >= KEYED {
                let at = rfind(&line, b" [").unwrap_or(line.len());
                let key = line[at..].strip_prefix(b" [").unwrap_or(&line[..]).to_vec();
                line.truncate(at);
                line.extend_from_slice(b" (fails) [");
                line.extend_from_slice(&key);
            } else {
                line.extend_from_slice(b" (fails)");
            }
        }
        lines.push(line);
    }
    let mut out = Vec::new();
    if lines.is_empty() {
        out.extend_from_slice(
            format!(
                "{LABEL} ({range}): no long sentence, no soft modal verb, and no avoided term on \
                 the lines the change adds to the rules.\n"
            )
            .as_bytes(),
        );
        return Outcome {
            report: out,
            failed: false,
        };
    }
    out.extend_from_slice(
        format!(
            "{LABEL} ({range}): {} to read on the lines the change adds to the rules (AGENTS.md \
             section 4):\n",
            lines.len()
        )
        .as_bytes(),
    );
    for line in &lines {
        out.extend_from_slice(b"  - ");
        out.extend_from_slice(line);
        out.push(b'\n');
    }
    if failing > 0 {
        out.extend_from_slice(
            format!(
                "{LABEL}: {failing} of them fail: a soft modal verb or an avoided word in a \
                 project ADR (NUC-documents-are-fixed §5).\n"
            )
            .as_bytes(),
        );
    }
    Outcome {
        report: out,
        failed: failing > 0,
    }
}

/// The position of the last occurrence of the needle.
fn rfind(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).rposition(|w| w == needle)
}

/// A row of `rule_stream`: a file, the end of a run of prose, or a line of the rules with whether
/// the change adds it and its text as the reader leaves it.
#[derive(Debug, PartialEq, Eq)]
enum Row {
    File(Vec<u8>),
    Para,
    Line {
        nr: usize,
        added: bool,
        text: Vec<u8>,
    },
}

/// The line after its leading space, where `^[[:space:]]*` ends.
fn after_space(line: &[u8]) -> &[u8] {
    let n = line.iter().take_while(|&&b| is_space(b)).count();
    &line[n..]
}

/// Whether the line opens a list item: `^[[:space:]]*([-*+]|[0-9]+\.)[[:space:]]`.
fn is_item(line: &[u8]) -> bool {
    let t = after_space(line);
    let marker = match t.first() {
        Some(b'-' | b'*' | b'+') => 1,
        Some(b) if b.is_ascii_digit() => {
            let digits = t.iter().take_while(|b| b.is_ascii_digit()).count();
            if t.get(digits) == Some(&b'.') {
                digits + 1
            } else {
                return false;
            }
        }
        _ => return false,
    };
    t.get(marker).is_some_and(|&b| is_space(b))
}

/// Whether the line ends a run of prose and is no prose itself: a heading, a block quote, a table,
/// or a blank line.
fn is_break(line: &[u8]) -> bool {
    let t = after_space(line);
    line.starts_with(b"#") || matches!(t.first(), Some(b'>' | b'|') | None)
}

/// The rows of one file's rules, as the awk of `rule_stream` of the release reads them.
fn read_rules(release: Version, file: &RuleFile) -> Vec<Row> {
    if release >= HELD {
        Reader::default().read(file)
    } else {
        read_rules_first(file)
    }
}

/// `ADR_FENCE_AWK`: fenced code as `fenced()` reads it.
#[derive(Default)]
struct Fence {
    open: Option<(u8, usize)>,
}

impl Fence {
    /// 2 on the line that opens a fence, 1 inside it and on the line that closes it, 0 elsewhere.
    fn fenced(&mut self, line: &[u8]) -> u8 {
        let t = after_space(line);
        let run = |c: u8| t.iter().take_while(|&&b| b == c).count();
        match self.open {
            None => {
                if !(t.starts_with(b"```") || t.starts_with(b"~~~")) {
                    return 0;
                }
                let n = run(t[0]);
                if t[0] == b'`' && t[n..].contains(&b'`') {
                    return 0;
                }
                self.open = Some((t[0], n));
                2
            }
            Some((c, len)) => {
                let n = run(c);
                if n >= len && t[n..].iter().all(|&b| is_space(b)) {
                    self.open = None;
                }
                1
            }
        }
    }
}

/// The reader of `rule_stream` from `HELD`: a run of prose held until it ends.
#[derive(Default)]
struct Reader {
    rows: Vec<Row>,
    gap: bool,
    held: Vec<usize>,
    buf: Vec<u8>,
}

impl Reader {
    fn read(mut self, file: &RuleFile) -> Vec<Row> {
        let adr = file.path != b"AGENTS.md";
        let mut fence = Fence::default();
        let mut inside = false;
        for (i, line) in awk_lines(&file.text).into_iter().enumerate() {
            match fence.fenced(line) {
                2 => {
                    self.para(file);
                    continue;
                }
                1 => continue,
                _ => {}
            }
            if line.starts_with(b"## ") {
                self.para(file);
                if adr {
                    inside = line == b"## Decision" || line == b"## Enforcement";
                }
                continue;
            }
            if adr && !inside {
                continue;
            }
            if is_break(line) {
                self.para(file);
                continue;
            }
            if is_item(line) {
                self.para(file);
            }
            if !self.held.is_empty() {
                self.buf.push(b'\n');
            }
            self.held.push(i + 1);
            self.buf.extend_from_slice(line);
        }
        self.flush(file);
        self.rows
    }

    fn para(&mut self, file: &RuleFile) {
        self.flush(file);
        if !self.gap {
            self.rows.push(Row::Para);
        }
        self.gap = true;
    }

    fn flush(&mut self, file: &RuleFile) {
        if self.held.is_empty() {
            return;
        }
        let plain = plain(&self.buf);
        let parts: Vec<&[u8]> = plain.split(|&b| b == b'\n').collect();
        for (i, &nr) in self.held.iter().enumerate() {
            self.rows.push(Row::Line {
                nr,
                added: file.added.contains(&nr),
                text: parts.get(i).copied().unwrap_or_default().to_vec(),
            });
        }
        self.held.clear();
        self.buf.clear();
        self.gap = false;
    }
}

/// The line breaks of the bytes, all that is kept of a code span or an italic.
fn breaks(bytes: &[u8]) -> impl Iterator<Item = u8> + '_ {
    bytes.iter().copied().filter(|&b| b == b'\n')
}

/// `spans`: the text without code spans; a run of backticks opens one and the next run of the same
/// length closes it; a run that no other closes is text.
fn spans(s: &[u8]) -> Vec<u8> {
    let run = |i: usize| s[i..].iter().take_while(|&&b| b == b'`').count();
    let mut out = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if s[i] != b'`' {
            out.push(s[i]);
            i += 1;
            continue;
        }
        let n = run(i);
        let mut j = i + n;
        let mut closed = None;
        while j < s.len() {
            let m = if s[j] == b'`' { run(j) } else { 1 };
            if m == n && s[j] == b'`' {
                closed = Some(j);
                break;
            }
            j += m;
        }
        match closed {
            Some(j) => {
                out.extend(breaks(&s[i..j + n]));
                i = j + n;
            }
            None => {
                out.extend_from_slice(&s[i..i + n]);
                i += n;
            }
        }
    }
    out
}

/// Each link target `](...)` left as `]`: one that holds no `)`, and from `HELD` no line break.
fn targets_out(s: &[u8], newline_ends: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == b']' && s.get(i + 1) == Some(&b'(') {
            let rest = &s[i + 2..];
            let end = rest
                .iter()
                .position(|&b| b == b')' || (newline_ends && b == b'\n'));
            if let Some(end) = end.filter(|&end| rest[end] == b')') {
                out.push(b']');
                i += 2 + end + 1;
                continue;
            }
        }
        out.push(s[i]);
        i += 1;
    }
    out
}

/// Each `**` and `__` as one marker byte, left to right, as the bash's `gsub` swaps them.
fn mark_bold(s: &[u8]) -> Vec<u8> {
    // Two passes: `**` first, then `__` on what the first left.
    let mut first = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'*' && s.get(i + 1) == Some(&b'*') {
            first.push(1);
            i += 2;
        } else {
            first.push(s[i]);
            i += 1;
        }
    }
    let mut out = Vec::with_capacity(first.len());
    let mut i = 0;
    while i < first.len() {
        if first[i] == b'_' && first.get(i + 1) == Some(&b'_') {
            out.push(2);
            i += 2;
        } else {
            out.push(first[i]);
            i += 1;
        }
    }
    out
}

/// The markers of `mark_bold` back as `**` and `__`, and every such byte the text held besides.
fn unmark_bold(s: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    for b in s {
        match b {
            1 => out.extend_from_slice(b"**"),
            2 => out.extend_from_slice(b"__"),
            b => out.push(b),
        }
    }
    out
}

/// `plain` from `HELD`: the text without code spans, link targets, and italics, each italic a space
/// at either end with its line breaks kept. A star or an underscore opens an italic before a
/// non-space where a closer follows; an underscore inside a word does not. Bold is no italic.
fn plain(s: &[u8]) -> Vec<u8> {
    let s = mark_bold(&targets_out(&spans(s), true));
    let mut out = Vec::with_capacity(s.len());
    let mut prev = b' ';
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        let opens = (c == b'*' || (c == b'_' && !prev.is_ascii_alphanumeric()))
            && s.get(i + 1).is_some_and(|&b| !is_space(b));
        if let Some(j) = opens.then(|| closer(&s, i, c)).flatten() {
            out.push(b' ');
            out.extend(breaks(&s[i..=j]));
            out.push(b' ');
            i = j;
        } else {
            out.push(c);
        }
        prev = c;
        i += 1;
    }
    unmark_bold(out)
}

/// `closer`: where the italic the byte at i opens closes, the same byte after a non-space, an
/// underscore not before a letter or digit.
fn closer(s: &[u8], i: usize, c: u8) -> Option<usize> {
    (i + 2..s.len()).find(|&j| {
        s[j] == c
            && !is_space(s[j - 1])
            && (c == b'*' || !s.get(j + 1).is_some_and(u8::is_ascii_alphanumeric))
    })
}

/// The reader of `rule_stream` of v0.8.0: each line of prose read alone, an italic carried to the
/// next line of its paragraph, fenced code from a marker line to a line that holds the marker.
fn read_rules_first(file: &RuleFile) -> Vec<Row> {
    let adr = file.path != b"AGENTS.md";
    let mut rows = Vec::new();
    let mut fence: Vec<u8> = Vec::new();
    let mut ital: Option<u8> = None;
    let mut gap = false;
    let mut inside = false;
    let para = |rows: &mut Vec<Row>, gap: &mut bool, ital: &mut Option<u8>| {
        if !*gap {
            rows.push(Row::Para);
        }
        *gap = true;
        *ital = None;
    };
    for (i, line) in awk_lines(&file.text).into_iter().enumerate() {
        let t = after_space(line);
        let marker = t.starts_with(b"```") || t.starts_with(b"~~~");
        if fence.is_empty() && marker {
            para(&mut rows, &mut gap, &mut ital);
            fence = t[..3].to_vec();
            continue;
        }
        if !fence.is_empty() && contains(line, &fence) && marker {
            fence.clear();
            continue;
        }
        if !fence.is_empty() {
            continue;
        }
        if line.starts_with(b"## ") {
            para(&mut rows, &mut gap, &mut ital);
            if adr {
                inside = line == b"## Decision" || line == b"## Enforcement";
            }
            continue;
        }
        if adr && !inside {
            continue;
        }
        if is_break(line) {
            para(&mut rows, &mut gap, &mut ital);
            continue;
        }
        if is_item(line) {
            para(&mut rows, &mut gap, &mut ital);
        }
        let text = targets_out(&single_spans_out(line), false);
        let nr = i + 1;
        rows.push(Row::Line {
            nr,
            added: file.added.contains(&nr),
            text: plain_first(&text, &mut ital),
        });
        gap = false;
    }
    rows
}

/// Each code span of single backticks taken out, `` `[^`]*` ``, nothing left in its place.
fn single_spans_out(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'`'
            && let Some(end) = s[i + 1..].iter().position(|&b| b == b'`')
        {
            i += end + 2;
            continue;
        }
        out.push(s[i]);
        i += 1;
    }
    out
}

/// `plain` of v0.8.0: a star or an underscore opens an italic before a non-space and closes it
/// after one; what an italic holds is dropped, each of its ends a space; the italic carries on.
fn plain_first(s: &[u8], ital: &mut Option<u8>) -> Vec<u8> {
    let s = mark_bold(s);
    let mut out = Vec::with_capacity(s.len());
    let mut prev = b' ';
    for (i, &c) in s.iter().enumerate() {
        let next = s.get(i + 1).copied();
        if ital.is_none()
            && (c == b'*' || (c == b'_' && !prev.is_ascii_alphanumeric()))
            && next.is_some_and(|b| !is_space(b))
        {
            *ital = Some(c);
            out.push(b' ');
        } else if *ital == Some(c)
            && !is_space(prev)
            && (c == b'*' || !next.is_some_and(|b| b.is_ascii_alphanumeric()))
        {
            *ital = None;
            out.push(b' ');
        } else if ital.is_none() {
            out.push(c);
        }
        prev = c;
    }
    unmark_bold(out)
}

/// The rows of `avoid_terms` of the release, as its awk prints them: from `REQUIRE`
/// `A<TAB>word<TAB>term<TAB>qualifier`, `T<TAB>term`, and `U<TAB>line<TAB>item<TAB>term`; before
/// it `word<TAB>term`.
fn avoid_terms(glossary: &[u8], release: Version) -> Vec<u8> {
    let mut out = Vec::new();
    let mut term: Vec<u8> = Vec::new();
    let mut entry = false;
    let mut open = false;
    let mut list: Vec<u8> = Vec::new();
    let mut from = 0;
    let named = release >= REQUIRE;
    let done = |out: &mut Vec<u8>, open: &mut bool, list: &[u8], from: usize, term: &[u8]| {
        if *open {
            take(out, list, from, term);
        }
        *open = false;
    };
    for (i, line) in awk_lines(glossary).into_iter().enumerate() {
        let opens_entry = entry_term(line);
        if let Some(t) = opens_entry {
            if named {
                done(&mut out, &mut open, &list, from, &term);
            } else {
                open = false;
            }
            term = t.to_vec();
            entry = true;
            if named {
                out.extend_from_slice(b"T\t");
                out.extend(term.iter().map(u8::to_ascii_lowercase));
                out.push(b'\n');
            }
        }
        let indented = line.first().is_some_and(|&b| is_space(b))
            && line.iter().find(|&&b| !is_space(b)).is_some();
        if !line.starts_with(b"- **") && !indented {
            if named {
                done(&mut out, &mut open, &list, from, &term);
            }
            open = false;
            entry = false;
            term.clear();
        }
        if release >= HELD && line.first().is_some_and(|&b| is_space(b)) && is_item(line) {
            if named {
                done(&mut out, &mut open, &list, from, &term);
            }
            open = false;
        }
        let at = find(line, b"_Avoid:_");
        let continues = line.first().is_some_and(|&b| is_space(b))
            && line
                .iter()
                .find(|&&b| !is_space(b))
                .is_some_and(|&b| b != b'-');
        if entry && open && at.is_none() && continues {
            if named {
                list.push(3);
                list.extend_from_slice(line);
            } else {
                take_first(&mut out, line, &term);
            }
            continue;
        }
        if let Some(at) = at.filter(|_| entry && !term.is_empty()) {
            let rest = &line[at + 8..];
            if named {
                done(&mut out, &mut open, &list, from, &term);
                open = true;
                from = i + 1;
                list = rest.to_vec();
            } else {
                open = true;
                take_first(&mut out, rest, &term);
            }
        }
    }
    if named {
        done(&mut out, &mut open, &list, from, &term);
    }
    out
}

/// The term of a line `^- \*\*[^*]+\*\*`: what stands between `- **` and the next `**`.
fn entry_term(line: &[u8]) -> Option<&[u8]> {
    let rest = line.strip_prefix(b"- **")?;
    let end = rest.iter().position(|&b| b == b'*')?;
    (end > 0 && rest.get(end + 1) == Some(&b'*')).then(|| &rest[..end])
}

/// `take` of v0.8.x: each item of the list, split at commas and semicolons, in lower case, without
/// a leading `and` and the white space and periods at its ends; one with an angle bracket dropped.
fn take_first(out: &mut Vec<u8>, list: &[u8], term: &[u8]) {
    for part in list.split(|&b| b == b',' || b == b';') {
        let w: Vec<u8> = part.iter().map(u8::to_ascii_lowercase).collect();
        let mut s = &w[..];
        let lead = s.iter().take_while(|&&b| is_space(b)).count();
        s = &s[lead..];
        if s.starts_with(b"and") && s.get(3).is_some_and(|&b| is_space(b)) {
            let n = s[3..].iter().take_while(|&&b| is_space(b)).count();
            s = &s[3 + n..];
        }
        let tail = s
            .iter()
            .rev()
            .take_while(|&&b| b == b'.' || is_space(b))
            .count();
        s = &s[..s.len() - tail];
        if s.is_empty() || s.contains(&b'<') || s.contains(&b'>') {
            continue;
        }
        out.extend_from_slice(s);
        out.push(b'\t');
        out.extend_from_slice(term);
        out.push(b'\n');
    }
}

/// `take` from `REQUIRE`: the items of a list that starts on the line, each `\003` where a line of
/// it breaks.
fn take(out: &mut Vec<u8>, list: &[u8], mut line: usize, term: &[u8]) {
    for part in list.split(|&b| b == b',' || b == b';') {
        let mut w = [b" ", part, b" "].concat();
        let lead = w.iter().take_while(|&&b| is_space(b) || b == 3).count();
        let at = line + w[..lead].iter().filter(|&&b| b == 3).count();
        line += w.iter().filter(|&&b| b == 3).count();
        w.retain(|&b| b != 3 && b != b'*' && b != b'`');
        // An underscore beside a byte that is no letter or digit is dropped, one at a time from
        // the left, as the loop of `match` and `substr` drops it.
        while let Some(p) = (0..w.len().saturating_sub(1)).find(|&p| {
            (!w[p].is_ascii_alphanumeric() && w[p + 1] == b'_')
                || (w[p] == b'_' && !w[p + 1].is_ascii_alphanumeric())
        }) {
            w.remove(if w[p] == b'_' { p } else { p + 1 });
        }
        let mut squeezed = Vec::with_capacity(w.len());
        for b in w.iter().map(u8::to_ascii_lowercase) {
            if is_space(b) {
                if squeezed.last() != Some(&b' ') {
                    squeezed.push(b' ');
                }
            } else {
                squeezed.push(b);
            }
        }
        let mut w = trim_item(&squeezed);
        if w.is_empty() || w.contains(&b'<') || w.contains(&b'>') {
            continue;
        }
        let mut qualifier: &[u8] = b"";
        if let Some(word) = w.strip_suffix(b" alone") {
            qualifier = b"alone";
            w = word.to_vec();
        } else if let Some(word) = w.strip_suffix(b" in prose") {
            w = word.to_vec();
        }
        let words = w.split(|&b| b == b' ').filter(|s| !s.is_empty()).count();
        let odd = b"[]!\"#$%&()*+,/:;<=>?@\\^_`{|}~";
        let unread = words > 3
            || matches!(w.first(), Some(b'-' | b'.' | b'\''))
            || w.iter().any(|b| odd.contains(b));
        if unread {
            out.extend_from_slice(format!("U\t{at}\t").as_bytes());
            out.extend_from_slice(&w);
            if !qualifier.is_empty() {
                out.push(b' ');
                out.extend_from_slice(qualifier);
            }
            out.push(b'\t');
            out.extend_from_slice(term);
        } else {
            out.extend_from_slice(b"A\t");
            out.extend_from_slice(&w);
            out.push(b'\t');
            out.extend_from_slice(term);
            out.push(b'\t');
            out.extend_from_slice(qualifier);
        }
        out.push(b'\n');
    }
}

/// `gsub(/^ (and )?| ?[.]? ?$/, "", w)` on an item whose white space is single spaces.
fn trim_item(w: &[u8]) -> Vec<u8> {
    let tail = |s: &[u8]| {
        let mut s = s;
        s = s.strip_prefix(b" ").unwrap_or(s);
        s = s.strip_prefix(b".").unwrap_or(s);
        s = s.strip_prefix(b" ").unwrap_or(s);
        s.is_empty()
    };
    // At the start the longer of the two matches wins, and the tail matches the whole only where
    // the item is no more than such a tail.
    if tail(w) {
        return Vec::new();
    }
    let head = if w.starts_with(b" and ") {
        5
    } else {
        usize::from(w.starts_with(b" "))
    };
    let rest = &w[head..];
    let cut = (0..=rest.len())
        .find(|&p| tail(&rest[p..]))
        .unwrap_or(rest.len());
    rest[..cut].to_vec()
}

/// The sensor's awk: the findings of the rows, each with whether `--require` fails on it.
struct Findings {
    release: Version,
    words: Vec<(Vec<u8>, Vec<u8>)>,
    terms: Vec<Vec<u8>>,
    unread: Vec<Vec<u8>>,
    categories: Vec<String>,
    out: Vec<(bool, Vec<u8>)>,
}

/// The sentence being counted: its words, the line it starts on, and the lines it holds with
/// whether the change adds each.
#[derive(Default)]
struct Sentence {
    count: usize,
    start: usize,
    seen: BTreeMap<usize, bool>,
}

impl Findings {
    fn new(release: Version, avoid: &[u8], categories: Vec<String>) -> Self {
        let mut words = Vec::new();
        let mut terms = Vec::new();
        let mut unread = Vec::new();
        if release >= REQUIRE {
            for row in avoid.split(|&b| b == b'\n') {
                let kv: Vec<&[u8]> = row.split(|&b| b == b'\t').collect();
                let field = |i: usize| kv.get(i).copied().unwrap_or_default().to_vec();
                match kv[0] {
                    b"A" => words.push((field(1), field(2))),
                    b"T" => terms.push(field(1)),
                    b"U" => {
                        let mut line = b"docs/GLOSSARY.md:".to_vec();
                        line.extend_from_slice(&field(1));
                        line.extend_from_slice(b": cannot read \"");
                        line.extend_from_slice(&field(2));
                        line.extend_from_slice(b"\" under _Avoid:_ of \"");
                        line.extend_from_slice(&field(3));
                        line.push(b'"');
                        if release >= KEYED {
                            line.extend_from_slice(b" [rules/avoid-list:unread]");
                        }
                        unread.push(line);
                    }
                    _ => {}
                }
            }
        } else {
            // The rows reach awk through `-v`, their trailing line breaks cut by the command
            // substitution first.
            let kept = avoid.len() - avoid.iter().rev().take_while(|&&b| b == b'\n').count();
            let avoid = adr::awk_unescape(&avoid[..kept]);
            if !avoid.is_empty() {
                for row in avoid.split(|&b| b == b'\n') {
                    let kv: Vec<&[u8]> = row.split(|&b| b == b'\t').collect();
                    let field = |i: usize| kv.get(i).copied().unwrap_or_default().to_vec();
                    words.push((field(0), field(1)));
                }
            }
        }
        Self {
            release,
            words,
            terms,
            unread,
            categories,
            out: Vec::new(),
        }
    }

    fn run(mut self, rows: &[Row]) -> Vec<(bool, Vec<u8>)> {
        self.out = self.unread.drain(..).map(|line| (false, line)).collect();
        let mut file: Vec<u8> = Vec::new();
        let mut fails = false;
        let mut sentence = Sentence::default();
        let mut prev: Vec<u8> = Vec::new();
        let mut prev_nr = 0;
        let mut prev_added = false;
        let mut told: BTreeSet<(usize, usize)> = BTreeSet::new();
        for row in rows {
            match row {
                Row::File(path) => {
                    self.flush(&mut sentence, &file);
                    file.clone_from(path);
                    prev.clear();
                    told.clear();
                    fails = self.release >= REQUIRE
                        && file.starts_with(b"docs/adr/")
                        && !file[9..].is_empty()
                        && !file[9..].contains(&b'/')
                        && !file.starts_with(b"docs/adr/NUC-");
                }
                Row::Para => {
                    self.flush(&mut sentence, &file);
                    prev.clear();
                }
                Row::Line { nr, added, text } => {
                    let text = text.split(|&b| b == b'\t').next().unwrap_or_default();
                    if self.release >= HELD {
                        self.line(
                            &file,
                            fails,
                            (*nr, *added, text),
                            (&mut prev, &mut prev_nr, &mut prev_added),
                            &mut told,
                        );
                    } else {
                        self.line_first(&file, *nr, *added, text);
                    }
                    self.count(&mut sentence, &file, *nr, *added, text);
                }
            }
        }
        self.flush(&mut sentence, &file);
        self.out
    }

    /// The soft modal verbs and avoided words of a line from `HELD`.
    fn line(
        &mut self,
        file: &[u8],
        fails: bool,
        (nr, added, text): (usize, bool, &[u8]),
        (prev, prev_nr, prev_added): (&mut Vec<u8>, &mut usize, &mut bool),
        told: &mut BTreeSet<(usize, usize)>,
    ) {
        let prose = if self.release >= TOKENS {
            unref(text, &self.categories)
        } else {
            text.to_vec()
        };
        let low = squash(&prose);
        if !prev.is_empty() && (added || *prev_added) {
            let joined = [&prev[..], b" ", &low[1..]].concat();
            let at = prev.len() + 1;
            for i in 0..self.words.len() {
                if found(&self.bare(&joined, i), &self.words[i].0, at).is_some() {
                    let on = if *prev_added { *prev_nr } else { nr };
                    self.avoided(file, fails, on, i, told);
                }
            }
        }
        let lowered: Vec<u8> = prose.iter().map(u8::to_ascii_lowercase).collect();
        let lead = lowered.iter().take_while(|&&b| is_space(b)).count();
        *prev = [b" ", &lowered[lead..]].concat();
        *prev_nr = nr;
        *prev_added = added;
        if !added {
            return;
        }
        for verb in SOFT {
            if bounded(&low, verb) {
                let mut line = self.at(file, nr);
                line.extend_from_slice(b": the soft modal verb \"");
                line.extend_from_slice(verb);
                line.push(b'"');
                if self.release >= KEYED {
                    line.extend_from_slice(b" [rules/soft-modal:verb]");
                }
                self.out.push((fails, line));
            }
        }
        for i in 0..self.words.len() {
            if found(&self.bare(&low, i), &self.words[i].0, 0).is_some() {
                self.avoided(file, fails, nr, i, told);
            }
        }
    }

    /// The soft modal verbs and avoided words of a line of v0.8.0, each line read alone.
    fn line_first(&mut self, file: &[u8], nr: usize, added: bool, text: &[u8]) {
        if !added {
            return;
        }
        let low: Vec<u8> = text.iter().map(u8::to_ascii_lowercase).collect();
        for verb in SOFT {
            // `(^|[^a-z])verb([^a-z]|$)`
            let edge = |b: Option<&u8>| !b.is_some_and(u8::is_ascii_lowercase);
            let hit = (0..low.len()).any(|at| {
                low[at..].starts_with(verb)
                    && edge(at.checked_sub(1).and_then(|p| low.get(p)))
                    && edge(low.get(at + verb.len()))
            });
            if hit {
                let mut line = self.at(file, nr);
                line.extend_from_slice(b": the soft modal verb \"");
                line.extend_from_slice(verb);
                line.push(b'"');
                self.out.push((false, line));
            }
        }
        let low = [b" ", &low[..], b" "].concat();
        for i in 0..self.words.len() {
            if found(&low, &self.words[i].0, 0).is_some() {
                let line = self.avoided_line(file, nr, i);
                self.out.push((false, line));
            }
        }
    }

    fn at(&self, file: &[u8], nr: usize) -> Vec<u8> {
        let mut line = file.to_vec();
        line.extend_from_slice(format!(":{nr}").as_bytes());
        line
    }

    fn avoided_line(&self, file: &[u8], nr: usize, i: usize) -> Vec<u8> {
        let (word, term) = &self.words[i];
        let mut line = self.at(file, nr);
        line.extend_from_slice(b": \"");
        line.extend_from_slice(word);
        line.extend_from_slice(
            "\" is listed under _Avoid:_ in docs/GLOSSARY.md — write \"".as_bytes(),
        );
        line.extend_from_slice(term);
        line.push(b'"');
        if self.release >= KEYED {
            line.extend_from_slice(b" [rules/avoided-term:listed]");
        }
        line
    }

    fn avoided(
        &mut self,
        file: &[u8],
        fails: bool,
        nr: usize,
        i: usize,
        told: &mut BTreeSet<(usize, usize)>,
    ) {
        if told.insert((nr, i)) {
            let line = self.avoided_line(file, nr, i);
            self.out.push((fails, line));
        }
    }

    /// `bare`: the text with each longer term of the glossary that holds the word of item i
    /// blanked, from `REQUIRE`.
    fn bare(&self, s: &[u8], i: usize) -> Vec<u8> {
        let mut s = s.to_vec();
        let word = &self.words[i].0;
        for t in &self.terms {
            if t == word || found(&[b" ", &t[..], b" "].concat(), word, 0).is_none() {
                continue;
            }
            while let Some(at) = found(&s, t, 0) {
                s[at..at + t.len()].fill(b' ');
            }
        }
        s
    }

    /// The words of a line counted into the sentence, which a stop ends.
    fn count(&mut self, sentence: &mut Sentence, file: &[u8], nr: usize, added: bool, text: &[u8]) {
        for token in space_split(text) {
            if !token.iter().any(u8::is_ascii_alphanumeric) {
                let stop = |b: &u8| matches!(b, b'.' | b'!' | b'?');
                let ends = if self.release >= KEYED {
                    token.iter().any(stop)
                } else {
                    token.first().is_some_and(stop)
                };
                if sentence.count > 0 && ends {
                    self.flush(sentence, file);
                }
                continue;
            }
            if sentence.count == 0 {
                sentence.start = nr;
            }
            sentence.count += 1;
            if !sentence.seen.contains_key(&nr) || added {
                sentence.seen.insert(nr, added);
            }
            let closers = token
                .iter()
                .rev()
                .take_while(|b| matches!(b, b')' | b'*' | b'_' | b'"'))
                .count();
            if token[..token.len() - closers]
                .last()
                .is_some_and(|b| matches!(b, b'.' | b'!' | b'?'))
            {
                self.flush(sentence, file);
            }
        }
    }

    fn flush(&mut self, sentence: &mut Sentence, file: &[u8]) {
        if sentence.count > MAX_WORDS && sentence.seen.values().any(|&added| added) {
            let mut line = self.at(file, sentence.start);
            line.extend_from_slice(format!(": a sentence of {} words", sentence.count).as_bytes());
            if self.release >= KEYED {
                line.extend_from_slice(b" [rules/long-sentence:words]");
            }
            self.out.push((false, line));
        }
        *sentence = Sentence::default();
    }
}

/// The tokens of `split(text, words, /[[:space:]]+/)`: an empty one before a leading run of space
/// and after a trailing one, none for an empty text.
fn space_split(text: &[u8]) -> Vec<&[u8]> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < text.len() {
        if is_space(text[i]) {
            tokens.push(&text[start..i]);
            while i < text.len() && is_space(text[i]) {
                i += 1;
            }
            start = i;
        } else {
            i += 1;
        }
    }
    tokens.push(&text[start..]);
    tokens
}

/// `squash`: the text in lower case without the white space at its ends, one space at either end.
fn squash(text: &[u8]) -> Vec<u8> {
    let low: Vec<u8> = text.iter().map(u8::to_ascii_lowercase).collect();
    let start = low.iter().take_while(|&&b| is_space(b)).count();
    let end = low.len()
        - low[start..]
            .iter()
            .rev()
            .take_while(|&&b| is_space(b))
            .count();
    [b" ", &low[start..end], b" "].concat()
}

/// Whether the byte joins a word for `found`: `[a-z0-9_]`.
fn joins(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'
}

/// Whether the verb stands in the text between two bytes that join no word, `[^a-z0-9_]`.
fn bounded(text: &[u8], verb: &[u8]) -> bool {
    (1..text.len()).any(|at| {
        text[at..].starts_with(verb)
            && !joins(text[at - 1])
            && text.get(at + verb.len()).is_some_and(|&b| !joins(b))
    })
}

/// `found`: where the word stands whole in the text, by its index from 1 as awk counts; with a
/// break, only where it spans that position.
fn found(s: &[u8], w: &[u8], b: usize) -> Option<usize> {
    if w.is_empty() {
        return None;
    }
    let mut off = 0;
    while let Some(at) = find(&s[off..], w) {
        let at = at + off + 1;
        // mawk reads `substr(s, 0, 1)` as the first byte of s.
        let before = s[at.saturating_sub(2)];
        let after = s.get(at - 1 + w.len()).copied();
        if !joins(before) && !after.is_some_and(joins) && (b == 0 || (at < b && at + w.len() > b)) {
            return Some(at - 1);
        }
        off = at;
    }
    None
}

/// `unref`: the text with each ADR identifier of a known category blanked where it stands as a
/// word of its own and its slug has two to five parts.
fn unref(s: &[u8], categories: &[String]) -> Vec<u8> {
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'-';
    let mut out: Vec<u8> = Vec::with_capacity(s.len());
    let mut rest = s;
    while let Some((start, len)) = leftmost_reference(rest, categories) {
        let id = &rest[start..start + len];
        let parts = id.iter().filter(|&&b| b == b'-').count();
        let before = if start > 0 {
            Some(rest[start - 1])
        } else {
            out.last().copied()
        };
        let after = rest.get(start + len).copied();
        out.extend_from_slice(&rest[..start]);
        if !before.is_some_and(word) && !after.is_some_and(word) && (2..=5).contains(&parts) {
            out.extend(std::iter::repeat_n(b' ', len));
        } else {
            out.extend_from_slice(id);
        }
        rest = &rest[start + len..];
    }
    out.extend_from_slice(rest);
    out
}

/// The leftmost longest match of `(<categories>)-[a-z0-9]+(-[a-z0-9]+)+`: its start and length.
fn leftmost_reference(s: &[u8], categories: &[String]) -> Option<(usize, usize)> {
    let part = |b: &u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    (0..s.len()).find_map(|start| {
        categories
            .iter()
            .filter_map(|category| {
                let rest = s[start..].strip_prefix(category.as_bytes())?;
                let rest = rest.strip_prefix(b"-")?;
                let mut len = rest.iter().take_while(|b| part(b)).count();
                if len == 0 {
                    return None;
                }
                let mut parts = 1;
                while rest.get(len) == Some(&b'-') && rest.get(len + 1).is_some_and(part) {
                    len += 1 + rest[len + 1..].iter().take_while(|b| part(b)).count();
                    parts += 1;
                }
                (parts >= 2).then_some(category.len() + 1 + len)
            })
            .max()
            .map(|len| (start, len))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const V080: Version = Version(0, 8, 0);
    const NEWEST: Version = Version(0, 12, 0);

    fn file(path: &str, text: &str, added: &[usize]) -> RuleFile {
        RuleFile {
            path: path.as_bytes().to_vec(),
            text: text.as_bytes().to_vec(),
            added: added.iter().copied().collect(),
        }
    }

    fn lines(report: &[u8]) -> Vec<String> {
        String::from_utf8_lossy(report)
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn the_files_of_rules_are_named_as_adr_is_file_of_the_release_names_them() {
        for (path, before, after) in [
            ("AGENTS.md", true, true),
            ("docs/adr/0001-x.md", true, false),
            ("docs/adr/draft-x.md", true, true),
            ("docs/adr/NUC-x.md", false, true),
            ("docs/adr/A-.md", false, true),
            ("docs/adr/A1-x.md", false, false),
            ("docs/adr/README.md", false, false),
            ("docs/adr/sub/NUC-x.md", false, false),
            ("docs/AGENTS.md", false, false),
        ] {
            assert_eq!(is_rule_file(path.as_bytes(), V080), before, "{path}");
            assert_eq!(is_rule_file(path.as_bytes(), REQUIRE), after, "{path}");
        }
    }

    #[test]
    fn the_listing_pairs_a_rename_with_its_source_and_sorts_the_lines_as_sort_does() {
        let diff = b"M\tdocs/adr/b.md\nR087\tdocs/adr/old.md\tdocs/adr/a.md\nA\tAGENTS.md\n";
        let got = listed(diff, b"docs/adr/a.md\n", REQUIRE);
        let shown: Vec<(String, String)> = got
            .iter()
            .map(|(p, f)| (adr::show(p), adr::show(f)))
            .collect();
        assert_eq!(
            shown,
            [
                ("AGENTS.md".into(), String::new()),
                ("docs/adr/a.md".into(), String::new()),
                ("docs/adr/a.md".into(), "docs/adr/old.md".into()),
                ("docs/adr/b.md".into(), String::new()),
            ]
        );
        let got = listed(b"docs/adr/0001-x.md\n", b"", V080);
        assert_eq!(got, [(b"docs/adr/0001-x.md".to_vec(), Vec::new())]);
    }

    #[test]
    fn the_new_sides_and_the_drivers_read_as_git_writes_them() {
        let sides = new_sides(b"M\ta\nA\t\"z\\\"q\\\\\\tr\\303\\251.bin\"\nR090\tx\ty\n");
        assert!(sides.sources);
        assert_eq!(
            sides.added,
            [b"z\"q\\\tr\xc3\xa9.bin".to_vec(), b"y".to_vec()]
        );
        assert!(!new_sides(b"A\ta\nM\tb\n").sources);
        assert_eq!(unquoted(b"plain\\name"), b"plain\\name");
        let drivers = filter_drivers(b"filter.lfs.clean\ngit-lfs clean\0filter.a.b.process\nx\0filter.off.clean\n\0filter.req.required\ntrue\0");
        let names = |set: &BTreeSet<Vec<u8>>| set.iter().map(|n| adr::show(n)).collect::<Vec<_>>();
        assert_eq!(names(&drivers), ["a.b", "lfs"]);
    }

    #[test]
    fn the_added_lines_are_the_new_side_of_each_hunk() {
        let diff =
            b"diff --git a/x b/x\n@@ -1,2 +3,2 @@ ctx\n+a\n@@ -9 +12 @@\n@@ -20,3 +30,0 @@\n";
        assert_eq!(added_lines(diff).unwrap(), BTreeSet::from([3, 4, 12]));
        assert_eq!(every_line(b"a\nb"), BTreeSet::from([1, 2]));
    }

    #[test]
    fn a_hunk_head_awk_reads_inexactly_is_not_decided() {
        for diff in [
            &b"@@ -1 +99999999999999999999 @@\n"[..],
            b"@@ -1 +9007199254740993 @@\n",
            b"@@ -1 +9007199254740992,5 @@\n",
        ] {
            let refusal = added_lines(diff).unwrap_err();
            assert!(
                refusal.to_string().ends_with("[sense/plain-rules:hunk]"),
                "{refusal}"
            );
        }
        assert_eq!(
            added_lines(b"@@ -1 +9007199254740990,2 @@\n").unwrap(),
            BTreeSet::from([9007199254740990, 9007199254740991])
        );
    }

    #[test]
    fn the_glossary_reads_items_as_avoid_terms_of_each_release_reads_them() {
        let glossary = "# G\n\n- **Node shape** — a form.\n  _Avoid:_ *shape* alone, self-edge in prose,\n  include\n  file, Größe.\n- **Pipeline** — what runs.\n  _Avoid:_ `pin`, ci/cd, -flag, .dot, 'quoted, and card; <x>.\n";
        let rows = String::from_utf8(avoid_terms(glossary.as_bytes(), NEWEST)).unwrap();
        assert_eq!(
            rows,
            "T\tnode shape\nA\tshape\tNode shape\talone\nA\tself-edge\tNode shape\t\n\
             A\tinclude file\tNode shape\t\nA\tgröße\tNode shape\t\nT\tpipeline\n\
             A\tpin\tPipeline\t\nU\t8\tci/cd\tPipeline\nU\t8\t-flag\tPipeline\n\
             U\t8\t.dot\tPipeline\nU\t8\t'quoted\tPipeline\nA\tcard\tPipeline\t\n"
        );
        let rows = String::from_utf8(avoid_terms(glossary.as_bytes(), V080)).unwrap();
        assert!(rows.starts_with("*shape* alone\tNode shape\n"), "{rows}");
        assert!(
            rows.contains("include\tNode shape\nfile\tNode shape\n"),
            "{rows}"
        );
    }

    #[test]
    fn an_identifier_of_a_known_category_standing_alone_is_blanked() {
        let categories: Vec<String> = ["NUC", "UI"].map(String::from).to_vec();
        // `NUC:` stands for `NUC-`, so that no reader of references reads one here.
        let id = |s: &str| s.replace("NUC:", "NUC-");
        let blank = |s: &str| String::from_utf8(unref(id(s).as_bytes(), &categories)).unwrap();
        assert_eq!(blank("see NUC:a-b."), format!("see {}.", " ".repeat(7)));
        assert_eq!(blank("xNUC:a-b"), id("xNUC:a-b"));
        assert_eq!(blank("NUC:a"), id("NUC:a"));
        assert_eq!(blank("UI-a-b-c-d-e-f"), "UI-a-b-c-d-e-f");
        // An identifier right after one that is no word of its own stays words.
        assert_eq!(blank("NUC:a-bNUC:c-d"), id("NUC:a-bNUC:c-d"));
        assert_eq!(blank("FIX-a-b"), "FIX-a-b");
    }

    #[test]
    fn require_fails_on_a_project_adr_and_marks_each_finding_as_its_release_does() {
        let adr = "# X\n\n## Decision\n\nIt should run.\n";
        let files = [
            file("docs/adr/FIX-x.md", adr, &[5]),
            file("docs/adr/NUC-x.md", adr, &[5]),
        ];
        let outcome = report(NEWEST, "a..b", &files, b"", b"", true);
        assert!(outcome.failed);
        assert_eq!(
            lines(&outcome.report)[1..],
            [
                "  - docs/adr/FIX-x.md:5: the soft modal verb \"should\" (fails) [rules/soft-modal:verb]",
                "  - docs/adr/NUC-x.md:5: the soft modal verb \"should\" [rules/soft-modal:verb]",
                "Plain rules sensor: 1 of them fail: a soft modal verb or an avoided word in a project ADR (NUC-documents-are-fixed §5).",
            ]
        );
        let outcome = report(REQUIRE, "a..b", &files[..1], b"", b"", true);
        assert_eq!(
            lines(&outcome.report)[1],
            "  - docs/adr/FIX-x.md:5: the soft modal verb \"should\" (fails)"
        );
        let outcome = report(NEWEST, "a..b", &files, b"", b"", false);
        assert!(!outcome.failed);
        assert_eq!(lines(&outcome.report).len(), 3);
    }

    #[test]
    fn the_first_reader_carries_an_italic_and_a_span_does_not_wrap() {
        let text = "# X\n\n## Decision\n\nSay *must\nshould* and `a\nmight` b.\n";
        let line = |nr, text: &str| Row::Line {
            nr,
            added: true,
            text: text.as_bytes().to_vec(),
        };
        // v0.8.0 reads each line alone: the italic carries on, the code span does not close.
        let rows = read_rules(V080, &file("docs/adr/0001-x.md", text, &[5, 6, 7]));
        assert_eq!(
            rows[1..],
            [line(5, "Say  "), line(6, "  and `a"), line(7, "might` b.")]
        );
        // From v0.8.1 the paragraph is held: the span closes on the next line, which it blanks.
        let rows = read_rules(HELD, &file("docs/adr/0001-x.md", text, &[5, 6, 7]));
        assert_eq!(
            rows[1..],
            [line(5, "Say  "), line(6, "  and "), line(7, " b.")]
        );
    }

    #[test]
    fn a_long_sentence_counts_across_lines_and_ends_at_a_stop() {
        let long = "one two three four five six seven eight nine ten eleven twelve thirteen";
        let text = format!("# Agent Guide\n\n{long}\n{long}.\nShort.\n");
        let outcome = report(
            NEWEST,
            "a..b",
            &[file("AGENTS.md", &text, &[4])],
            b"",
            b"",
            false,
        );
        assert_eq!(
            lines(&outcome.report)[1],
            "  - AGENTS.md:3: a sentence of 26 words [rules/long-sentence:words]"
        );
    }
}
