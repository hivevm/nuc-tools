//! The citations of the record.
//!
//! No tracked file outside `docs/adr/` and `docs/plan/` cites an ADR whose status is superseded or
//! rejected: a supersession moves every citation of the old ADR to its successor in the same
//! change, as the convention "A rule changes everywhere it is stated" of `docs/CONVENTIONS.md`
//! asks. The tickets under `docs/plan/` name superseded ADRs as history. Such an identifier stands
//! only where `adr_citations.template`, the list kept beside this test, names it by file. An
//! identifier is `<CAT>-<slug>`, the form of the record's file names; an old number such as a
//! message of `nuc` copies from the bash names an ADR of another record and is not read
//! (TEST-copies-keep-old-identifiers). An entry of the list that names no citation fails too, so
//! the list shrinks when a citation goes. A tracked symbolic link is not followed.

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

/// An identifier of the record, assembled so that this file cites none.
fn id(slug: &str) -> String {
    format!("{}-{slug}", "TEST")
}

/// The list of the places that may cite a dead ADR, one `<path>\t<identifier>` per line.
const TEMPLATE: &str = include_str!("adr_citations.template");
const TEMPLATE_PATH: &str = "tests/adr_citations.template";

/// The directories whose files may cite any ADR: the record itself, and the open work.
const FREE: [&str; 2] = ["docs/adr/", "docs/plan/"];

/// The entries of the list, `(path, identifier)`, without comments and blank lines.
fn exempt(list: &str) -> BTreeSet<(String, String)> {
    list.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (path, id) = line
                .split_once('\t')
                .unwrap_or_else(|| panic!("'{line}' is no '<path>\\t<identifier>' entry"));
            (path.to_owned(), id.to_owned())
        })
        .collect()
}

/// Every identifier `<CAT>-<slug>` the line names as a word: a key of two to five capital letters,
/// a hyphen, and a slug of lower-case words or numbers joined by hyphens, with no letter, digit, or
/// hyphen beside it. Whether it names an ADR of the record is the caller's to look up.
fn cited(line: &str) -> Vec<String> {
    let bytes = line.as_bytes();
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'-';
    let slug = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-';
    let mut ids = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_uppercase() || (i > 0 && word(bytes[i - 1])) {
            i += 1;
            continue;
        }
        let key = bytes[i..]
            .iter()
            .take_while(|b| b.is_ascii_uppercase())
            .count();
        let dash = i + key;
        let tail = bytes.get(dash + 1..).unwrap_or_default();
        let len = tail.iter().take_while(|&&b| slug(b)).count();
        let end = dash + 1 + len;
        if (2..=5).contains(&key)
            && bytes.get(dash) == Some(&b'-')
            && len > 0
            && bytes[end - 1] != b'-'
            && bytes.get(end).is_none_or(|&b| !word(b))
        {
            ids.push(line[i..end].to_owned());
        }
        i = end.max(i + 1);
    }
    ids
}

/// What a supersession left behind: each citation of a dead ADR, by `file:line`, at a place the
/// list does not name, and each entry of the list that names no citation.
fn left_behind(
    files: &[(String, String)],
    dead: &BTreeMap<String, String>,
    exempt: &BTreeSet<(String, String)>,
) -> Vec<String> {
    let mut found = Vec::new();
    let mut used = BTreeSet::new();
    for (path, text) in files {
        if FREE.iter().any(|dir| path.starts_with(dir)) || path == TEMPLATE_PATH {
            continue;
        }
        for (n, line) in text.lines().enumerate() {
            for id in cited(line) {
                let Some(status) = dead.get(&id) else {
                    continue;
                };
                let key = (path.clone(), id.clone());
                if exempt.contains(&key) {
                    used.insert(key);
                } else {
                    found.push(format!("{path}:{}: cites {id}, which is {status}", n + 1));
                }
            }
        }
    }
    for (path, id) in exempt.difference(&used) {
        found.push(format!(
            "{TEMPLATE_PATH}: '{path}' cites {id} no more, or {id} is in force — drop the entry"
        ));
    }
    found
}

/// The ADRs of the record whose status is superseded or rejected, by identifier, with the status.
/// A file of the record is `docs/adr/<CAT>-<slug>.md`; the index and the template are none.
fn dead(root: &Path) -> BTreeMap<String, String> {
    let mut dead = BTreeMap::new();
    for entry in fs::read_dir(root.join("docs/adr")).expect("docs/adr/ is listed") {
        let name = entry.expect("an entry").file_name();
        let name = name.to_string_lossy();
        let Some(identifier) = name
            .strip_suffix(".md")
            .filter(|stem| cited(stem).first().map(String::as_str) == Some(*stem))
        else {
            continue;
        };
        let text = fs::read_to_string(root.join("docs/adr").join(&*name)).expect("an ADR");
        let status = text
            .lines()
            .find_map(|line| line.strip_prefix("- **Status:** "))
            .unwrap_or_default();
        if status.contains("superseded") || status.contains("rejected") {
            let status = status.trim_start_matches(|c: char| !c.is_ascii_alphabetic());
            dead.insert(identifier.to_owned(), status.to_owned());
        }
    }
    dead
}

/// The tracked regular files of the repository with their text, as git lists them, a file of no
/// UTF-8 read lossily.
fn tracked(root: &Path) -> Vec<(String, String)> {
    let output = Command::new("git")
        .args(["-C", &root.display().to_string(), "ls-files", "-z"])
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .expect("git lists the tracked files");
    assert!(output.status.success(), "git ls-files failed");
    String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .filter(|path| {
            !path.is_empty()
                && fs::symlink_metadata(root.join(path)).is_ok_and(|meta| meta.is_file())
        })
        .map(|path| {
            let bytes = fs::read(root.join(path)).expect("a tracked file is read");
            (
                path.to_owned(),
                String::from_utf8_lossy(&bytes).into_owned(),
            )
        })
        .collect()
}

#[test]
fn no_file_outside_the_record_cites_a_dead_adr_but_where_the_list_names_it() {
    let root = support::sources::root();
    let found = left_behind(&tracked(root), &dead(root), &exempt(TEMPLATE));
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_supersession_that_leaves_a_citation_behind_fails() {
    let root = support::sources::root();
    let files = tracked(root);
    let mut dead = dead(root);
    // An ADR in force that the workflow cites: were it superseded today, every citation of it
    // would be left behind. It is taken from the tree, so that it holds after a normalization.
    let workflow = ".github/workflows/checks.yml";
    let cited_id = files
        .iter()
        .filter(|(path, _)| path == workflow)
        .flat_map(|(_, text)| text.lines().flat_map(cited).collect::<Vec<_>>())
        .find(|id| !dead.contains_key(id))
        .expect("the workflow cites an ADR in force");
    let successor = format!("superseded by {}", id("successor-of-it"));
    dead.insert(cited_id.clone(), successor.clone());
    let found = left_behind(&files, &dead, &exempt(TEMPLATE));
    let message = format!("cites {cited_id}, which is {successor}");
    assert!(
        found
            .iter()
            .any(|f| f.starts_with(&format!("{workflow}:")) && f.ends_with(&message)),
        "{}",
        found.join("\n")
    );
}

#[test]
fn a_dead_citation_fails_by_file_and_line_and_passes_where_the_list_names_it() {
    let (gone, other) = (id("gone-one"), id("other-one"));
    let files = vec![
        ("src/a.rs".to_owned(), format!("x\n// {gone} and {other}\n")),
        (
            format!("docs/adr/{}.md", id("third-one")),
            format!("{gone}\n"),
        ),
        ("docs/plan/s/01-t.md".to_owned(), format!("{gone}\n")),
        (
            "scripts/b.sh".to_owned(),
            format!("# {gone}, {gone}-more, X{gone}\n"),
        ),
    ];
    let dead: BTreeMap<String, String> = [(gone.clone(), "rejected".to_owned())].into();
    assert_eq!(
        left_behind(&files, &dead, &BTreeSet::new()),
        [
            format!("src/a.rs:2: cites {gone}, which is rejected"),
            format!("scripts/b.sh:1: cites {gone}, which is rejected"),
        ]
    );
    let list = exempt(&format!(
        "# a copy of another record\nscripts/b.sh\t{gone}\n"
    ));
    assert_eq!(
        left_behind(&files, &dead, &list),
        [format!("src/a.rs:2: cites {gone}, which is rejected")]
    );
}

#[test]
fn an_entry_that_names_no_citation_fails() {
    let gone = id("gone-one");
    let files = vec![("src/a.rs".to_owned(), "nothing\n".to_owned())];
    let dead: BTreeMap<String, String> = [(gone.clone(), "rejected".to_owned())].into();
    assert_eq!(
        left_behind(&files, &dead, &exempt(&format!("src/a.rs\t{gone}\n"))),
        [format!(
            "tests/adr_citations.template: 'src/a.rs' cites {gone} no more, or {gone} is in force — drop the entry"
        )]
    );
}
