//! `nuc sense binding-adrs`: the ADRs in force whose *Applies to* names a path the change touches,
//! or a path named with `--path`, and with `--sections` what a session reads of the record, as the
//! template's `scripts/sensor-binding-adrs.sh` of the tree's release reports them. The shell
//! resolves the change, lists its paths, and reads the ADRs as the change has them; the sensor
//! reads them as the bash's grep, awk, sed, and `[[ ]]` do, under `LC_ALL=C`.

use std::collections::{BTreeMap, BTreeSet};

use regex::bytes::Regex;

use super::{Refusal, Sensor};
use crate::adr;
use crate::bash;
use crate::config::Version;

/// The expression, compiled once a run at the site that names it, as `src/adr.rs` keeps its own.
macro_rules! regex {
    ($pattern:literal) => {{
        static COMPILED: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        COMPILED.get_or_init(|| Regex::new($pattern).expect("a valid pattern"))
    }};
}

/// The sensor. The releases before v0.5.0 list no draft, so `nuc` reports from v0.5.0 on.
pub const SENSOR: Sensor = Sensor {
    name: "binding-adrs",
    since: Version(0, 5, 0),
    changes: &[MODULE_DIRS, OPTIONS, PROSE_SPLIT, FENCES, NAMED, RULE_KEYS],
    reasons: &[
        "ref",
        "git",
        "file",
        "name",
        "filter",
        "submodule",
        super::DRIVER,
        super::RELEASE,
    ],
};

/// The label that opens each line of the report.
pub const LABEL: &str = "Binding ADRs sensor";

/// The release from which a Rust file names the directory of its module's children.
pub const MODULE_DIRS: Version = Version(0, 6, 2);

/// The release that brings `--sections` and `--path`, and reads the prose of the record from the
/// index's *Binding* table.
pub const OPTIONS: Version = Version(0, 7, 0);

/// The release from which the split of an *Applies to* into items drops the newlines that end it,
/// as `<<< "$(sed …)"` drops them, where v0.7.0 hands sed's output on whole.
pub const PROSE_SPLIT: Version = Version(0, 8, 0);

/// The release from which a `## ` line inside a fence ends no section, as `ADR_FENCE_AWK` reads it.
pub const FENCES: Version = Version(0, 8, 1);

/// The release from which the record is named by category, an ADR is named by its file, the prose
/// is read from each ADR's file, and a file named with checks of its own binds only in them.
pub const NAMED: Version = Version(0, 9, 0);

/// The release from which a file binds by the rule keys of its checks, not their numbers, and an
/// argument the sensor does not read exits 2.
pub const RULE_KEYS: Version = Version(0, 10, 0);

/// The directory of the record.
pub const DIR: &str = "docs/adr";

/// The index, whose *Binding* table holds the prose before `NAMED`.
pub const INDEX: &str = "docs/adr/README.md";

/// What the arguments ask for, as the bash reads them.
#[derive(Debug, PartialEq, Eq)]
pub struct Args<'a> {
    pub sections: bool,
    /// The paths named with `--path`, a leading `./` cut; none where the change is a range.
    pub named: Vec<&'a str>,
    pub base: Option<&'a str>,
    pub head: Option<&'a str>,
}

/// The arguments as the bash of the release reads them, or `None` where `nuc sense` refuses them:
/// `--root` wherever it stands, `--path` without a path, and what [`super::positional`] refuses,
/// one rule for every sensor that takes a base and a head. Beside `--path` the sensor reads no
/// base or head: from `RULE_KEYS` one is refused, before it is ignored. Before `OPTIONS` the bash
/// reads `--sections` and `--path` as a base, which names no commit, and `nuc` refuses them as
/// arguments that read as an option.
pub fn parse<'a>(args: &[&'a str], release: Version) -> Option<Args<'a>> {
    let mut parsed = Args {
        sections: false,
        named: Vec::new(),
        base: None,
        head: None,
    };
    // `--root` is refused before the options are read, so that `--path --root` names no path, as
    // the bash of v0.10.0 on refuses it as the value of `--path`.
    if args.contains(&"--root") {
        return None;
    }
    let mut rest = args;
    if release >= OPTIONS {
        loop {
            match rest {
                ["--sections", tail @ ..] => {
                    parsed.sections = true;
                    rest = tail;
                }
                ["--path", path, tail @ ..] if !path.is_empty() => {
                    parsed.named.push(path.strip_prefix("./").unwrap_or(path));
                    rest = tail;
                }
                ["--path", ..] => return None,
                _ => break,
            }
        }
    }
    let most = if parsed.named.is_empty() { 2 } else { 0 };
    let positional = super::positional(rest, most, release >= RULE_KEYS)?;
    parsed.base = positional.first().copied();
    parsed.head = positional.get(1).copied();
    Some(parsed)
}

/// The paths the report reads.
#[derive(Debug)]
pub enum Listing {
    /// No base was given and there is no main to compare with.
    Nothing,
    /// The paths the change touches, as git lists them, sorted and unique; and its range.
    Changed { range: String, paths: Vec<Vec<u8>> },
    /// The paths named with `--path`, in the order given.
    Named(Vec<Vec<u8>>),
}

/// Whether the report reads the ADRs: where the change has a path to bind, and for `--sections`
/// from `NAMED` on, which reads the prose from the ADRs' files.
pub fn reads_adrs(listing: &Listing, sections: bool, release: Version) -> bool {
    let paths = match listing {
        Listing::Nothing => false,
        Listing::Changed { paths, .. } | Listing::Named(paths) => !paths.is_empty(),
    };
    paths || (sections && release >= NAMED)
}

/// Whether the report reads the index: for `--sections` before `NAMED`.
pub fn reads_index(sections: bool, release: Version) -> bool {
    sections && (OPTIONS..NAMED).contains(&release)
}

/// Whether the name is that of an ADR file of the release, as `adr_files` matches it.
pub fn is_adr_name(name: &str, release: Version) -> bool {
    if release >= NAMED {
        adr::is_category_name(name)
    } else {
        adr::is_adr_name(name)
    }
}

/// Of the lines `git ls-tree -r --name-only` lists under `docs/adr/` at a head, those
/// `ADR_FILE_RE` matches, as git quotes them: a name git quotes matches none.
pub fn listed_adrs(ls_tree: &[u8], release: Version) -> Vec<Vec<u8>> {
    let pattern = if release >= NAMED {
        regex!(r"(?-u)^docs/adr/[A-Za-z]+-[^/]*\.md$")
    } else {
        regex!(r"(?-u)^docs/adr/([0-9]{4}-[^/]*|draft-[^/]*)\.md$")
    };
    bash::sed_lines(bash::substituted(ls_tree))
        .filter(|line| pattern.is_match(line))
        .map(<[u8]>::to_vec)
        .collect()
}

/// An ADR as the change has it: its path, `docs/adr/<name>`, and its bytes as `cat` or `git show`
/// gives them, empty where they fail.
pub struct Adr {
    pub path: Vec<u8>,
    pub text: Vec<u8>,
}

/// The file at a path the change touches as `cat` or `git show` gives it, empty where they fail,
/// and `git diff -U0` of it, which the report reads for a file named with checks of its own.
pub struct Diffed {
    pub file: Vec<u8>,
    pub diff: Vec<u8>,
}

/// What the report reads of the record: the ADRs as the change has them, where
/// [`reads_adrs`] says so, and the index where [`reads_index`] says so.
pub struct Record {
    pub adrs: Vec<Adr>,
    pub index: Vec<u8>,
}

/// The report, as the bash of the release writes it on the standard output. `diffed` gives the file
/// at a path and its diff, asked only where the bash runs `checked` on that path.
pub fn report(
    release: Version,
    sections: bool,
    listing: &Listing,
    record: &Record,
    diffed: &mut dyn FnMut(&[u8]) -> Result<Diffed, Refusal>,
) -> Result<Vec<u8>, Refusal> {
    let mut out = Vec::new();
    let sections = sections && release >= OPTIONS;
    let (paths, range, verb, named): (&[Vec<u8>], &str, &str, bool) = match listing {
        Listing::Nothing => {
            out.extend(super::change::nothing_to_compare(LABEL));
            finish(&mut out, release, sections, record, &[]);
            return Ok(out);
        }
        Listing::Changed { range, paths } => (paths, range, "changed", false),
        Listing::Named(paths) => (paths, "named paths", "named", true),
    };
    if paths.is_empty() {
        out.extend(format!("{LABEL} ({range}): nothing to compare — no path changed.").bytes());
        if release >= OPTIONS {
            out.extend(b" Name the paths the work will touch with --path.");
        }
        out.push(b'\n');
        finish(&mut out, release, sections, record, &[]);
        return Ok(out);
    }
    let (found, files) = bind(&record.adrs, paths, named, release, diffed)?;
    let (hint, hint_list) = if sections {
        (
            "The ADRs whose Applies to is prose follow below.",
            "the ADRs whose Applies to is prose, below:",
        )
    } else {
        (
            "Read the index for the ADRs whose Applies to is prose.",
            "the index for the ADRs whose Applies to is prose:",
        )
    };
    let count = paths.len();
    if found.is_empty() {
        out.extend(
            format!(
                "{LABEL} ({range}): {count} path(s) {verb}; no ADR in force names one of them. \
                 {hint}\n"
            )
            .bytes(),
        );
    } else {
        out.extend(
            format!(
                "{LABEL} ({range}): {count} path(s) {verb}; {} ADR(s) in force name one of them. \
                 Read these, and {hint_list}\n",
                found.len()
            )
            .bytes(),
        );
        for line in &found {
            out.extend(b"  - ");
            out.extend(line);
            out.push(b'\n');
        }
    }
    finish(&mut out, release, sections, record, &files);
    Ok(out)
}

/// The line of each ADR in force that binds a path, and the ADR, as the bash's loop over the ADRs
/// finds them.
fn bind<'r>(
    adrs: &'r [Adr],
    paths: &[Vec<u8>],
    named: bool,
    release: Version,
    diffed: &mut dyn FnMut(&[u8]) -> Result<Diffed, Refusal>,
) -> Result<(Vec<Vec<u8>>, Vec<&'r Adr>), Refusal> {
    let mut touched: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
    let mut found = Vec::new();
    let mut files = Vec::new();
    for adr in adrs {
        let text = Text::of(&adr.text);
        if !text.in_force() {
            continue;
        }
        let applies = text.field_line(b"Applies to").unwrap_or_default();
        let bound = bound_paths(applies, release);
        if bound.is_empty() {
            continue;
        }
        let mut hits: Vec<&[u8]> = Vec::new();
        for changed in paths {
            for path in &bound {
                if !path.matches(changed, named, release) {
                    continue;
                }
                let Some(checks) = path.checks.as_deref().filter(|_| !named) else {
                    hits.push(changed);
                    break;
                };
                if !touched.contains_key(changed) {
                    let got = diffed(changed)?;
                    touched.insert(changed.clone(), held_in_hunks(&got, release));
                }
                if in_checks(&touched[changed], checks) {
                    hits.push(changed);
                    break;
                }
            }
        }
        if hits.is_empty() {
            continue;
        }
        let mut line = id(&adr.path, release);
        line.extend(b" (");
        line.extend(&adr.path);
        line.extend(") — ".as_bytes());
        line.extend(hits.iter().take(3).copied().collect::<Vec<_>>().join(&b' '));
        if hits.len() > 3 {
            line.extend(format!(", and {} more", hits.len() - 3).bytes());
        }
        found.push(line);
        files.push(adr);
    }
    Ok((found, files))
}

/// What `finish` prints for `--sections`: the prose of the ADRs in force not listed, and of each
/// ADR listed its header and the sections `ADR_RULE_SECTIONS` names.
fn finish(out: &mut Vec<u8>, release: Version, sections: bool, record: &Record, files: &[&Adr]) {
    if !sections {
        return;
    }
    let (rows, prose) = prose_rows(release, record, files);
    out.push(b'\n');
    if rows == 0 {
        if release >= NAMED {
            out.extend(
                b"No ADR in force found in docs/adr/; read the index for the ADRs whose Applies \
                  to is prose.\n",
            );
        } else {
            out.extend(
                b"No row found under '### Binding' in docs/adr/README.md; read the index for the \
                  ADRs whose Applies to is prose.\n",
            );
        }
    } else if prose.is_empty() {
        out.extend(
            b"Every other ADR in force names paths alone, and none of them names one of the \
              work.\n",
        );
    } else {
        out.extend(
            b"The ADRs in force no path above decides, by their Applies to; match the prose \
              against the work:\n",
        );
        for row in &prose {
            out.extend(b"  - ");
            out.extend(row);
            out.push(b'\n');
        }
    }
    for adr in files {
        print_sections(out, release, adr);
    }
}

/// How many ADRs in force `finish` counts, and the prose of each it prints: from `NAMED` read from
/// the ADRs' files, before it from the index's *Binding* table; an ADR listed left out.
fn prose_rows(release: Version, record: &Record, files: &[&Adr]) -> (usize, Vec<Vec<u8>>) {
    let mut rows = 0;
    let mut prose: Vec<Vec<u8>> = Vec::new();
    if release >= NAMED {
        let mut listed = b" ".to_vec();
        listed.extend(
            files
                .iter()
                .map(|adr| adr.path.as_slice())
                .collect::<Vec<_>>()
                .join(&b' '),
        );
        listed.push(b' ');
        for adr in &record.adrs {
            let text = Text::of(&adr.text);
            if !text.in_force() {
                continue;
            }
            rows += 1;
            if bash::contains(&listed, &[b" ", adr.path.as_slice(), b" "].concat()) {
                continue;
            }
            let applies = field_value(text.field_line(b"Applies to").unwrap_or_default());
            if is_prose(applies, release) {
                let mut row = id(&adr.path, release);
                row.extend(b": ");
                row.extend(applies);
                prose.push(row);
            }
        }
    } else {
        let mut listed = b" ".to_vec();
        for adr in files {
            listed.extend(key(&adr.path));
            listed.push(b' ');
        }
        for (key, applies) in binding_rows(&record.index) {
            rows += 1;
            if bash::contains(&listed, &[b" ", key.as_slice(), b" "].concat()) {
                continue;
            }
            if is_prose(&applies, release) {
                let mut row = b"ADR-".to_vec();
                row.extend(&key);
                row.extend(b": ");
                row.extend(&applies);
                prose.push(row);
            }
        }
    }
    (rows, prose)
}

/// Of an ADR listed, its banner, its header, and the sections `ADR_RULE_SECTIONS` names.
fn print_sections(out: &mut Vec<u8>, release: Version, adr: &Adr) {
    let text = Text::of(&adr.text);
    out.extend(b"\n=== ");
    out.extend(id(&adr.path, release));
    out.extend(b" (");
    out.extend(&adr.path);
    out.extend(b"): the header, the Decision, and the Enforcement ===\n");
    for line in &text.lines {
        if line.starts_with(b"## ") {
            break;
        }
        out.extend(*line);
        out.push(b'\n');
    }
    for (at, part) in ["Decision", "Enforcement"].into_iter().enumerate() {
        if at > 0 {
            out.push(b'\n');
        }
        let lines = if release >= FENCES {
            adr::section(text.stream(), part)
        } else {
            unfenced_section(&text.lines, part)
        };
        for line in lines {
            out.extend(line);
            out.push(b'\n');
        }
    }
}

/// An ADR's text as `text="$(show "$file")"` holds it and `<<< "$text"` hands it on: its newlines
/// at the end dropped, and one added.
struct Text<'a> {
    stream: Vec<u8>,
    lines: Vec<&'a [u8]>,
}

impl<'a> Text<'a> {
    fn of(bytes: &'a [u8]) -> Self {
        let kept = bash::substituted(bytes);
        let mut stream = kept.to_vec();
        stream.push(b'\n');
        Self {
            stream,
            lines: kept.split(|&b| b == b'\n').collect(),
        }
    }

    fn stream(&self) -> &[u8] {
        &self.stream
    }

    /// The first line `grep -m1 -E "^- [*][*]<field>:[*][*]"` finds.
    fn field_line(&self, field: &[u8]) -> Option<&'a [u8]> {
        let opening = [b"- **".as_slice(), field, b":**"].concat();
        self.lines
            .iter()
            .find(|line| line.starts_with(&opening))
            .copied()
    }

    /// Whether the status line names the ADR accepted or proposed, by its emoji.
    fn in_force(&self) -> bool {
        self.field_line(b"Status").is_some_and(|status| {
            bash::contains(status, "🟢".as_bytes()) || bash::contains(status, "🟡".as_bytes())
        })
    }
}

/// The section `## <name>` before `FENCES`: from its heading to the next `## ` line, as the
/// sensor's own `section` and the first `adr_section` print it.
fn unfenced_section<'a>(lines: &[&'a [u8]], name: &str) -> Vec<&'a [u8]> {
    let heading = format!("## {name}");
    let mut inside = false;
    let mut found = Vec::new();
    for line in lines {
        if *line == heading.as_bytes() {
            inside = true;
            found.push(*line);
            continue;
        }
        if inside && line.starts_with(b"## ") {
            break;
        }
        if inside {
            found.push(*line);
        }
    }
    found
}

/// What `adr_field_value` leaves of the line for `Applies to`, its leading POSIX space dropped:
/// what follows the last `**Applies to:**` on it.
fn field_value(line: &[u8]) -> &[u8] {
    let mark = b"**Applies to:**";
    let value = line
        .windows(mark.len())
        .rposition(|window| window == mark)
        .map_or(line, |at| &line[at + mark.len()..]);
    let start = value
        .iter()
        .position(|&b| !bash::is_space(b))
        .unwrap_or(value.len());
    &value[start..]
}

/// Whether an *Applies to* is prose: an item of it, split at `, and `, ` and `, and `, `, is not
/// one path in backticks, as `grep -qvE` finds a line of the split; from `PROSE_SPLIT` the
/// newlines that end the split are dropped first.
fn is_prose(applies: &[u8], release: Version) -> bool {
    let mut split = applies.to_vec();
    for separator in [&b", and "[..], b" and ", b", "] {
        split = replace(&split, separator, b"\n");
    }
    let split = if release >= PROSE_SPLIT {
        bash::substituted(&split)
    } else {
        &split[..]
    };
    split.split(|&b| b == b'\n').any(|item| !is_one_path(item))
}

/// The bytes with each occurrence of the needle, leftmost first, replaced, as `sed s///g` does.
fn replace(text: &[u8], needle: &[u8], with: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = bash::find(rest, needle) {
        out.extend(&rest[..at]);
        out.extend(with);
        rest = &rest[at + needle.len()..];
    }
    out.extend(rest);
    out
}

/// Whether the item is one path in backticks with POSIX space around it only.
fn is_one_path(item: &[u8]) -> bool {
    let item = bash::trim(item);
    item.len() > 2
        && item.starts_with(b"`")
        && item.ends_with(b"`")
        && item[1..item.len() - 1]
            .iter()
            .all(|&b| b != b'`' && !bash::is_space(b))
}

/// The rows of the index's *Binding* table, as the awk before `NAMED` prints them and
/// `IFS=$'\t' read -r key applies` reads them: the key of the row's link and its *Applies to*.
fn binding_rows(index: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut rows = Vec::new();
    let mut inside = false;
    for line in bash::awk_lines(index) {
        if line.starts_with(b"### Binding") {
            inside = true;
            continue;
        }
        if inside && line.starts_with(b"#") {
            break;
        }
        if !(inside && line.starts_with(b"| [")) {
            continue;
        }
        let fields: Vec<&[u8]> = line.split(|&b| b == b'|').collect();
        let field = |n: usize| fields.get(n).copied().unwrap_or_default();
        let key = field(1);
        // sub(/^ *\[/, "", key): the spaces go only with the bracket after them.
        let spaces = key.iter().take_while(|&&b| b == b' ').count();
        let key = key[spaces..].strip_prefix(b"[").unwrap_or(key);
        let key = key
            .iter()
            .position(|&b| b == b']')
            .map_or(key, |at| &key[..at]);
        let applies = field(3);
        let start = applies.iter().take_while(|&&b| b == b' ').count();
        let end = applies
            .iter()
            .rposition(|&b| b != b' ')
            .map_or(start, |i| i + 1);
        let printed = [key, b"\t", &applies[start..end.max(start)]].concat();
        rows.push(bash::read_tab_pair(&printed));
    }
    rows
}

/// The identifier the report names an ADR by: `ADR-` and its key before `NAMED`, its file's name
/// without `.md` from it.
fn id(path: &[u8], release: Version) -> Vec<u8> {
    let name = basename(path);
    if release >= NAMED {
        return name.strip_suffix(b".md").unwrap_or(name).to_vec();
    }
    let mut id = b"ADR-".to_vec();
    id.extend(key(path));
    id
}

/// What follows `ADR-` in the identifier of the file before `NAMED`: a draft's name without
/// `.md`, else what precedes its first hyphen.
fn key(path: &[u8]) -> &[u8] {
    let name = basename(path);
    if name.starts_with(b"draft-") {
        name.strip_suffix(b".md").unwrap_or(name)
    } else {
        name.iter()
            .position(|&b| b == b'-')
            .map_or(name, |at| &name[..at])
    }
}

/// The part of the path after its last `/`.
fn basename(path: &[u8]) -> &[u8] {
    path.iter()
        .rposition(|&b| b == b'/')
        .map_or(path, |at| &path[at + 1..])
}

/// A path an *Applies to* names, with the directory of its module's children, and the checks of
/// the file it is named with: their numbers before `RULE_KEYS`, their rule keys from it, each a
/// word as `for n in $1` splits them; none where it binds the whole file.
struct Bound {
    path: Vec<u8>,
    dir: Vec<u8>,
    checks: Option<Vec<Vec<u8>>>,
}

impl Bound {
    /// Whether the changed or named path lies at or under the path, under the directory of its
    /// module's children, or, named, above it.
    fn matches(&self, changed: &[u8], named: bool, release: Version) -> bool {
        let p = &self.path;
        let under = |prefix: &[u8], path: &[u8]| path.starts_with(prefix);
        let mut slashed = p.clone();
        slashed.push(b'/');
        if changed == p.as_slice() || under(&slashed, changed) {
            return true;
        }
        if release >= MODULE_DIRS && !self.dir.is_empty() && under(&self.dir, changed) {
            return true;
        }
        if named {
            let mut above = changed.strip_suffix(b"/").unwrap_or(changed).to_vec();
            above.push(b'/');
            return under(&above, p);
        }
        false
    }
}

/// The paths the *Applies to* line names, as the release reads them.
fn bound_paths(applies: &[u8], release: Version) -> Vec<Bound> {
    let mut bound = Vec::new();
    let mut push = |path: &[u8], checks: Option<Vec<Vec<u8>>>| {
        bound.push(Bound {
            dir: module_dir(path),
            path: path.to_vec(),
            checks,
        });
    };
    if release >= RULE_KEYS {
        for (path, rules) in adr::rule_path_bytes(applies) {
            // printf '%s\t%s\n' "${p%/}" "$rules" read back by `IFS=$'\t' read -r p rules`.
            let printed = [path.as_slice(), b"\t", &rules.join(&b' ')].concat();
            let (path, rules) = bash::read_tab_pair(&printed);
            push(&path, (!rules.is_empty()).then(|| words(&rules)));
        }
    } else if release >= NAMED {
        let token = regex!(r"(?-u)(checks? [0-9][0-9, and]* of )?`[^`]+`");
        for m in token.find_iter(applies) {
            let text = m.as_bytes();
            let open = text.iter().position(|&b| b == b'`').expect("a code span");
            let path = &text[open + 1..text.len() - 1];
            if !is_word(path) {
                continue;
            }
            let numbers: Vec<Vec<u8>> = text[..open]
                .split(|b| !b.is_ascii_digit())
                .filter(|run| !run.is_empty())
                .map(<[u8]>::to_vec)
                .collect();
            let checks = (!numbers.is_empty()).then_some(numbers);
            push(path.strip_suffix(b"/").unwrap_or(path), checks);
        }
    } else {
        let token = regex!(r"(?-u)`[^`]+`");
        for m in token.find_iter(applies) {
            let text = m.as_bytes();
            let path = &text[1..text.len() - 1];
            if is_word(path) {
                push(path.strip_suffix(b"/").unwrap_or(path), None);
            }
        }
    }
    bound
}

/// Whether the bytes are one run with no POSIX space, as `=~ ^[^[:space:]]+$` matches them.
fn is_word(bytes: &[u8]) -> bool {
    !bytes.is_empty() && !bytes.iter().any(|&b| bash::is_space(b))
}

/// The words of the keys as `for n in $1` splits them with the default IFS.
fn words(keys: &[u8]) -> Vec<Vec<u8>> {
    keys.split(|&b| b == b' ' || b == b'\t' || b == b'\n')
        .filter(|word| !word.is_empty())
        .map(<[u8]>::to_vec)
        .collect()
}

/// The extension whose files lay out a module's children in the directory of the file's name, and
/// the names of that extension that do not, as `MODULE_DIRS` lists them.
const MODULE_EXTENSION: &[u8] = b"rs";
const NOT_MODULE_DIRS: [&[u8]; 4] = [b"lib.rs", b"main.rs", b"mod.rs", b"build.rs"];

/// The directory that holds the children of the module in the file, with its trailing slash, as
/// `module_dir` finds it; empty for a path that names none.
fn module_dir(path: &[u8]) -> Vec<u8> {
    let name = basename(path);
    // `?*.*`: a dot after the first byte.
    if name.len() < 2 || !name[1..].contains(&b'.') {
        return Vec::new();
    }
    let dot = name.iter().rposition(|&b| b == b'.').expect("a dot");
    if &name[dot + 1..] != MODULE_EXTENSION || NOT_MODULE_DIRS.contains(&name) {
        return Vec::new();
    }
    let at = path.iter().rposition(|&b| b == b'.').expect("a dot");
    let mut dir = path[..at].to_vec();
    dir.push(b'/');
    dir
}

/// What `checked` reads of a file the change touches: the string its awk prints of the checks
/// that hold the lines of each hunk and the lines on either side, ` <held> <held> … `, where a line
/// outside every check is `all`; ` ` where the diff has no hunk.
fn held_in_hunks(diffed: &Diffed, release: Version) -> Vec<u8> {
    let held = held(&diffed.file, release);
    let hunk = regex!(r"(?-u)^@@ [^+]*\+([0-9,]+) ");
    let mut touched: BTreeSet<&[u8]> = BTreeSet::new();
    for line in bash::sed_lines(&diffed.diff) {
        let Some(captures) = hunk.captures(line) else {
            continue;
        };
        let range = &captures[1];
        let mut parts = range.split(|&b| b == b',');
        let start = awk_number(parts.next().unwrap_or_default());
        let count = parts
            .next()
            .filter(|count| !count.is_empty())
            .map_or(1, awk_number);
        let first = start - i64::from(count > 0);
        let last = first + count + 1;
        // Each line of the range; one outside the file holds no check.
        let within = |i: i64| i >= 1 && i <= held.len() as i64;
        if !within(first) || !within(last) {
            touched.insert(b"all");
        }
        let from = first.max(1);
        let to = last.min(held.len() as i64);
        for i in from..=to {
            let line = held[(i - 1) as usize].as_slice();
            touched.insert(if awk_true(line) { line } else { b"all" });
        }
    }
    // `o = o " " k` for each key, then `print o " "`.
    let mut out = b" ".to_vec();
    for key in touched {
        out.extend(key);
        out.push(b' ');
    }
    out
}

/// The number awk reads at the start of the field: its leading digits, 0 for none.
fn awk_number(field: &[u8]) -> i64 {
    field
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .fold(0i64, |n, &b| {
            n.saturating_mul(10).saturating_add(i64::from(b - b'0'))
        })
}

/// Whether awk reads the line `held` printed as true: a line of digits as its number, any other
/// line as a string, which is true unless empty.
fn awk_true(line: &[u8]) -> bool {
    if line.is_empty() {
        return false;
    }
    if line.iter().all(u8::is_ascii_digit) {
        return line.iter().any(|&b| b != b'0');
    }
    true
}

/// Of each line of the file, and one more, what `held` prints: the number of the check whose
/// function holds it before `RULE_KEYS`, its rule key and the old keys its item carries in the
/// header from it; empty, or `0` after a check's closing brace before `RULE_KEYS`, outside.
fn held(file: &[u8], release: Version) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    if release >= RULE_KEYS {
        let item = regex!(r"(?-u)^#   [a-z]+/[a-z0-9-]+ \(was: [^)]+\)$");
        let check = regex!(r"(?-u)^check_[a-z_]+\(\) \{  # [a-z]+/[a-z0-9-]+$");
        let mut header = true;
        let mut old: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        let mut n: Vec<u8> = Vec::new();
        for line in bash::awk_lines(file) {
            if line.starts_with(b"set ") {
                header = false;
            }
            if header && item.is_match(line) {
                let rest = &line[b"#   ".len()..];
                let after = bash::find(rest, b" (was: ").expect("the item matched") + 7;
                let keys: Vec<u8> = rest[after..rest.len() - 1]
                    .iter()
                    .copied()
                    .filter(|&b| b != b',')
                    .collect();
                let fields = bash::awk_fields(line);
                let mut value = b" ".to_vec();
                value.extend(keys);
                old.insert(fields[1].to_vec(), value);
            }
            if check.is_match(line) {
                n = bash::awk_fields(line).last().expect("a field").to_vec();
            }
            let mut printed = n.clone();
            printed.extend(old.get(&n).map(Vec::as_slice).unwrap_or_default());
            out.push(printed);
            if line.starts_with(b"}") {
                n.clear();
            }
        }
        out.push(Vec::new());
    } else {
        let check = regex!(r"(?-u)^check_.*# check [0-9]+$");
        let mut n: Vec<u8> = Vec::new();
        for line in bash::awk_lines(file) {
            if check.is_match(line) {
                n = bash::awk_fields(line).last().expect("a field").to_vec();
            }
            out.push(n.clone());
            if line.starts_with(b"}") {
                n = b"0".to_vec();
            }
        }
        out.push(b"0".to_vec());
    }
    out
}

/// Whether `checked` finds the change in one of the checks: a hunk in the function of a check
/// whose key or number is among them, or outside every check, or no hunk at all.
fn in_checks(touched: &[u8], checks: &[Vec<u8>]) -> bool {
    touched == b" "
        || checks
            .iter()
            .map(Vec::as_slice)
            .chain([&b"all"[..]])
            .any(|word| bash::contains(touched, &[b" ", word, b" "].concat()))
}

/// The refusal of a path the sensor reads that holds a NUL byte, which the bash's command
/// substitution drops and awk does not read as the other bytes (Q-1).
pub fn with_nul(path: &str) -> Refusal {
    Refusal {
        sensor: SENSOR.name,
        reason: "file",
        message: format!("{path}: a NUL byte, which the bash does not read as the other bytes"),
    }
}

#[cfg(test)]
mod tests;
