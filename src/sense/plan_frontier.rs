//! `nuc sense plan-frontier`: the open tickets under `docs/plan/`, sorted into the frontier, the
//! claimed, and the blocked, as the template's `scripts/sensor-plan-frontier.sh` of the tree's
//! release reports them (NUC-large-work-is-planned §3, §4). The shell lists the tickets as `find`
//! does, reads them, asks git for the ticket branches, and tests which blockers are still files;
//! the sensor reads each ticket's sections as the bash's awk and grep do, under `LC_ALL=C`, and
//! writes the report.

use std::collections::BTreeSet;

use regex::bytes::{Regex, RegexBuilder};

use super::{Refusal, Sensor};
use crate::bash;
use crate::config::Version;

/// The sensor, from the release `nuc` ports first: it reports alike in every release from it on.
pub const SENSOR: Sensor = Sensor {
    name: "plan-frontier",
    since: Version(0, 5, 0),
    changes: &[],
    reasons: &["ticket", "git", "file", super::ENVIRONMENT, super::RELEASE],
};

/// The directory of the plan.
pub const PLAN: &str = "docs/plan";

/// An expression as grep reads it in the C locale: bytes, not characters.
fn expression(text: &str) -> Regex {
    RegexBuilder::new(text)
        .unicode(false)
        .build()
        .expect("a valid expression")
}

/// Whether the name of a file in a spec's directory is one `find -name '[0-9][0-9]-*.md'` lists.
pub fn is_ticket_name(name: &[u8]) -> bool {
    name.len() >= 6
        && name[0].is_ascii_digit()
        && name[1].is_ascii_digit()
        && name[2] == b'-'
        && name.ends_with(b".md")
}

/// The lines of the `## <heading>` section, HTML comments removed, as the bash's awk prints them;
/// none where the file has no such section. A comment left open stays open into the next line the
/// section holds, wherever it is.
pub fn section(text: &[u8], heading: &str) -> Option<Vec<Vec<u8>>> {
    let want = format!("## {heading}");
    let (mut inside, mut found, mut comment) = (false, false, false);
    let mut lines = Vec::new();
    for line in bash::awk_lines(text) {
        if line.starts_with(b"## ") {
            inside = line == want.as_bytes();
            found |= inside;
            continue;
        }
        if !inside {
            continue;
        }
        let mut rest: &[u8] = line;
        let mut out = Vec::new();
        while !rest.is_empty() {
            if comment {
                let Some(end) = bash::find(rest, b"-->") else {
                    break;
                };
                rest = &rest[end + 3..];
                comment = false;
            } else {
                let Some(start) = bash::find(rest, b"<!--") else {
                    out.extend_from_slice(rest);
                    break;
                };
                out.extend_from_slice(&rest[..start]);
                rest = &rest[start + 4..];
                comment = true;
            }
        }
        lines.push(out);
    }
    found.then_some(lines)
}

/// A ticket read: its path, the branch that claims it, the first word of its mode, and the paths
/// its `Blocked by` section names, sorted and once each.
#[derive(Debug, PartialEq, Eq)]
pub struct Ticket {
    pub path: String,
    pub branch: String,
    pub mode: &'static str,
    pub blockers: Vec<Vec<u8>>,
}

/// The ticket at the path, from its text; or why the sensor cannot place it, as the bash words it.
pub fn read(path: &str, text: &[u8]) -> Result<Ticket, String> {
    let name = expression(r"^docs/plan/[A-Za-z0-9._-]+/[0-9][0-9]-[A-Za-z0-9._-]+\.md$");
    if !name.is_match(path.as_bytes()) {
        return Err(format!(
            "{path}: a name the claim refuses — use letters, digits, '.', '_', and '-' only, as docs/plan/<spec>/<NN>-<slug>.md"
        ));
    }
    let branch = format!(
        "ticket/{}",
        path.strip_prefix("docs/plan/")
            .and_then(|rest| rest.strip_suffix(".md"))
            .unwrap_or(path)
    );
    let blockers = section(text, "Blocked by").ok_or_else(|| {
        format!(
            "{path}: no '## Blocked by' section — name the blocking tickets by path, or 'None (can start immediately)'"
        )
    })?;
    let fine = expression(
        r"^[ \t\n\x0B\x0C\r]*([-*][ \t\n\x0B\x0C\r]+)?None([^A-Za-z]|$)|docs/plan/[A-Za-z0-9._/-]+\.md|^[ \t\n\x0B\x0C\r]*$",
    );
    // The here-string the bash hands grep ends the last line, so an empty section is one empty
    // line, and a line holds no newline.
    let lines = here_string_lines(&blockers);
    if let Some(odd) = lines.iter().find(|line| !fine.is_match(line)) {
        let start = odd
            .iter()
            .position(|&b| !bash::is_space(b))
            .unwrap_or(odd.len());
        return Err(format!(
            "{path}: '## Blocked by' names '{}', which is no path under {PLAN}/ — write each blocker as its path, or 'None (can start immediately)'",
            String::from_utf8_lossy(&odd[start..])
        ));
    }
    let mode_lines = section(text, "Mode").ok_or_else(|| {
        format!("{path}: no '## Mode' section — say 'unattended' or 'with the human', and why")
    })?;
    let first = here_string_lines(&mode_lines)
        .into_iter()
        .find(|line| line.iter().any(|&b| !matches!(b, b' ' | b'\t')));
    let mode = match first {
        Some(line) if line.starts_with(b"unattended") => "unattended",
        _ => "with the human",
    };
    let named = expression(r"docs/plan/[A-Za-z0-9._/-]+\.md");
    let blockers: BTreeSet<Vec<u8>> = lines
        .iter()
        .flat_map(|line| named.find_iter(line).map(|m| m.as_bytes().to_vec()))
        .collect();
    Ok(Ticket {
        path: path.to_owned(),
        branch,
        mode,
        blockers: blockers.into_iter().collect(),
    })
}

/// The lines grep reads from `<<< "$(section …)"`: the section's lines joined, trailing newlines
/// dropped by the substitution, and one newline added by the here-string.
fn here_string_lines(lines: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut joined = lines.join(&b'\n');
    while joined.last() == Some(&b'\n') {
        joined.pop();
    }
    joined.split(|&b| b == b'\n').map(<[u8]>::to_vec).collect()
}

/// The ticket branches among the refs `git for-each-ref --format=%(refname)` lists under
/// `refs/heads/ticket` and `refs/remotes`, as the bash's sed reads them.
pub fn claims(refs: &[u8]) -> BTreeSet<Vec<u8>> {
    bash::sed_lines(refs)
        .filter_map(|line| {
            if let Some(branch) = line.strip_prefix(b"refs/heads/") {
                return branch.starts_with(b"ticket/").then(|| branch.to_vec());
            }
            let remote = line.strip_prefix(b"refs/remotes/")?;
            let slash = remote.iter().position(|&b| b == b'/')?;
            let branch = &remote[slash + 1..];
            branch.starts_with(b"ticket/").then(|| branch.to_vec())
        })
        .collect()
}

/// A ticket placed: claimed, or blocked by the blockers still open, or on the frontier.
pub enum Place {
    Claimed,
    Blocked(Vec<Vec<u8>>),
    Frontier,
}

/// The report of the tickets, each with its place; or, where any could not be read, the refusal
/// that names them, where the bash exits 1.
pub fn report(
    count: usize,
    placed: &[(Ticket, Place)],
    errors: &[String],
) -> Result<Vec<u8>, Refusal> {
    if count == 0 {
        return Ok(format!("Plan frontier sensor: no open tickets under {PLAN}/.\n").into_bytes());
    }
    if !errors.is_empty() {
        return Err(Refusal {
            sensor: SENSOR.name,
            reason: "ticket",
            message: format!(
                "{} ticket(s) unreadable (NUC-large-work-is-planned §3, §4): {}",
                errors.len(),
                errors.join("; ")
            ),
        });
    }
    let (mut frontier, mut claimed, mut blocked) = (Vec::new(), Vec::new(), Vec::new());
    for (ticket, place) in placed {
        match place {
            Place::Frontier => {
                frontier.push(format!("{} — {}", ticket.path, ticket.mode).into_bytes())
            }
            Place::Claimed => {
                claimed.push(format!("{} — {}", ticket.path, ticket.branch).into_bytes())
            }
            Place::Blocked(open) => {
                let mut line = format!("{} — by ", ticket.path).into_bytes();
                line.extend_from_slice(&open.join(&b' '));
                blocked.push(line);
            }
        }
    }
    let mut out = format!(
        "Plan frontier sensor: {count} open, {} on the frontier, {} claimed, {} blocked.\n",
        frontier.len(),
        claimed.len(),
        blocked.len()
    )
    .into_bytes();
    for (heading, items) in [
        ("Frontier (claim by creating the branch):", &frontier),
        ("Claimed:", &claimed),
        ("Blocked:", &blocked),
    ] {
        if items.is_empty() {
            continue;
        }
        out.extend_from_slice(format!("  {heading}\n").as_bytes());
        for item in items {
            out.extend_from_slice(b"  - ");
            out.extend_from_slice(item);
            out.push(b'\n');
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TICKET: &str = "docs/plan/s/01-a.md";

    #[test]
    fn a_section_drops_its_comments_and_a_comment_left_open_stays_open() {
        let text = b"## Blocked by\n<!-- x\ny --> - docs/plan/s/02-b.md\n## Mode\nwith <!-- a -->the human\n";
        assert_eq!(
            section(text, "Blocked by").unwrap(),
            vec![b"".to_vec(), b" - docs/plan/s/02-b.md".to_vec()]
        );
        assert_eq!(
            section(text, "Mode").unwrap(),
            vec![b"with the human".to_vec()]
        );
        assert_eq!(section(text, "Read first"), None);
    }

    #[test]
    fn a_ticket_reads_its_blockers_and_its_mode() {
        let text = b"## Blocked by\n\n- docs/plan/s/03-c.md and docs/plan/s/02-b.md\n\n## Mode\n\nunattended: yes\n";
        let ticket = read(TICKET, text).unwrap();
        assert_eq!(ticket.branch, "ticket/s/01-a");
        assert_eq!(ticket.mode, "unattended");
        assert_eq!(
            ticket.blockers,
            vec![
                b"docs/plan/s/02-b.md".to_vec(),
                b"docs/plan/s/03-c.md".to_vec()
            ]
        );
        let none = read(
            TICKET,
            b"## Blocked by\n- None (can start immediately)\n## Mode\n  unattended\n",
        )
        .unwrap();
        assert!(none.blockers.is_empty());
        assert_eq!(none.mode, "with the human");
    }

    #[test]
    fn an_unreadable_ticket_names_why() {
        assert!(
            read("docs/plan/s/01-a b.md", b"")
                .unwrap_err()
                .contains("a name the claim refuses")
        );
        assert!(
            read(TICKET, b"## Mode\nx\n")
                .unwrap_err()
                .contains("no '## Blocked by'")
        );
        let odd = read(TICKET, b"## Blocked by\n  [b](02-b.md)\n## Mode\nx\n").unwrap_err();
        assert!(odd.contains("names '[b](02-b.md)'"), "{odd}");
        assert!(
            read(TICKET, b"## Blocked by\nNone\n")
                .unwrap_err()
                .contains("no '## Mode'")
        );
    }

    #[test]
    fn claims_are_the_ticket_branches_local_or_remote() {
        let refs = b"refs/heads/ticket/s/01-a\nrefs/heads/main\nrefs/remotes/origin/ticket/s/02-b\nrefs/remotes/origin/main\n";
        let claims = claims(refs);
        assert!(claims.contains(b"ticket/s/01-a".as_slice()));
        assert!(claims.contains(b"ticket/s/02-b".as_slice()));
        assert_eq!(claims.len(), 2);
    }
}
