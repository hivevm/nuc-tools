//! `nuc sense review-binding`: the commits of a change and the review that binds them, as the
//! template's `scripts/sensor-review-binding.sh` of the tree's release reports them. The line
//! `Reviewed at: <commit>` of a hand-off is held against the commits between a base and a head:
//! before v0.11.0 the hand-off of the branch first and then that of each merge on its first-parent
//! line, from v0.11.0 the hand-off of the branch alone, its merges listed. Every commit that
//! changes the text of an ADR together with another path is listed too. The sensor asks the
//! repository through [`Repo`], which the shell answers with git, in the order the bash asks; it
//! reads the answers as the bash's grep, sed, awk, and `[[ ]]` do, under `LC_ALL=C`.

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

/// The sensor. The releases before v0.5.0 read no draft ADR, so `nuc` reports from v0.5.0 on.
pub const SENSOR: Sensor = Sensor {
    name: "review-binding",
    since: Version(0, 5, 0),
    changes: &[BELOW, NAMED, ARGUMENTS, LINEAR],
    reasons: &["ref", "git", "file", super::ENVIRONMENT, super::RELEASE],
};

/// The label that opens each headline of the report.
pub const LABEL: &str = "Review binding sensor";

/// The release from which a headline counts the findings listed under it.
pub const BELOW: Version = Version(0, 7, 0);

/// The release from which a comma ends the commit a `Reviewed at:` line names, the record is named
/// by category, and the headline of the mixed commits cites the clause of a `NUC-` ADR.
pub const NAMED: Version = Version(0, 9, 0);

/// The release from which an argument past the base and the head is refused, where the releases
/// before ignore it.
pub const ARGUMENTS: Version = Version(0, 10, 0);

/// The release from which the branch integrates linearly: the hand-off of the branch alone binds,
/// and a merge between the base and the head is listed, its message not read.
pub const LINEAR: Version = Version(0, 11, 0);

/// The arguments as the bash of the release reads them.
#[derive(Debug, PartialEq, Eq)]
pub struct Args<'a> {
    /// The path `--hand-off` names, from where `nuc` runs: none where `--hand-off` is not given,
    /// and empty where it is given without a value or with an empty one.
    pub hand_off: Option<&'a str>,
    pub base: Option<&'a str>,
    pub head: Option<&'a str>,
}

/// The arguments as the bash of the release reads them: `--hand-off <file>` first, then the base
/// and the head; none where `nuc sense` refuses them: `--root` wherever it stands
/// (API-sensor-cores-ask-the-shell), and what [`super::positional`] refuses, one rule for every
/// sensor that takes a base and a head.
pub fn parse<'a>(args: &[&'a str], release: Version) -> Option<Args<'a>> {
    if args.contains(&"--root") {
        return None;
    }
    let (hand_off, rest) = match args {
        ["--hand-off", path, rest @ ..] => (Some(*path), rest),
        ["--hand-off"] => (Some(""), &[][..]),
        rest => (None, rest),
    };
    let positional = super::positional(rest, 2, release >= ARGUMENTS)?;
    Some(Args {
        hand_off,
        base: positional.first().copied(),
        head: positional.get(1).copied(),
    })
}

/// What the sensor asks the repository, each as the bash asks git; the shell answers. A revision is
/// passed as the bash passes it, such as `<merge>^2`.
pub trait Repo {
    /// The commit `git rev-parse --verify -q <rev>^{commit}` names, none where it fails.
    fn commit(&mut self, rev: &str) -> Result<Option<String>, Refusal>;
    /// The object `git rev-parse <rev>` names.
    fn full(&mut self, rev: &str) -> Result<String, Refusal>;
    /// The short hash `git rev-parse --short <rev>` gives.
    fn short(&mut self, rev: &str) -> Result<String, Refusal>;
    /// Whether `git merge-base --is-ancestor <ancestor> <rev>` exits 0.
    fn is_ancestor(&mut self, ancestor: &str, rev: &str) -> Result<bool, Refusal>;
    /// The lines of `git rev-list <options> <range>`.
    fn rev_list(&mut self, options: &[&str], range: &str) -> Result<Vec<Vec<u8>>, Refusal>;
    /// `git rev-list --count <options> <range>`.
    fn count(&mut self, options: &[&str], range: &str) -> Result<u64, Refusal>;
    /// The lines of `git log --format='%h %s' <range>`.
    fn subjects(&mut self, range: &str) -> Result<Vec<Vec<u8>>, Refusal>;
    /// `git log -1 --format='%h %s' <commit>`, as a command substitution gives it.
    fn subject(&mut self, commit: &str) -> Result<Vec<u8>, Refusal>;
    /// `git log -1 --format=%B <commit>`, the message of a merge.
    fn message(&mut self, commit: &str) -> Result<Vec<u8>, Refusal>;
    /// The lines of `git diff --stat <from> <to>`.
    fn stat(&mut self, from: &str, to: &str) -> Result<Vec<Vec<u8>>, Refusal>;
    /// `git diff-tree -r --root --no-commit-id -p -U0 --no-color <commit> -- <pathspecs>`.
    fn patch(&mut self, commit: &str, pathspecs: &[&str]) -> Result<Vec<u8>, Refusal>;
    /// The lines of `git diff-tree -r --root --no-commit-id --name-only <commit> -- . <exclude>`,
    /// the paths outside `docs/adr/`.
    fn outside_record(&mut self, commit: &str) -> Result<Vec<Vec<u8>>, Refusal>;
    /// The hand-off, read only where the bash reads it, as `$(cat "$hand_off")` gives it.
    fn hand_off(&mut self) -> Result<Vec<u8>, Refusal>;
}

/// The pathspecs of the ADR files whose text a commit can change, as the release's glob names them.
fn adr_pathspecs(release: Version) -> Vec<String> {
    adr::name_patterns(release >= NAMED)
        .iter()
        .map(|pattern| format!(":(glob)docs/adr/{pattern}"))
        .collect()
}

/// The pathspecs of the paths outside the record, the index under `docs/adr/` among those left out.
pub const OUTSIDE_RECORD: [&str; 2] = [".", ":(exclude)docs/adr/"];

/// The commit a message's first `Reviewed at:` line names, as grep finds the line and sed cuts the
/// commit out of it: the word after the colon, up to a space, or from `NAMED` on a comma, its
/// backticks dropped. None where no line names one.
fn reviewed(message: &[u8], release: Version) -> Option<Vec<u8>> {
    let pattern =
        regex!(r"(?-u)^[[:space:]]*([-*][[:space:]]+)?Reviewed at:[[:space:]]*[^[:space:]]");
    let line = bash::sed_lines(message).find(|line| pattern.is_match(line))?;
    let colon = line
        .iter()
        .position(|&b| b == b':')
        .expect("the line has its colon");
    let rest = &line[colon + 1..];
    let rest = &rest[rest
        .iter()
        .position(|&b| !bash::is_space(b))
        .unwrap_or(rest.len())..];
    let ends = |b: u8| bash::is_space(b) || (release >= NAMED && b == b',');
    let word = &rest[..rest.iter().position(|&b| ends(b)).unwrap_or(rest.len())];
    let word: Vec<u8> = word.iter().copied().filter(|&b| b != b'`').collect();
    (!word.is_empty()).then_some(word)
}

/// The commit as a hash, where it reads as one, `^[0-9a-fA-F]{7,40}$`: a name that is none, such
/// as a branch's or `HEAD~1`, resolves to wherever it points now, not to what the review saw.
fn as_hash(word: &[u8]) -> Option<&str> {
    let hash = (7..=40).contains(&word.len()) && word.iter().all(u8::is_ascii_hexdigit);
    hash.then(|| std::str::from_utf8(word).expect("hex digits are ASCII"))
}

/// What a `Reviewed at:` line binds among the commits after one up to a tip.
#[derive(Debug, PartialEq, Eq)]
pub enum Binding {
    /// No line names a commit.
    Unnamed,
    /// The line names what is no commit of them, as written.
    Elsewhere(Vec<u8>),
    /// The review saw the tip.
    AtTip,
    /// Commits landed after the review: their count, the short hash of the reviewed one, and the
    /// lines of their log and of the diffstat they add up to.
    After {
        count: u64,
        short: String,
        log: Vec<Vec<u8>>,
        stat: Vec<Vec<u8>>,
    },
}

/// What the message's `Reviewed at:` line binds among the commits after `before` up to `tip`, as
/// `bind` of the bash asks git: a hash only, that names a commit the tip has and `before` has not,
/// and whose ref git does not prefer to it.
fn bind(
    repo: &mut dyn Repo,
    message: &[u8],
    tip: &str,
    before: &str,
    release: Version,
) -> Result<Binding, Refusal> {
    let Some(word) = reviewed(message, release) else {
        return Ok(Binding::Unnamed);
    };
    let mut sha = None;
    if let Some(hash) = as_hash(&word) {
        sha = repo
            .commit(hash)?
            .filter(|sha| sha.starts_with(&hash.to_ascii_lowercase()));
    }
    let Some(sha) = sha else {
        return Ok(Binding::Elsewhere(word));
    };
    if !repo.is_ancestor(&sha, tip)? || repo.is_ancestor(&sha, before)? {
        return Ok(Binding::Elsewhere(word));
    }
    if sha == repo.full(tip)? {
        return Ok(Binding::AtTip);
    }
    let range = format!("{sha}..{tip}");
    Ok(Binding::After {
        count: repo.count(&[], &range)?,
        short: repo.short(&sha)?,
        log: repo.subjects(&range)?,
        stat: repo.stat(&sha, tip)?,
    })
}

/// `s` where the count is not one, as the bash's `$( ((n == 1)) || echo s)`.
fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

/// The finding of a binding under its label, `what` naming the commits; none for a review at the
/// tip.
fn finding(label: &[u8], binding: &Binding, what: &str) -> Option<Vec<u8>> {
    let mut out = label.to_vec();
    match binding {
        Binding::AtTip => return None,
        Binding::Unnamed => out.extend_from_slice(
            ": no 'Reviewed at:' commit in its message — the review is bound to nothing".as_bytes(),
        ),
        Binding::Elsewhere(word) => {
            out.extend_from_slice(b": reviewed at '");
            out.extend_from_slice(word);
            out.extend_from_slice(format!("', which is no commit of {what}").as_bytes());
        }
        Binding::After {
            count,
            short,
            log,
            stat,
        } => {
            let n = usize::try_from(*count).unwrap_or(usize::MAX);
            out.extend_from_slice(
                format!(
                    ": {count} commit{} after the review at {short}, unreviewed:",
                    plural(n)
                )
                .as_bytes(),
            );
            for line in log {
                out.extend_from_slice(b"\n    ");
                out.extend_from_slice(line);
            }
            for line in stat {
                out.extend_from_slice(b"\n   ");
                out.extend_from_slice(line);
            }
        }
    }
    Some(out)
}

/// The ADR files whose text a commit's patch changes, as the bash's awk reads it: the last field of
/// each `diff --git` line, its `b/` dropped, once each in the order they come, where a line it
/// adds or removes is no Status, Date, or Deciders line.
fn adr_texts(patch: &[u8]) -> Vec<Vec<u8>> {
    let flip = regex!(r"(?-u)^[-+]- [*][*](Status|Date|Deciders):[*][*]");
    let mut file: &[u8] = b"";
    let mut seen: Vec<Vec<u8>> = Vec::new();
    for line in bash::awk_lines(patch) {
        if line.starts_with(b"diff --git ") {
            let last = bash::awk_fields(line).last().copied().unwrap_or_default();
            file = last.strip_prefix(b"b/").unwrap_or(last);
            continue;
        }
        if line.starts_with(b"+++ ") || line.starts_with(b"--- ") {
            continue;
        }
        let changed = line.first().is_some_and(|&b| b == b'-' || b == b'+');
        if changed && !flip.is_match(line) && !seen.iter().any(|known| known == file) {
            seen.push(file.to_vec());
        }
    }
    seen
}

/// The entry of a commit that changes the text of the ADRs together with the paths outside the
/// record: its `%h %s`, the ADRs, and the first three paths, with a count of the rest.
fn mixed_entry(subject: &[u8], texts: &[Vec<u8>], others: &[Vec<u8>]) -> Vec<u8> {
    let mut out = subject.to_vec();
    out.extend_from_slice(b": changes the text of ");
    out.extend_from_slice(&texts.join(&b' '));
    out.extend_from_slice(b" together with ");
    out.extend_from_slice(&others[..others.len().min(3)].join(&b' '));
    if others.len() > 3 {
        out.extend_from_slice(format!(" and {} more", others.len() - 3).as_bytes());
    }
    out
}

/// The commits between the base and the head, merges left out, that change the text of an ADR
/// together with a path outside the record, each as its entry.
fn mixed(repo: &mut dyn Repo, range: &str, release: Version) -> Result<Vec<Vec<u8>>, Refusal> {
    let mut entries = Vec::new();
    let pathspecs = adr_pathspecs(release);
    let pathspecs: Vec<&str> = pathspecs.iter().map(String::as_str).collect();
    for commit in repo.rev_list(&["--no-merges", "--reverse"], range)? {
        let commit = String::from_utf8_lossy(&commit).into_owned();
        let texts = adr_texts(&repo.patch(&commit, &pathspecs)?);
        if texts.is_empty() {
            continue;
        }
        let others = repo.outside_record(&commit)?;
        if others.is_empty() {
            continue;
        }
        entries.push(mixed_entry(&repo.subject(&commit)?, &texts, &others));
    }
    Ok(entries)
}

/// The commits that mix ADR text with other paths, under their headline; nothing where there are
/// none.
fn report_mixed(out: &mut Vec<u8>, mixed: &[Vec<u8>], release: Version) {
    if mixed.is_empty() {
        return;
    }
    let rule = if release >= NAMED {
        "NUC-decisions-are-proposed-then-accepted §4"
    } else {
        "AGENTS.md section 3"
    };
    out.extend_from_slice(
        format!(
            "{LABEL}: {} commit{} mixing ADR text with other paths, which {rule} keeps in commits \
             apart:\n",
            mixed.len(),
            plural(mixed.len())
        )
        .as_bytes(),
    );
    list(out, mixed);
}

/// Each entry as an item under a headline.
fn list(out: &mut Vec<u8>, entries: &[Vec<u8>]) {
    for entry in entries {
        out.extend_from_slice(b"  - ");
        out.extend_from_slice(entry);
        out.push(b'\n');
    }
}

/// What a headline says of the findings listed under it, the mixed commits counted as one, from
/// `BELOW` on; nothing before, or where there are none.
fn below(findings: usize, mixed: &[Vec<u8>], release: Version) -> String {
    let k = findings + usize::from(!mixed.is_empty());
    if release < BELOW || k == 0 {
        return String::new();
    }
    format!("; {k} finding{} below", plural(k))
}

/// The report on the change from `base` to `head`, the hand-off given or not, as the bash of the
/// release writes it.
pub fn report(
    repo: &mut dyn Repo,
    base: &str,
    head: &str,
    hand_off: bool,
    release: Version,
) -> Result<Vec<u8>, Refusal> {
    let range = format!("{base}..{head}");
    let mixed = mixed(repo, &range, release)?;
    if release >= LINEAR {
        linear(repo, base, head, hand_off, &mixed, release)
    } else {
        merged(repo, base, head, hand_off, &mixed, release)
    }
}

/// The report of the releases before `LINEAR`: the hand-off of the branch first, reviewed at the
/// head covering everything; else each merge on the first-parent line that lands a branch, held
/// against its branch, and the commits that landed outside a merge.
fn merged(
    repo: &mut dyn Repo,
    base: &str,
    head: &str,
    hand_off: bool,
    mixed: &[Vec<u8>],
    release: Version,
) -> Result<Vec<u8>, Refusal> {
    let range = format!("{base}..{head}");
    let mut merges = Vec::new();
    for merge in repo.rev_list(&["--first-parent", "--merges", "--reverse"], &range)? {
        let merge = String::from_utf8_lossy(&merge).into_owned();
        if !repo.is_ancestor(&format!("{merge}^2"), base)? {
            merges.push(merge);
        }
    }
    let linear = repo.rev_list(&["--first-parent", "--no-merges", "--reverse"], &range)?;
    let mut findings = Vec::new();
    let mut hand_off_binding = None;
    if hand_off {
        let message = repo.hand_off()?;
        let binding = bind(repo, &message, head, base, release)?;
        if binding == Binding::AtTip {
            let covered = if merges.is_empty() {
                String::new()
            } else {
                format!(
                    ", which covers the {} merge{} it carries",
                    merges.len(),
                    plural(merges.len())
                )
            };
            let mut out = format!(
                "{LABEL}: the hand-off was reviewed at the head, {}{covered}{}.\n",
                repo.short(head)?,
                below(0, mixed, release)
            )
            .into_bytes();
            report_mixed(&mut out, mixed, release);
            return Ok(out);
        }
        findings.extend(finding(
            b"the hand-off",
            &binding,
            "the branch since the base",
        ));
        hand_off_binding = Some(binding);
    }
    let mut at_tip = 0;
    for merge in &merges {
        let label = repo.subject(merge)?;
        let message = repo.message(merge)?;
        let binding = bind(
            repo,
            &message,
            &format!("{merge}^2"),
            &format!("{merge}^1"),
            release,
        )?;
        if binding == Binding::AtTip {
            at_tip += 1;
        }
        findings.extend(finding(&label, &binding, "the merged branch"));
    }
    // A hand-off reviewed at a commit of the branch has listed what came after it, the commits
    // outside a merge among them.
    let listed = matches!(hand_off_binding, Some(Binding::After { .. }));
    if !linear.is_empty() && !listed {
        let mut entry = format!(
            "{} commit{} landed outside a merge, named by no review:",
            linear.len(),
            plural(linear.len())
        )
        .into_bytes();
        for commit in &linear {
            entry.extend_from_slice(b"\n    ");
            entry.extend_from_slice(&repo.subject(&String::from_utf8_lossy(commit))?);
        }
        findings.push(entry);
    }
    let since = repo.short(base)?;
    let below = below(findings.len(), mixed, release);
    let mut out = if merges.is_empty() {
        let nothing = if findings.is_empty() && mixed.is_empty() {
            " — nothing to read"
        } else {
            ""
        };
        format!("{LABEL}: no merge since {since}{nothing}{below}.\n")
    } else {
        format!(
            "{LABEL}: {} merge{} since {since}, {at_tip} reviewed at the tip of its branch{below}.\n",
            merges.len(),
            plural(merges.len())
        )
    }
    .into_bytes();
    list(&mut out, &findings);
    report_mixed(&mut out, mixed, release);
    Ok(out)
}

/// The report from `LINEAR` on: the merges between the base and the head listed, and the commits
/// bound by the hand-off of the branch alone.
fn linear(
    repo: &mut dyn Repo,
    base: &str,
    head: &str,
    hand_off: bool,
    mixed: &[Vec<u8>],
    release: Version,
) -> Result<Vec<u8>, Refusal> {
    let range = format!("{base}..{head}");
    let merges = repo.rev_list(&["--merges", "--reverse"], &range)?;
    let n = repo.count(&["--no-merges"], &range)?;
    let commits = format!(
        "{n} commit{}",
        plural(usize::try_from(n).unwrap_or(usize::MAX))
    );
    let since = repo.short(base)?;
    let mut findings = Vec::new();
    if !merges.is_empty() {
        let one = merges.len() == 1;
        let mut entry = format!(
            "{} merge{} between the base and the head; a feat/ branch carries none \
             (NUC-main-only-via-pr §5), and no review is read from {} message{}:",
            merges.len(),
            plural(merges.len()),
            if one { "its" } else { "their" },
            plural(merges.len())
        )
        .into_bytes();
        for merge in &merges {
            entry.extend_from_slice(b"\n    ");
            entry.extend_from_slice(&repo.subject(&String::from_utf8_lossy(merge))?);
        }
        findings.push(entry);
    }
    if n == 0 && merges.is_empty() {
        return Ok(format!("{LABEL}: no commit since {since} — nothing to read.\n").into_bytes());
    }
    let headline = if hand_off {
        let message = repo.hand_off()?;
        let binding = bind(repo, &message, head, base, release)?;
        if binding == Binding::AtTip {
            format!(
                "{LABEL}: the hand-off was reviewed at the head, {}, which covers the {commits} \
                 since {since}{}.\n",
                repo.short(head)?,
                below(findings.len(), mixed, release)
            )
        } else {
            findings.extend(finding(
                b"the hand-off",
                &binding,
                "the branch since the base",
            ));
            format!(
                "{LABEL}: {commits} since {since}, bound only by the hand-off{}.\n",
                below(findings.len(), mixed, release)
            )
        }
    } else {
        format!(
            "{LABEL}: {commits} since {since}, bound only by a hand-off: name the pull request's \
             description with --hand-off <file>{}.\n",
            below(findings.len(), mixed, release)
        )
    };
    let mut out = headline.into_bytes();
    list(&mut out, &findings);
    report_mixed(&mut out, mixed, release);
    Ok(out)
}

#[cfg(test)]
mod tests;
