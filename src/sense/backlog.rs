//! `nuc sense backlog`: the entries of the backlog under `docs/backlog/`, those on main and those a
//! local branch adds that main lacks, as the template's `scripts/sensor-backlog.sh` of the tree's
//! release reports them (NUC-large-work-is-planned §7). The shell asks git for the trees and the
//! branches; the sensor names each entry by its title and writes the report.

use super::Sensor;
use crate::bash;
use crate::config::Version;

/// The sensor, which the template brings at v0.8.0.
pub const SENSOR: Sensor = Sensor {
    name: "backlog",
    since: Version(0, 8, 0),
    changes: &[],
    reasons: &["git", "name", super::ENVIRONMENT, super::RELEASE],
};

/// The directory of the backlog.
pub const DIR: &str = "docs/backlog";

/// The candidates for main, in the order the bash tries them.
pub const MAINS: [&str; 3] = ["origin/main", "main", "HEAD"];

/// The report where the repository has no commit yet.
pub fn no_commit() -> Vec<u8> {
    b"Backlog sensor: no commit yet \xe2\x80\x94 nothing to report.\n".to_vec()
}

/// Whether the path, as git lists it, names an entry: a Markdown file.
pub fn is_entry(path: &[u8]) -> bool {
    path.ends_with(b".md")
}

/// The title of an entry: the first `# ` heading of its text as git shows it, or its file name
/// where it has none or git could not show it. The bash's command substitution drops each NUL byte.
pub fn title(shown: Option<&[u8]>, path: &[u8]) -> Vec<u8> {
    let heading =
        shown.and_then(|text| bash::sed_lines(text).find_map(|line| line.strip_prefix(b"# ")));
    match heading.map(|h| h.iter().copied().filter(|&b| b != 0).collect::<Vec<u8>>()) {
        Some(title) if !title.is_empty() => title,
        _ => match path.iter().rposition(|&b| b == b'/') {
            Some(at) => path[at + 1..].to_vec(),
            None => path.to_vec(),
        },
    }
}

/// An entry of the report: its path, its title, and the branch it is only on, none for main.
pub struct Entry {
    pub path: Vec<u8>,
    pub title: Vec<u8>,
    pub branch: Option<Vec<u8>>,
}

/// The report of the entries, those on `main` first.
pub fn report(main: &str, entries: &[Entry]) -> Vec<u8> {
    if entries.is_empty() {
        return format!("Backlog sensor: no entries under {DIR}/ — nothing to report.\n")
            .into_bytes();
    }
    let on_branch = entries.iter().filter(|e| e.branch.is_some()).count();
    let mut out = format!(
        "Backlog sensor: {} on {main}, {on_branch} only on unmerged local branches.\n",
        entries.len() - on_branch
    )
    .into_bytes();
    for entry in entries {
        out.extend_from_slice(b"- ");
        out.extend_from_slice(&entry.path);
        out.extend_from_slice(" — ".as_bytes());
        out.extend_from_slice(&entry.title);
        if let Some(branch) = &entry.branch {
            out.extend_from_slice(b" (on ");
            out.extend_from_slice(branch);
            out.push(b')');
        }
        out.push(b'\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_title_is_the_first_heading_or_the_file_name() {
        assert_eq!(
            title(Some(b"x\n# One\n# Two\n"), b"docs/backlog/a.md"),
            b"One"
        );
        assert_eq!(title(Some(b"no heading"), b"docs/backlog/a.md"), b"a.md");
        assert_eq!(title(Some(b"# \n"), b"docs/backlog/a.md"), b"a.md");
        assert_eq!(title(None, b"docs/backlog/b.md"), b"b.md");
        assert_eq!(title(Some(b"#  Spaced\r\n"), b"x.md"), b" Spaced\r");
    }

    #[test]
    fn the_report_counts_main_and_the_branches() {
        let entries = [
            Entry {
                path: b"docs/backlog/a.md".to_vec(),
                title: b"A".to_vec(),
                branch: None,
            },
            Entry {
                path: b"docs/backlog/b.md".to_vec(),
                title: b"B".to_vec(),
                branch: Some(b"feat/x".to_vec()),
            },
        ];
        assert_eq!(
            String::from_utf8(report("origin/main", &entries)).unwrap(),
            "Backlog sensor: 1 on origin/main, 1 only on unmerged local branches.\n- docs/backlog/a.md — A\n- docs/backlog/b.md — B (on feat/x)\n"
        );
        assert_eq!(
            String::from_utf8(report("HEAD", &[])).unwrap(),
            "Backlog sensor: no entries under docs/backlog/ — nothing to report.\n"
        );
    }
}
