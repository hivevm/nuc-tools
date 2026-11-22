//! `nuc upgrade <tag>`: the take-up of a template release, decided from three trees, the old
//! release's, the new release's, and the project's `HEAD`, as a plan the shell writes into the
//! working tree once no refusal stands (API-upgrade-takes-up-by-blob). Every function here is pure;
//! the shell reads the trees and the README, and carries out the plan.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::config::Version;
use crate::finding::Finding;
use crate::rules::{Carried, docs_base_blobs, docs_template_release};

/// A rule of `nuc upgrade`: the ID and reasons of the judgments it lists, and what `nuc explain`
/// gives for it. `nuc.toml` turns none of them on or off.
#[derive(Debug)]
pub struct Rule {
    pub id: &'static str,
    pub reasons: &'static [&'static str],
    pub explain: &'static str,
}

/// A path the plan would write or delete that a tree holds otherwise than as a file.
pub const LINK: &str = "link";

/// A path the plan would write where the working tree keeps a file or a directory of its own, or
/// a file above it.
pub const ADDED: &str = "added";

/// A mode the plan writes that the working tree cannot carry.
pub const MODE: &str = "mode";

/// The judgments on a path of the tree.
pub const FILE: Rule = Rule {
    id: "upgrade/file",
    reasons: &[ADDED, LINK, MODE],
    explain: "nuc upgrade takes up a path of a template release only where the project left it as \
              its release had it, and writes regular files only; it lists each other path for the \
              maintainer to take up by hand, and leaves its bytes as they are \
              (API-upgrade-takes-up-by-blob).",
};

/// Every rule of `nuc upgrade`.
pub const ALL: &[&Rule] = &[&FILE];

/// The rule a refusal of `nuc upgrade` is keyed under, `[upgrade/run:<reason>]`.
pub const RUN: &str = "upgrade/run";

/// The reasons of a refusal, each a cause that keeps `nuc upgrade` from writing.
pub const RUN_REASONS: &[&str] = &[
    "usage", "main", "changed", "release", "old", "order", "ref", "version", "path", "write",
];

/// The earliest release a take-up starts from: before it, a project still has numbered ADRs.
pub const EARLIEST: Version = Version(0, 9, 0);

/// The rule of that ID, if `nuc upgrade` has it.
pub fn find(id: &str) -> Option<&'static Rule> {
    ALL.iter().copied().find(|rule| rule.id == id)
}

/// A refusal: nothing is written, and the run exits 2 with the key `[upgrade/run:<reason>]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub reason: &'static str,
    pub message: String,
}

impl Refusal {
    pub fn new(reason: &'static str, message: impl Into<String>) -> Self {
        debug_assert!(RUN_REASONS.contains(&reason), "{reason}");
        Self {
            reason,
            message: message.into(),
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} — not decided [{RUN}:{}]", self.message, self.reason)
    }
}

/// The tag `vX.Y.Z` of a release.
pub fn tag(release: Version) -> String {
    format!("v{release}")
}

/// The release a tag `vX.Y.Z` names, each number without a leading zero, as
/// NUC-releases-are-semver-tags forms it; none for any other text.
pub fn parse_tag(text: &str) -> Option<Version> {
    let rest = docs_template_release::tag(text)?;
    rest.is_empty()
        .then(|| Version::parse(&text[1..]))
        .flatten()
}

/// The old release, the one the README's Template release line names, and the new one, the
/// argument; refused where the line names none, `unreleased`, or a fork, where the old release is
/// before v0.9.0, and where the new one is no tag above the old.
pub fn releases(readme: &str, argument: &str) -> Result<(Version, Version), Refusal> {
    let old = match docs_template_release::carried(readme) {
        Some(Carried::Release(old)) if !docs_base_blobs::forked(readme) => old,
        Some(Carried::Release(old)) => {
            return Err(Refusal::new(
                "release",
                format!(
                    "README.md: the Template release line names {}, forked; a fork takes up a \
                     release by hand",
                    tag(old)
                ),
            ));
        }
        Some(Carried::Unreleased) => {
            return Err(Refusal::new(
                "release",
                "README.md: the Template release line names 'unreleased', no release to take up \
                 from",
            ));
        }
        None => {
            return Err(Refusal::new(
                "release",
                "README.md: no Template release line names a 'vX.Y.Z' tag to take up from",
            ));
        }
    };
    if old < EARLIEST {
        return Err(Refusal::new(
            "old",
            format!(
                "the project carries {}, before {}: it takes up a v0.9 release first, with the \
                 normalize skill that release ships, which migrates its numbered ADRs",
                tag(old),
                tag(EARLIEST)
            ),
        ));
    }
    let new = parse_tag(argument).ok_or_else(|| {
        Refusal::new(
            "usage",
            format!("'{argument}' is no release tag 'vX.Y.Z' to take up"),
        )
    })?;
    if new <= old {
        return Err(Refusal::new(
            "order",
            format!(
                "{} is not above {}, the release the project carries",
                tag(new),
                tag(old)
            ),
        ));
    }
    Ok((old, new))
}

/// What a tree holds at a path: a regular file, executable or not, a symbolic link, or a
/// submodule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    File,
    Executable,
    Link,
    Submodule,
}

impl Kind {
    /// The kind a mode of `git ls-tree` names; none for a mode git writes for no entry of a tree
    /// it lists recursively.
    pub fn of_mode(mode: &str) -> Option<Self> {
        match mode {
            "100644" => Some(Self::File),
            "100755" => Some(Self::Executable),
            "120000" => Some(Self::Link),
            "160000" => Some(Self::Submodule),
            _ => None,
        }
    }
}

/// An entry of a tree: its kind and its object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub kind: Kind,
    pub object: String,
}

/// A tree, every path it holds below its root, `/`-separated, with its entry.
pub type Tree = BTreeMap<String, Entry>;

/// What the plan does to a path: write the entry, a regular file, or delete the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Write(Entry),
    Delete,
}

/// A take-up: the changes to the working tree, by path, and the judgments it leaves to the
/// maintainer, each a finding.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Plan {
    pub changes: BTreeMap<String, Change>,
    pub judgments: Vec<Finding>,
}

/// Whether the path is part of the base a project never edits: `AGENTS.md`, `docs/adr/NUC.blobs`,
/// and each `docs/adr/NUC-*.md` (NUC-base-is-fixed §2).
pub fn is_base(path: &str) -> bool {
    path == "AGENTS.md"
        || path == "docs/adr/NUC.blobs"
        || path
            .strip_prefix("docs/adr/NUC-")
            .is_some_and(|rest| rest.ends_with(".md") && !rest.contains('/'))
}

/// The take-up from `old` to `new` of the project at `head`: the base takes the new release's
/// entry, whatever the project holds, and a base path the new release lacks is deleted. A path
/// the plan would write or delete is a judgment instead where one of the trees holds a link or a
/// submodule there. Where the working tree carries no executable bit, as `carries_mode` says, a
/// file whose bit the plan changes is written, and its mode is a judgment.
pub fn plan(old: &Tree, new: &Tree, head: &Tree, carries_mode: bool) -> Plan {
    let mut plan = Plan::default();
    let paths: BTreeSet<&String> = old.keys().chain(new.keys()).chain(head.keys()).collect();
    for path in paths {
        if !is_base(path) {
            continue;
        }
        let (was, now, have) = (old.get(path), new.get(path), head.get(path));
        if now == have {
            continue;
        }
        let change = match now {
            Some(entry) => Change::Write(entry.clone()),
            None => Change::Delete,
        };
        let linked = [was, now, have]
            .into_iter()
            .flatten()
            .any(|entry| matches!(entry.kind, Kind::Link | Kind::Submodule));
        if linked {
            plan.judgments.push(Finding {
                rule: FILE.id,
                reason: LINK,
                file: path.clone(),
                line: None,
                message: "a symbolic link or a submodule in the old release, the new one, or the \
                          project; take the path up by hand"
                    .to_owned(),
            });
            continue;
        }
        if let Change::Write(entry) = &change {
            let executable =
                |entry: Option<&Entry>| entry.is_some_and(|e| e.kind == Kind::Executable);
            if !carries_mode && executable(Some(entry)) != executable(have) {
                plan.judgments.push(Finding {
                    rule: FILE.id,
                    reason: MODE,
                    file: path.clone(),
                    line: None,
                    message: format!(
                        "written, but the working tree carries no executable bit; set it in the \
                         index with 'git update-index --chmod={}x {path}'",
                        if executable(Some(entry)) { '+' } else { '-' }
                    ),
                });
            }
        }
        plan.changes.insert(path.clone(), change);
    }
    plan
}

/// What the working tree holds at a path, read without following a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Found {
    Nothing,
    Regular,
    Dir,
    Link,
    /// Anything else, such as a device or a socket.
    Other,
}

/// The paths of the working tree `settle` needs: each path the plan writes or deletes, and each
/// directory above it.
pub fn asked(plan: &Plan) -> BTreeSet<String> {
    plan.changes
        .keys()
        .flat_map(|path| {
            path.match_indices('/')
                .map(move |(at, _)| path[..at].to_owned())
                .chain(std::iter::once(path.clone()))
        })
        .collect()
}

/// Refused where a path of the plan is one `nuc upgrade` writes on no platform, or differs only in
/// letter case from another it writes or one `HEAD` keeps: nothing is written.
pub fn refuse_paths(plan: &Plan, head: &Tree) -> Result<(), Refusal> {
    for path in plan.changes.keys() {
        if let Some(why) = refused_path(path) {
            return Err(Refusal::new("path", format!("{path}: {why}")));
        }
    }
    let writes: Vec<&str> = plan
        .changes
        .iter()
        .filter(|(_, change)| matches!(change, Change::Write(_)))
        .map(|(path, _)| path.as_str())
        .collect();
    let kept: Vec<&str> = head
        .keys()
        .filter(|path| plan.changes.get(*path) != Some(&Change::Delete))
        .map(String::as_str)
        .collect();
    if let Some((path, other)) = case_collision(&writes, &kept) {
        return Err(Refusal::new(
            "path",
            format!(
                "{path}: differs only in letter case from {other}, which a file system that \
                 ignores case reads as one"
            ),
        ));
    }
    Ok(())
}

/// The plan settled against the working tree, as `found` reads each path `asked` names: refused
/// where a directory above a path is a symbolic link or no file and no directory, since a write
/// there would leave the tree (Q-8). A write the working tree has a path of its own at, or a file
/// above that the plan does not delete, becomes a judgment.
pub fn settle(
    plan: &mut Plan,
    head: &Tree,
    found: &BTreeMap<String, Found>,
    ignores_case: bool,
) -> Result<(), Refusal> {
    let at = |path: &str| found.get(path).copied().unwrap_or(Found::Nothing);
    // A path the plan deletes that the working tree lacks, as under a sparse checkout, needs no
    // deletion.
    plan.changes
        .retain(|path, change| *change != Change::Delete || at(path) != Found::Nothing);
    // A file a file system that ignores case finds at a path the plan writes, where the plan
    // deletes the path of the same letters in another case: the release renamed it by case.
    let renamed = |path: &str| {
        ignores_case
            && plan.changes.iter().any(|(other, change)| {
                *change == Change::Delete
                    && other != path
                    && other.to_lowercase() == path.to_lowercase()
            })
    };
    let mut taken = Vec::new();
    for (path, change) in &plan.changes {
        let mut file_above = None;
        for (end, _) in path.match_indices('/') {
            let dir = &path[..end];
            match at(dir) {
                Found::Link | Found::Other => {
                    return Err(Refusal::new(
                        "path",
                        format!(
                            "{path}: {dir} in the working tree is a symbolic link or no directory, \
                             through which a write would leave the tree"
                        ),
                    ));
                }
                Found::Regular if plan.changes.get(dir) != Some(&Change::Delete) => {
                    file_above.get_or_insert(dir);
                }
                _ => {}
            }
        }
        let tracked_file = head
            .get(path)
            .is_some_and(|entry| matches!(entry.kind, Kind::File | Kind::Executable));
        let own = match change {
            Change::Write(_) => match at(path) {
                Found::Nothing => None,
                Found::Regular if tracked_file || renamed(path) => None,
                Found::Regular => Some("a file of the working tree's own".to_owned()),
                Found::Dir => Some("a directory of the working tree".to_owned()),
                Found::Link | Found::Other => {
                    Some("a symbolic link or no file of the working tree".to_owned())
                }
            },
            Change::Delete => match at(path) {
                Found::Regular | Found::Nothing => None,
                _ => {
                    return Err(Refusal::new(
                        "path",
                        format!("{path}: the working tree keeps no file there to delete"),
                    ));
                }
            },
        };
        let own = own.or_else(|| file_above.map(|dir| format!("the file {dir} above it")));
        if let Some(own) = own {
            taken.push((path.clone(), own));
        }
    }
    for (path, own) in taken {
        plan.changes.remove(&path);
        plan.judgments
            .retain(|judgment| judgment.file != path || judgment.reason != MODE);
        plan.judgments.push(Finding {
            rule: FILE.id,
            reason: ADDED,
            file: path,
            line: None,
            message: format!(
                "the new release writes this path, and the working tree keeps {own}; take it up by \
                 hand"
            ),
        });
    }
    plan.judgments.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(())
}

/// The code points HFS+ ignores in a name, as git's `is_hfs_dotgit` skips them.
fn hfs_ignored(c: char) -> bool {
    matches!(
        c,
        '\u{200c}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{206a}'..='\u{206f}' | '\u{feff}'
    )
}

/// Whether a component of a path names the git directory to some file system: `.git` in any
/// case, `.git` with the code points HFS+ ignores, and the names NTFS reads as `.git`, its short
/// name `git~1` and either followed by dots and spaces or by a stream after a colon, as git's
/// `verify_path` refuses them with `core.protectHFS` and `core.protectNTFS` on.
fn names_git_dir(component: &str) -> bool {
    let hfs: String = component.chars().filter(|&c| !hfs_ignored(c)).collect();
    if hfs.eq_ignore_ascii_case(".git") {
        return true;
    }
    let lower = component.to_ascii_lowercase();
    [".git", "git~1"].iter().any(|name| {
        lower.strip_prefix(name).is_some_and(|rest| {
            let rest = rest.trim_start_matches(['.', ' ']);
            rest.is_empty() || rest.starts_with(':')
        })
    })
}

/// Why `nuc upgrade` refuses to write or delete the path, whatever the platform: none where it may.
/// Refused are an absolute path, a colon, which names a drive or a stream, a backslash, an empty,
/// `.`, or `..` component, and a component that names the git directory to some file system.
pub fn refused_path(path: &str) -> Option<&'static str> {
    if path.starts_with('/') {
        return Some("an absolute path");
    }
    if path.contains('\\') {
        return Some("a backslash, which Windows reads as a separator");
    }
    if path.contains(':') {
        return Some("a colon, which Windows reads as a drive or a stream of a file");
    }
    for component in path.split('/') {
        if component.is_empty() || component == "." || component == ".." {
            return Some("an empty, '.', or '..' component");
        }
        if names_git_dir(component) {
            return Some("a component a file system reads as '.git'");
        }
    }
    None
}

/// The first two paths, one the plan writes, that a file system which ignores case reads as one:
/// a path or a directory above it that differs only in letter case from another the plan writes or
/// one the working tree keeps.
pub fn case_collision<'a>(writes: &[&'a str], kept: &[&'a str]) -> Option<(&'a str, &'a str)> {
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    let prefixes = |path: &'a str| {
        path.match_indices('/')
            .map(move |(at, _)| &path[..at])
            .chain(std::iter::once(path))
    };
    for path in kept.iter().copied().flat_map(prefixes) {
        seen.entry(path.to_lowercase()).or_insert(path);
    }
    for path in writes.iter().copied().flat_map(prefixes) {
        match seen.get(&path.to_lowercase()) {
            Some(&other) if other != path => return Some((path, other)),
            Some(_) => {}
            None => {
                seen.insert(path.to_lowercase(), path);
            }
        }
    }
    None
}

/// The README with the old tag in its first Template release line replaced by the new one, and
/// no other byte changed; the README as it is where that line names the new tag already, and none
/// where it names neither or is no UTF-8.
pub fn move_release_line(readme: &[u8], old: Version, new: Version) -> Option<Vec<u8>> {
    const LABEL: &str = "**Template release:**";
    let (old, new) = (tag(old), tag(new));
    let mut start = 0;
    for line in readme.split_inclusive(|&b| b == b'\n') {
        // The first release line, as `releases` read it from the README read lossily; none where
        // that line is no UTF-8, so that no later line is written in its place.
        let lossy = String::from_utf8_lossy(line);
        if lossy.contains(LABEL) && docs_template_release::carried(&lossy).is_some() {
            let text = std::str::from_utf8(line).ok()?;
            let after = text.find(LABEL)? + LABEL.len();
            let rest = &text[after..];
            let names = |tag: &str| {
                rest.find(tag).filter(|&at| {
                    rest[..at]
                        .chars()
                        .all(|c| c.is_whitespace() || c == '[' || c == '`')
                })
            };
            if names(&new).is_some() {
                return Some(readme.to_vec());
            }
            let at = start + after + names(&old)?;
            let mut moved = readme[..at].to_vec();
            moved.extend_from_slice(new.as_bytes());
            moved.extend_from_slice(&readme[at + old.len()..]);
            return Some(moved);
        }
        start += line.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(object: &str) -> Entry {
        Entry {
            kind: Kind::File,
            object: object.to_owned(),
        }
    }

    fn tree(entries: &[(&str, Entry)]) -> Tree {
        entries
            .iter()
            .map(|(path, entry)| ((*path).to_owned(), entry.clone()))
            .collect()
    }

    const README: &str = "# P\n\n- **Template release:** v0.12.0 — the release of NUC.\n";

    #[test]
    fn the_releases_are_the_line_s_and_the_argument_s() {
        assert_eq!(
            releases(README, "v0.12.1"),
            Ok((Version(0, 12, 0), Version(0, 12, 1)))
        );
    }

    #[test]
    fn a_line_without_a_release_to_start_from_is_refused() {
        for readme in [
            "# P\n",
            "- **Template release:** unreleased\n",
            "- **Template release:** v0.12.0, forked\n",
        ] {
            assert_eq!(releases(readme, "v0.12.1").unwrap_err().reason, "release");
        }
    }

    #[test]
    fn a_release_before_v0_9_0_is_refused() {
        let readme = "- **Template release:** v0.8.1\n";
        assert_eq!(releases(readme, "v0.12.1").unwrap_err().reason, "old");
        let readme = "- **Template release:** v0.9.0\n";
        assert!(releases(readme, "v0.12.1").is_ok());
    }

    #[test]
    fn the_new_release_is_a_tag_above_the_old() {
        assert_eq!(releases(README, "v0.12.0").unwrap_err().reason, "order");
        assert_eq!(releases(README, "v0.11.9").unwrap_err().reason, "order");
        for argument in ["0.12.1", "v0.12", "v0.012.1", "v0.12.1-rc1", "main"] {
            assert_eq!(
                releases(README, argument).unwrap_err().reason,
                "usage",
                "{argument}"
            );
        }
    }

    #[test]
    fn a_refusal_ends_in_its_key() {
        let refusal = Refusal::new("main", "main is checked out");
        assert_eq!(
            refusal.to_string(),
            "main is checked out — not decided [upgrade/run:main]"
        );
    }

    #[test]
    fn the_base_is_agents_the_blob_list_and_the_nuc_adrs() {
        for path in [
            "AGENTS.md",
            "docs/adr/NUC.blobs",
            "docs/adr/NUC-base-is-fixed.md",
        ] {
            assert!(is_base(path), "{path}");
        }
        for path in [
            "docs/adr/README.md",
            "docs/adr/API-x.md",
            "docs/adr/NUC-x/y.md",
            "docs/adr/NUC-x.txt",
            "sub/AGENTS.md",
        ] {
            assert!(!is_base(path), "{path}");
        }
    }

    #[test]
    fn the_base_takes_the_new_release_s_entry_whatever_the_project_holds() {
        let old = tree(&[
            ("AGENTS.md", file("a1")),
            ("docs/adr/NUC-gone.md", file("g1")),
            ("docs/adr/NUC-same.md", file("s1")),
        ]);
        let new = tree(&[
            ("AGENTS.md", file("a2")),
            ("docs/adr/NUC-new.md", file("n2")),
            ("docs/adr/NUC-same.md", file("s1")),
        ]);
        let head = tree(&[
            ("AGENTS.md", file("edited")),
            ("docs/adr/NUC-gone.md", file("g1")),
            ("docs/adr/NUC-same.md", file("s1")),
            ("docs/adr/API-own.md", file("own")),
        ]);
        let plan = plan(&old, &new, &head, true);
        assert!(plan.judgments.is_empty());
        let expected: BTreeMap<String, Change> = [
            ("AGENTS.md", Change::Write(file("a2"))),
            ("docs/adr/NUC-gone.md", Change::Delete),
            ("docs/adr/NUC-new.md", Change::Write(file("n2"))),
        ]
        .into_iter()
        .map(|(path, change)| (path.to_owned(), change))
        .collect();
        assert_eq!(plan.changes, expected);
    }

    #[test]
    fn a_mode_the_new_release_changes_is_written() {
        let exec = Entry {
            kind: Kind::Executable,
            object: "a1".to_owned(),
        };
        let old = tree(&[("AGENTS.md", file("a1"))]);
        let new = tree(&[("AGENTS.md", exec.clone())]);
        assert_eq!(
            plan(&old, &new, &old, true).changes.get("AGENTS.md"),
            Some(&Change::Write(exec))
        );
        assert_eq!(plan(&old, &old, &old, true), Plan::default());
    }

    #[test]
    fn a_link_where_the_plan_would_write_is_a_judgment() {
        let link = Entry {
            kind: Kind::Link,
            object: "l".to_owned(),
        };
        let old = tree(&[("AGENTS.md", file("a1"))]);
        let new = tree(&[("AGENTS.md", file("a2"))]);
        let head = tree(&[("AGENTS.md", link.clone())]);
        let plan = plan(&old, &new, &head, true);
        assert!(plan.changes.is_empty());
        assert_eq!(plan.judgments.len(), 1);
        assert_eq!(plan.judgments[0].reason, LINK);
        assert_eq!(plan.judgments[0].file, "AGENTS.md");
        // The same link in every tree is left alone.
        let same = tree(&[("AGENTS.md", link)]);
        assert_eq!(super::plan(&same, &same, &same, true), Plan::default());
    }

    #[test]
    fn a_symlink_above_a_path_to_write_is_refused() {
        let new = tree(&[("docs/adr/NUC-new.md", file("n"))]);
        let mut plan = plan(&Tree::new(), &new, &Tree::new(), true);
        let found: BTreeMap<String, Found> = [("docs".to_owned(), Found::Link)].into();
        assert_eq!(
            settle(&mut plan, &Tree::new(), &found, true)
                .unwrap_err()
                .reason,
            "path"
        );
    }

    #[test]
    fn a_path_the_working_tree_keeps_of_its_own_is_a_judgment() {
        let new = tree(&[
            ("AGENTS.md", file("a")),
            ("docs/adr/NUC-dir.md", file("d")),
            ("docs/adr/NUC-link.md", file("l")),
        ]);
        let mut plan = plan(&Tree::new(), &new, &Tree::new(), true);
        assert_eq!(asked(&plan).len(), 5);
        let found: BTreeMap<String, Found> = [
            ("AGENTS.md".to_owned(), Found::Regular),
            ("docs".to_owned(), Found::Dir),
            ("docs/adr".to_owned(), Found::Dir),
            ("docs/adr/NUC-dir.md".to_owned(), Found::Dir),
            ("docs/adr/NUC-link.md".to_owned(), Found::Link),
        ]
        .into();
        settle(&mut plan, &Tree::new(), &found, true).unwrap();
        assert!(plan.changes.is_empty());
        let files: Vec<(&str, &str)> = plan
            .judgments
            .iter()
            .map(|j| (j.file.as_str(), j.reason))
            .collect();
        assert_eq!(
            files,
            [
                ("AGENTS.md", ADDED),
                ("docs/adr/NUC-dir.md", ADDED),
                ("docs/adr/NUC-link.md", ADDED)
            ]
        );
    }

    #[test]
    fn a_tracked_file_the_plan_replaces_is_no_judgment() {
        let old = tree(&[("AGENTS.md", file("a1"))]);
        let new = tree(&[("AGENTS.md", file("a2"))]);
        let mut plan = plan(&old, &new, &old, true);
        let found: BTreeMap<String, Found> = [("AGENTS.md".to_owned(), Found::Regular)].into();
        settle(&mut plan, &old, &found, true).unwrap();
        assert_eq!(plan.changes.len(), 1);
        assert!(plan.judgments.is_empty());
    }

    #[test]
    fn a_path_refused_on_some_platform_refuses_the_plan() {
        let new = tree(&[("docs/adr/NUC-a.md", file("a"))]);
        let head = tree(&[("Docs/x.md", file("x"))]);
        let plan = plan(&Tree::new(), &new, &head, true);
        assert_eq!(refuse_paths(&plan, &head).unwrap_err().reason, "path");
        let plan = super::plan(&Tree::new(), &new, &Tree::new(), true);
        assert_eq!(refuse_paths(&plan, &Tree::new()), Ok(()));
    }

    #[test]
    fn a_path_git_s_verify_path_refuses_is_refused() {
        for path in [
            "/etc/passwd",
            "a\\..\\..\\x",
            "C:x",
            "a//b",
            "./a",
            "a/../../x",
            ".git/config",
            "sub/.GIT/hooks/x",
            ".git. ./x",
            ".git::$INDEX_ALLOCATION/x",
            "GIT~1/config",
            ".g\u{200c}it/config",
            "a/",
            "docs/adr/NUC-a:b.md",
        ] {
            assert!(refused_path(path).is_some(), "{path}");
        }
        for path in [
            "AGENTS.md",
            "docs/adr/NUC-x.md",
            ".github/x.yml",
            ".gitignore",
        ] {
            assert_eq!(refused_path(path), None, "{path}");
        }
    }

    #[test]
    fn paths_that_differ_only_in_case_collide() {
        assert_eq!(
            case_collision(&["Docs/a.md"], &["docs/b.md"]),
            Some(("Docs", "docs"))
        );
        assert_eq!(
            case_collision(&["README.md", "readme.md"], &[]),
            Some(("readme.md", "README.md"))
        );
        assert_eq!(case_collision(&["docs/a.md"], &["docs/b.md"]), None);
        // The project's own pair, which the plan does not write, collides with nothing.
        assert_eq!(case_collision(&["x.md"], &["Readme.md", "README.md"]), None);
    }

    #[test]
    fn the_release_line_moves_and_nothing_else() {
        let (old, new) = (Version(0, 12, 0), Version(0, 12, 1));
        let readme = b"# P\n\ncaf\xe9 v0.12.0 is named here.\n\n- **Template release:** v0.12.0 \
                       \xe2\x80\x94 the release of NUC; see v0.12.0.\n";
        let moved = move_release_line(readme, old, new).unwrap();
        assert_eq!(
            moved,
            b"# P\n\ncaf\xe9 v0.12.0 is named here.\n\n- **Template release:** v0.12.1 \
              \xe2\x80\x94 the release of NUC; see v0.12.0.\n"
        );
        let linked = b"**Template release:** [v0.12.0](https://x.invalid)\n";
        assert_eq!(
            move_release_line(linked, old, new).unwrap(),
            b"**Template release:** [v0.12.1](https://x.invalid)\n"
        );
        let there = b"- **Template release:** v0.12.1\n";
        assert_eq!(move_release_line(there, old, new).unwrap(), there);
        assert_eq!(
            move_release_line(b"- **Template release:** v0.11.0\n", old, new),
            None
        );
        // A first release line that is no UTF-8 is not written, nor a later one in its place.
        assert_eq!(
            move_release_line(
                b"- **Template release:** v0.12.0 \xe9\n\n**Template release:** v0.12.0\n",
                old,
                new
            ),
            None
        );
        assert_eq!(
            move_release_line(b"- **Template release:** v0.12.0 \xe9\n", old, new),
            None
        );
    }

    #[test]
    fn a_mode_the_working_tree_cannot_carry_is_written_and_listed() {
        let exec = Entry {
            kind: Kind::Executable,
            object: "a1".to_owned(),
        };
        let old = tree(&[("AGENTS.md", file("a1"))]);
        let new = tree(&[("AGENTS.md", exec.clone())]);
        let plan = plan(&old, &new, &old, false);
        assert_eq!(plan.changes.get("AGENTS.md"), Some(&Change::Write(exec)));
        assert_eq!(plan.judgments.len(), 1);
        assert_eq!(plan.judgments[0].reason, MODE);
        assert!(plan.judgments[0].message.contains("--chmod=+x AGENTS.md"));
        // Content alone, the bit unchanged, needs no judgment.
        let changed = tree(&[("AGENTS.md", file("a2"))]);
        assert!(
            super::plan(&old, &changed, &old, false)
                .judgments
                .is_empty()
        );
    }

    #[test]
    fn a_rename_by_case_alone_is_no_judgment_where_the_file_system_ignores_case() {
        let old = tree(&[("docs/adr/NUC-a.md", file("a"))]);
        let new = tree(&[("docs/adr/NUC-A.md", file("a"))]);
        let mut plan = plan(&old, &new, &old, true);
        let found: BTreeMap<String, Found> = [
            ("docs".to_owned(), Found::Dir),
            ("docs/adr".to_owned(), Found::Dir),
            ("docs/adr/NUC-a.md".to_owned(), Found::Regular),
            ("docs/adr/NUC-A.md".to_owned(), Found::Regular),
        ]
        .into();
        let mut sensitive = plan.clone();
        settle(&mut plan, &old, &found, true).unwrap();
        assert!(plan.judgments.is_empty(), "{:?}", plan.judgments);
        assert_eq!(plan.changes.len(), 2);
        // Where the file system tells case apart, the file found is one of the working tree's own.
        settle(&mut sensitive, &old, &found, false).unwrap();
        assert_eq!(sensitive.judgments.len(), 1);
        assert_eq!(sensitive.judgments[0].reason, ADDED);
    }

    #[test]
    fn a_deletion_the_working_tree_has_no_file_for_is_dropped() {
        let old = tree(&[("docs/adr/NUC-gone.md", file("g"))]);
        let mut plan = plan(&old, &Tree::new(), &old, true);
        settle(&mut plan, &old, &BTreeMap::new(), true).unwrap();
        assert_eq!(plan, Plan::default());
        let mut plan = super::plan(&old, &Tree::new(), &old, true);
        let found: BTreeMap<String, Found> =
            [("docs/adr/NUC-gone.md".to_owned(), Found::Dir)].into();
        assert_eq!(
            settle(&mut plan, &old, &found, true).unwrap_err().reason,
            "path"
        );
    }
}
