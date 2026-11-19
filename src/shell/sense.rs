//! `nuc sense <name>`: the report of a sensor of the template, as the bash sensor of the tree's
//! release writes it (API-git-only-sensors-under-sense). The shell reads `nuc.toml` as `nuc check`
//! does, the README's release line, and what the sensor needs from the tree and from git; the
//! sensor decides; the shell writes the report on the standard output and exits 0, or ends the
//! run not decided with the sensor's key (API-command-line-with-sense).

use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::config;
use crate::sense::{
    self, Refusal, action_refs, backlog, change, plain_rules, plan_frontier, revision_due,
    tests_kept,
};

use super::tree::Tree;
use super::{FOUND, PASSED, git};

/// `nuc sense <name> [--repo <path>] [<argument>...]`: the report of the sensor on the repository
/// that contains `path`, or the directory `nuc` runs in, given the arguments the bash sensor takes.
/// `nuc.toml` is judged before the name, so that a `nuc` too old for the project says so whatever
/// sensor it is asked for (G-5).
pub(super) fn run(name: &str, options: &[&str]) -> Result<u8, String> {
    let (repo, args) = match options {
        ["--repo", path, rest @ ..] => (Some(PathBuf::from(path)), rest),
        rest => (None, rest),
    };
    // `--repo` names the repository, as for every command; the bash's `--root` is refused, and so
    // is every other argument that reads as an option but the `--require` of `action-refs` and
    // `plain-rules`, as the bash of v0.10.0 on refuses them.
    let takes_require = [action_refs::SENSOR.name, plain_rules::SENSOR.name].contains(&name);
    let flag = |(at, arg): (usize, &&str)| {
        arg.starts_with('-') && !(takes_require && at == 0 && *arg == "--require")
    };
    if args.iter().enumerate().any(flag) {
        return Err(super::usage());
    }
    let start = match repo {
        Some(repo) => repo,
        None => {
            env::current_dir().map_err(|e| format!("the current directory is unreadable: {e}"))?
        }
    };
    let root = git::toplevel(&start)?;
    let tree = Tree::new(root)?;
    let text = tree
        .read(&tree.root.join("nuc.toml"))
        .map_err(config::unreadable)?
        .ok_or_else(|| config::Refusal::Missing.to_string())?;
    config::read(&text, super::running()).map_err(|refusal| refusal.to_string())?;
    let Some(sensor) = sense::find(name) else {
        return Err(format!(
            "{name}: no sensor nuc ports — it ports {} — not decided [{}]",
            sense::ALL
                .iter()
                .map(|sensor| sensor.name)
                .collect::<Vec<_>>()
                .join(", "),
            sense::UNKNOWN
        ));
    };
    let carried = super::template_release(&tree.root)?;
    let release = sense::release(sensor, carried).map_err(|refusal| refusal.to_string())?;
    if sensor.name == action_refs::SENSOR.name {
        let require = match args {
            [] => None,
            ["--require", value] => Some(action_refs::Form::parse(value).ok_or_else(|| {
                Refusal {
                    sensor: sensor.name,
                    reason: "require",
                    message: format!("'--require' takes 'major-tag' or 'sha', not '{value}'"),
                }
                .to_string()
            })?),
            _ => return Err(super::usage()),
        };
        let outcome = action_refs(&tree, require).map_err(|refusal| refusal.to_string())?;
        write_report(&outcome.report)?;
        return Ok(match outcome.failed {
            Some(failed) => {
                // The references outside the decided form stand on the standard error, where the
                // bash writes them, and the run exits 1, as a check with findings does.
                let _ = io::stderr().write_all(&failed);
                FOUND
            }
            None => PASSED,
        });
    }
    if sensor.name == plain_rules::SENSOR.name {
        let positional = args.strip_prefix(&["--require"]).unwrap_or(args);
        if positional.len() > 2 {
            return Err(super::usage());
        }
        let outcome = plain_rules(&tree, release, args).map_err(|refusal| refusal.to_string())?;
        write_report(&outcome.report)?;
        // A failed `--require` writes its report on the standard output, as the bash does, and
        // exits 1.
        return Ok(if outcome.failed { FOUND } else { PASSED });
    }
    let report = match (sensor.name, args) {
        ("backlog", []) => backlog(&tree),
        ("plan-frontier", []) => plan_frontier(&tree),
        ("revision-due", []) => revision_due(&tree, release),
        ("tests-kept", [] | [_] | [_, _]) => {
            tests_kept(&tree, release, args.first().copied(), args.get(1).copied())
        }
        _ => return Err(super::usage()),
    }
    .map_err(|refusal| refusal.to_string())?;
    write_report(&report)?;
    Ok(PASSED)
}

/// The report written on the standard output.
fn write_report(report: &[u8]) -> Result<(), String> {
    io::stdout()
        .write_all(report)
        .and_then(|()| io::stdout().flush())
        .map_err(|e| format!("the report could not be written: {e}"))
}

/// What `action-refs` needs: the workflows as the bash's globs list them, `*.yml` and then
/// `*.yaml`, each in the order of its bytes, with their texts; and `.github/dependabot.yml` where
/// it is a file. An entry the glob lists that is no file, such as a directory, has no line to read.
fn action_refs(
    tree: &Tree,
    require: Option<action_refs::Form>,
) -> Result<action_refs::Outcome, Refusal> {
    let sensor = action_refs::SENSOR.name;
    let refuse = |message: String| Refusal {
        sensor,
        reason: "file",
        message: unwrapped(message),
    };
    let dir = tree.root.join(action_refs::WORKFLOWS);
    let names = tree.list(&dir).map_err(refuse)?;
    let mut workflows = Vec::new();
    for suffix in [".yml", ".yaml"] {
        for name in &names {
            if !name.starts_with('.') && name.ends_with(suffix) {
                workflows.push(name.clone());
            }
        }
    }
    if workflows.is_empty() {
        return Ok(action_refs::Outcome {
            report: action_refs::no_workflow(),
            failed: None,
        });
    }
    // The bytes as grep reads them; a file with a NUL byte, which grep reads as binary, is not
    // decided.
    let read = |relative: &str| tree.read_file(relative).map_err(refuse);
    let mut count = action_refs::Count::default();
    for name in &workflows {
        let relative = format!("{}/{name}", action_refs::WORKFLOWS);
        if let Some(text) = read(&relative)? {
            action_refs::read(&mut count, &relative, &text, require);
        }
    }
    let keeps = action_refs::keeps_actions(read(action_refs::DEPENDABOT)?.as_deref());
    Ok(action_refs::outcome(count, require, keeps))
}

/// What `revision-due` needs: the overview where it is a file, the commit that set the date, the
/// count of the changes outside `docs/` since, and where the revision is due, the entries they
/// touched that still exist; git asked as the bash asks it.
fn revision_due(tree: &Tree, release: config::Version) -> Result<Vec<u8>, Refusal> {
    let refuse = |reason| {
        move |message: String| Refusal {
            sensor: revision_due::SENSOR.name,
            reason,
            message: unwrapped(message),
        }
    };
    let undecided = refuse("git");
    let overview_path = tree.root.join(revision_due::OVERVIEW);
    let overview = if overview_path.is_file() {
        tree.read_checked(&overview_path)
            .map_err(refuse("file"))?
            .map(String::into_bytes)
    } else {
        None
    };
    let line = revision_due::read(overview.as_deref(), release)?;
    let root = &tree.root;
    let head = git::probe(root, &["rev-parse", "--verify", "-q", "HEAD"]).map_err(undecided)?;
    let range = match (&head, &line.date) {
        (None, _) => None,
        (Some(_), None) => Some("HEAD".to_owned()),
        (Some(_), Some(date)) => {
            let pattern = revision_due::anchor_pattern(date, release);
            let pattern = String::from_utf8(pattern).expect("the date is ASCII");
            // No textconv driver of the repository runs on the diffs `-G` reads (Q-7): the bash's
            // git would run one, so on a repository that sets one for the overview the two may
            // find another commit.
            let log = git::run(
                root,
                &[
                    "log",
                    "--no-textconv",
                    "--reverse",
                    "--format=%H",
                    &format!("-G{pattern}"),
                    "HEAD",
                    "--",
                    revision_due::OVERVIEW,
                ],
            )
            .map_err(undecided)?;
            log.lines()
                .next()
                .filter(|anchor| !anchor.is_empty())
                .map(|anchor| format!("{anchor}..HEAD"))
        }
    };
    let Some(range) = range else {
        return Ok(revision_due::report(&line, 0, &[], release));
    };
    let outside_docs = ["--", ".", ":(exclude)docs"];
    let mut args = vec!["rev-list", "--count", "--no-merges", range.as_str()];
    args.extend(outside_docs);
    let count = git::run(root, &args).map_err(undecided)?;
    let count: u64 = count
        .trim_end()
        .parse()
        .map_err(|_| undecided(format!("git rev-list --count wrote '{}'", count.trim_end())))?;
    if !revision_due::is_due(&line, count) {
        return Ok(revision_due::report(&line, count, &[], release));
    }
    let mut args = vec![
        "log",
        "--no-merges",
        "--name-only",
        "--format=%x01",
        range.as_str(),
    ];
    args.extend(outside_docs);
    let log = git::run_bytes(root, &args).map_err(undecided)?;
    let scopes: Vec<_> = revision_due::scopes(&log)
        .into_iter()
        .filter(|(_, entry)| exists(root, revision_due::scope_path(entry)))
        .collect();
    Ok(revision_due::report(&line, count, &scopes, release))
}

/// The change a sensor reads, as `resolve_change` resolves it: the base and the head as named,
/// and whether the change ends in the working tree.
struct Change {
    base: String,
    head: String,
    worktree: bool,
}

/// The change as `resolve_change` resolves it, or the report where there is nothing to compare; a
/// base or head that resolves to no commit is refused.
fn resolve_change(
    root: &Path,
    sensor: &'static str,
    label: &str,
    base: Option<&str>,
    head: Option<&str>,
) -> Result<Result<Change, Vec<u8>>, Refusal> {
    let undecided = git_refusal(sensor);
    let given = change::given(base, head);
    let head = given.head.to_owned();
    let base = match given.base {
        Some(base) => Some(base.to_owned()),
        None => main_merge_base(root, &head).map_err(undecided)?,
    };
    let Some(base) = base else {
        return Ok(Err(change::nothing_to_compare(label)));
    };
    for reference in [&base, &head] {
        let verified = git::probe(
            root,
            &[
                "rev-parse",
                "--verify",
                "-q",
                &format!("{reference}^{{commit}}"),
            ],
        )
        .map_err(undecided)?;
        if verified.is_none() {
            return Err(change::unresolved(sensor, reference));
        }
    }
    Ok(Ok(Change {
        base,
        head,
        worktree: given.worktree,
    }))
}

/// The merge base of the head with the first of the mains that exists and has one, as
/// `main_merge_base` of the template's `scripts/lib/git.sh` finds it.
fn main_merge_base(root: &Path, head: &str) -> Result<Option<String>, String> {
    for main in change::MAINS {
        let commit = format!("{main}^{{commit}}");
        if git::probe(root, &["rev-parse", "-q", "--verify", &commit])?.is_none() {
            continue;
        }
        // merge-base exits 1 where there is none, and 128 on a head that does not resolve: the
        // bash tries the next main either way.
        let (code, out) = git::status_and_output(root, &["merge-base", main, head])?;
        if code == 0 {
            let base = String::from_utf8_lossy(&out).trim_end().to_owned();
            return Ok(Some(base));
        }
    }
    Ok(None)
}

/// A refusal of the sensor keyed `git`, for git that could not be run or failed.
fn git_refusal(sensor: &'static str) -> impl Fn(String) -> Refusal + Copy {
    move |message| Refusal {
        sensor,
        reason: "git",
        message: unwrapped(message),
    }
}

/// What `tests-kept` needs: the change, its deletions and renames, and its diff without the paths
/// that carry the markers as data; git asked as the bash of the release asks it, and never through
/// a textconv driver or an external diff of the repository (Q-7).
fn tests_kept(
    tree: &Tree,
    release: config::Version,
    base: Option<&str>,
    head: Option<&str>,
) -> Result<Vec<u8>, Refusal> {
    let root = &tree.root;
    let sensor = tests_kept::SENSOR.name;
    let undecided = git_refusal(sensor);
    let (base, head) = match resolve_change(root, sensor, tests_kept::LABEL, base, head)? {
        Ok(change) => (change.base, change.head),
        Err(report) => return Ok(report),
    };
    let commit = |reference: &str, short: bool| {
        let named = format!("{reference}^{{commit}}");
        let args: &[&str] = if short {
            &["rev-parse", "--short", reference]
        } else {
            &["rev-parse", &named]
        };
        git::run(root, args)
            .map(|out| out.trim_end().to_owned())
            .map_err(undecided)
    };
    if commit(&base, false)? == commit(&head, false)? {
        return Ok(tests_kept::same_commit());
    }
    let name_status = git::run_bytes(
        root,
        &[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--name-status",
            "-M",
            "--diff-filter=DR",
            &base,
            &head,
        ],
    )
    .map_err(undecided)?;
    let mut args: Vec<&str> = Vec::new();
    if release >= tests_kept::BY_EXTENSION {
        args.extend(["-c", "core.quotePath=false"]);
    }
    args.extend([
        "diff",
        "--no-ext-diff",
        "--no-textconv",
        "-U0",
        "--no-color",
        &base,
        &head,
        "--",
        ".",
    ]);
    args.extend(tests_kept::LEFT_OUT);
    let diff = git::run_bytes(root, &args).map_err(undecided)?;
    let mut findings = tests_kept::removals(&name_status);
    findings.extend(tests_kept::markers(&diff, release));
    Ok(tests_kept::report(
        &commit(&base, true)?,
        &commit(&head, true)?,
        &findings,
    ))
}

/// What `plain-rules` needs: the change; the files of rules it touches, each with its text and the
/// lines the change adds; and the glossary and the ADR index as the change has them.
fn plain_rules(
    tree: &Tree,
    release: config::Version,
    args: &[&str],
) -> Result<plain_rules::Outcome, Refusal> {
    let root = &tree.root;
    let sensor = plain_rules::SENSOR.name;
    let undecided = git_refusal(sensor);
    let (require, args) = match args {
        ["--require", rest @ ..] if release >= plain_rules::REQUIRE => (true, rest),
        rest => (false, rest),
    };
    let (base, head) = (args.first().copied(), args.get(1).copied());
    // Before `REQUIRE` the bash reads `--require` as its base, which names no commit; nuc hands
    // git no argument that reads as an option.
    if let Some(option) = base.filter(|base| base.starts_with('-')) {
        return Err(change::unresolved(sensor, option));
    }
    let change = match resolve_change(root, sensor, plain_rules::LABEL, base, head)? {
        Ok(change) => change,
        Err(report) => {
            return Ok(plain_rules::Outcome {
                report,
                failed: false,
            });
        }
    };
    let short = |reference: &str| {
        git::run(root, &["rev-parse", "--short", reference])
            .map(|out| out.trim_end().to_owned())
            .map_err(undecided)
    };
    let range = if change.worktree {
        format!("{}..working tree", short(&change.base)?)
    } else {
        format!("{}..{}", short(&change.base)?, short(&change.head)?)
    };
    let files = rule_files(tree, release, &change)?;
    let glossary = change_file(tree, &change, "docs/GLOSSARY.md")?.unwrap_or_default();
    let index = if release >= plain_rules::TOKENS {
        change_file(tree, &change, "docs/adr/README.md")?.unwrap_or_default()
    } else {
        Vec::new()
    };
    Ok(plain_rules::report(
        release, &range, &files, &glossary, &index, require,
    ))
}

/// A file as the change has it, as `change_file` reads it: in the working tree where the change
/// ends there, else at the head; none where it has no such file. A NUL byte, which the
/// bash's command substitution drops, is not decided (Q-1).
fn change_file(tree: &Tree, change: &Change, path: &str) -> Result<Option<Vec<u8>>, Refusal> {
    let bytes = if change.worktree {
        tree.read_file(path).map_err(file_refusal)?
    } else {
        git::blob_at(&tree.root, &change.head, path)
            .map_err(git_refusal(plain_rules::SENSOR.name))?
    };
    bytes.map(|bytes| without_nul(path, bytes)).transpose()
}

/// A refusal of `plain-rules` keyed `file`.
fn file_refusal(message: String) -> Refusal {
    Refusal {
        sensor: plain_rules::SENSOR.name,
        reason: "file",
        message: unwrapped(message),
    }
}

/// The bytes of a file the sensor reads, refused where they hold a NUL byte (Q-1).
fn without_nul(path: &str, bytes: Vec<u8>) -> Result<Vec<u8>, Refusal> {
    if bytes.contains(&0) {
        return Err(file_refusal(format!(
            "{path}: a NUL byte, which the bash does not read as the other bytes"
        )));
    }
    Ok(bytes)
}

/// The filters of a change that ends in the working tree: the keys that keep its diffs from
/// running one, and the tracked paths under one that git would run.
struct Filters {
    keys: Vec<String>,
    paths: std::collections::BTreeSet<Vec<u8>>,
}

/// The keys that keep a diff against the working tree from running a filter, and the tracked paths
/// under an active one. git runs a filter only for a driver that any entry of the configuration
/// gives a `clean` or `process` command, not empty, and the attribute's value names the driver. It
/// runs the filter of a stat-dirty file anywhere in the tree, whatever the pathspec, so every
/// active driver gets an empty `process` command, which keeps git from running both it and
/// the `clean` command, and `required` off, so that the diff does not fail for want of either
/// (Q-7). A driver no entry makes active runs no command and is left as the bash's git reads it. An
/// active driver whose name holds a `=`, which `-c` would split there and so neutralize another
/// key, or is no UTF-8, is not decided.
///
/// A file of rules under an active driver is not decided: the bash's git reads it through the
/// filter, nuc's would read it raw, and the two can tell another change. `check-attr` prints `set`
/// both for a bare `filter`, which names no driver, and for a driver named `set`, so such a file is
/// refused only where a driver of that name is active, the side that never passes.
fn neutralized_filters(root: &Path, release: config::Version) -> Result<Filters, Refusal> {
    let sensor = plain_rules::SENSOR.name;
    let undecided = git_refusal(sensor);
    let refuse = |message: String| Refusal {
        sensor,
        reason: "filter",
        message,
    };
    let mut filters = Filters {
        keys: Vec::new(),
        paths: std::collections::BTreeSet::new(),
    };
    // `<key> LF <value> NUL` for each key; none, and exit 1, where the configuration has none.
    let (code, config) =
        git::status_and_output(root, &["config", "-z", "--get-regexp", r"^filter\."])
            .map_err(undecided)?;
    if code > 1 {
        return Err(undecided("git config --get-regexp failed".to_owned()));
    }
    let active = plain_rules::filter_drivers(&config);
    for name in &active {
        let name = String::from_utf8(name.clone())
            .ok()
            .filter(|name| !name.contains('='))
            .ok_or_else(|| {
                refuse(format!(
                    "'{}': a filter driver whose name nuc cannot neutralize",
                    String::from_utf8_lossy(name)
                ))
            })?;
        for key in ["process=", "required=false"] {
            filters.keys.push(format!("filter.{name}.{key}"));
        }
    }
    if active.is_empty() {
        return Ok(filters);
    }
    let tracked = git::run_bytes(root, &["ls-files", "-z"]).map_err(undecided)?;
    if tracked.is_empty() {
        return Ok(filters);
    }
    let attrs =
        git::run_bytes_with_input(root, &["check-attr", "-z", "--stdin", "filter"], &tracked)
            .map_err(undecided)?;
    // `<path> NUL filter NUL <value> NUL` for each path.
    let fields: Vec<&[u8]> = attrs.split(|&b| b == 0).collect();
    for entry in fields.chunks(3) {
        let [path, _, value] = entry else {
            continue;
        };
        if !active.contains(*value) {
            continue;
        }
        if plain_rules::is_rule_file(path, release) {
            return Err(refuse(format!(
                "{}: its attributes name a filter, through which the bash's git reads it",
                String::from_utf8_lossy(path)
            )));
        }
        filters.paths.insert(path.to_vec());
    }
    Ok(filters)
}

/// The untracked files `ls-files` names, unquoted, where the change ends in the working tree.
fn untracked(root: &Path, change: &Change) -> Result<Vec<u8>, Refusal> {
    if !change.worktree {
        return Ok(Vec::new());
    }
    let args = [
        "-c",
        "core.quotePath=false",
        "ls-files",
        "--others",
        "--exclude-standard",
    ];
    git::run_bytes(root, &args).map_err(git_refusal(plain_rules::SENSOR.name))
}

/// Refused where rename detection could pair a file of rules otherwise than the bash's git does.
/// Its candidates for the new side of a rename are the paths the change adds, whatever their
/// content. git hashes one under a neutralized filter as its raw bytes, where the bash's git hashes
/// the bytes its filter cleans. So such a path can take or leave the old path a file of rules the
/// change adds would pair with: the file then reads with every line added, or only with its diff's,
/// and `--require` can pass where the bash fails. That needs an old side, a path the change deletes
/// or renames, and on the new side both a file of rules and a path under an active filter; a
/// modified path is no candidate. Both listings name the same paths on either side, since whether
/// a path is added or deleted does not depend on its content. Before `REQUIRE`, which finds no
/// renames, nothing is paired.
fn refuse_filtered_renames(
    diff: &[u8],
    filters: &Filters,
    release: config::Version,
) -> Result<(), Refusal> {
    if release < plain_rules::REQUIRE {
        return Ok(());
    }
    let listing = plain_rules::new_sides(diff);
    if !listing.sources {
        return Ok(());
    }
    let added = listing.added;
    let filtered = added.iter().find(|path| filters.paths.contains(*path));
    let rules = added
        .iter()
        .any(|path| plain_rules::is_rule_file(path, release));
    match filtered {
        Some(path) if rules => Err(Refusal {
            sensor: plain_rules::SENSOR.name,
            reason: "filter",
            message: format!(
                "{}: added under a filter beside a file of rules the change adds, whose rename \
                 git would find through the bytes the filter cleans",
                String::from_utf8_lossy(path)
            ),
        }),
        _ => Ok(()),
    }
}

/// The files of rules the change touches, as `rule_stream` of the release lists them, each with its
/// text and the lines the change adds: every line of an untracked file, else the lines of its diff.
/// A file is read only where the change adds a line to it, as the bash reads it. git is asked as
/// the bash of the release asks it, but never through a textconv driver, an external diff, or a
/// filter of the repository or of a submodule (Q-7): the diffs of a change that ends in the
/// working tree neutralize every filter driver, refuse a file of rules under one, and leave
/// submodules out, whose own filters git would run to tell whether they are dirty.
fn rule_files(
    tree: &Tree,
    release: config::Version,
    change: &Change,
) -> Result<Vec<plain_rules::RuleFile>, Refusal> {
    let root = &tree.root;
    let sensor = plain_rules::SENSOR.name;
    let undecided = git_refusal(sensor);
    let mut refs = vec![change.base.as_str()];
    if !change.worktree {
        refs.push(&change.head);
    }
    let (listing, renames) = if release >= plain_rules::REQUIRE {
        ("--name-status", "-M")
    } else {
        ("--name-only", "--no-renames")
    };
    let mut guards = vec!["--no-ext-diff", "--no-textconv"];
    let filters = if change.worktree {
        guards.push("--ignore-submodules=all");
        Some(neutralized_filters(root, release)?)
    } else {
        None
    };
    // `-c <key>` before the command, for each diff that may read the working tree.
    let unfiltered: Vec<&str> = filters
        .iter()
        .flat_map(|filters| &filters.keys)
        .flat_map(|key| ["-c", key.as_str()])
        .collect();
    let quoted = ["-c", "core.quotePath=false"];
    let mut args = quoted.to_vec();
    args.extend(&unfiltered);
    args.push("diff");
    args.extend(&guards);
    args.extend([listing, renames]);
    args.extend(&refs);
    let diff = git::run_bytes(root, &args).map_err(undecided)?;
    if let Some(filters) = &filters {
        refuse_filtered_renames(&diff, filters, release)?;
    }
    let untracked = untracked(root, change)?;
    let mut files = Vec::new();
    for (path, from) in plain_rules::listed(&diff, &untracked, release) {
        if !plain_rules::is_rule_file(&path, release) {
            continue;
        }
        let path = spelled_for(sensor, &path)?;
        let from = spelled_for(sensor, &from)?;
        // Whether the change has the file, as `-f` or `git cat-file -e` asks, before it is read.
        let at_head = if change.worktree {
            if !root.join(&path).is_file() {
                continue;
            }
            None
        } else {
            match git::blob_at(root, &change.head, &path).map_err(undecided)? {
                Some(bytes) => Some(bytes),
                None => continue,
            }
        };
        let tracked = !change.worktree
            || git::status_and_output(root, &["ls-files", "--error-unmatch", "--", &path])
                .map_err(undecided)?
                .0
                == 0;
        let (added, text) = if tracked {
            let mut args = unfiltered.clone();
            args.push("diff");
            args.extend(&guards);
            args.extend(["--no-color", "-U0", renames]);
            args.extend(&refs);
            args.push("--");
            if !from.is_empty() {
                args.push(&from);
            }
            args.push(&path);
            let added = plain_rules::added_lines(&git::run_bytes(root, &args).map_err(undecided)?);
            if added.is_empty() {
                continue;
            }
            let text = match at_head {
                Some(bytes) => without_nul(&path, bytes)?,
                None => match change_file(tree, change, &path)? {
                    Some(text) => text,
                    None => continue,
                },
            };
            (added, text)
        } else {
            let Some(text) = change_file(tree, change, &path)? else {
                continue;
            };
            (plain_rules::every_line(&text), text)
        };
        if !added.is_empty() {
            files.push(plain_rules::RuleFile {
                path: path.into_bytes(),
                text,
                added,
            });
        }
    }
    Ok(files)
}

/// The lines of git's output as `while IFS= read -r` reads them.
fn lines_of(
    root: &Path,
    args: &[&str],
    undecided: impl Fn(String) -> Refusal,
) -> Result<Vec<Vec<u8>>, Refusal> {
    let out = git::run_bytes(root, args).map_err(undecided)?;
    Ok(crate::bash::read_lines(&out)
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect())
}

/// What `backlog` needs: main, the first of the candidates that names a commit; the entries of
/// main's tree; and those each local branch that main does not hold adds after its merge base,
/// each with the text git shows of it.
fn backlog(tree: &Tree) -> Result<Vec<u8>, Refusal> {
    let root = &tree.root;
    let undecided = git_refusal(backlog::SENSOR.name);
    let Some(main) = first_commit(root, &backlog::MAINS).map_err(undecided)? else {
        return Ok(backlog::no_commit());
    };
    let dir = format!("{}/", backlog::DIR);
    let shown = |reference: &[u8], path: &[u8]| -> Result<Option<Vec<u8>>, Refusal> {
        // `git show` of a blob is its bytes; a path the commit lacks shows nothing, and an object
        // the clone lacks is not decided.
        git::blob_at(root, &spelled(reference)?, &spelled(path)?).map_err(undecided)
    };
    let mut entries = Vec::new();
    let mut on_main = std::collections::BTreeSet::new();
    for path in lines_of(
        root,
        &["ls-tree", "-r", "--name-only", main, "--", &dir],
        undecided,
    )? {
        if !backlog::is_entry(&path) {
            continue;
        }
        let title = backlog::title(shown(main.as_bytes(), &path)?.as_deref(), &path);
        on_main.insert(path.clone());
        entries.push(backlog::Entry {
            path,
            title,
            branch: None,
        });
    }
    let branches = lines_of(
        root,
        &["for-each-ref", "--format=%(refname:short)", "refs/heads/"],
        undecided,
    )?;
    for branch in branches {
        if branch == b"main" {
            continue;
        }
        let name = spelled(&branch)?;
        let Some(base) = unmerged_base(root, &name, main).map_err(undecided)? else {
            continue;
        };
        let added = lines_of(
            root,
            &[
                "diff",
                "--no-ext-diff",
                "--no-renames",
                "--name-only",
                "--diff-filter=A",
                &base,
                &name,
                "--",
                &dir,
            ],
            undecided,
        )?;
        for path in added {
            if !backlog::is_entry(&path) || on_main.contains(&path) {
                continue;
            }
            let title = backlog::title(shown(&branch, &path)?.as_deref(), &path);
            entries.push(backlog::Entry {
                path,
                title,
                branch: Some(branch.clone()),
            });
        }
    }
    Ok(backlog::report(main, &entries))
}

/// The first of the candidates that names a commit, as the bash tries them.
fn first_commit(root: &Path, candidates: &[&'static str]) -> Result<Option<&'static str>, String> {
    for candidate in candidates {
        let commit = format!("{candidate}^{{commit}}");
        if git::probe(root, &["rev-parse", "-q", "--verify", &commit])?.is_some() {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

/// The merge base of the branch with main, where main does not hold the branch and git finds one.
fn unmerged_base(root: &Path, branch: &str, main: &str) -> Result<Option<String>, String> {
    let (merged, _) = git::status_and_output(root, &["merge-base", "--is-ancestor", branch, main])?;
    if merged == 0 {
        return Ok(None);
    }
    let (code, base) = git::status_and_output(root, &["merge-base", main, branch])?;
    Ok((code == 0).then(|| String::from_utf8_lossy(&base).trim_end().to_owned()))
}

/// The bytes as text, where they are UTF-8: a branch or a path git names that is none is not
/// decided, since nuc hands git its arguments as text, and a name it would spell otherwise names
/// another object.
fn spelled(bytes: &[u8]) -> Result<String, Refusal> {
    spelled_for(backlog::SENSOR.name, bytes)
}

/// The bytes as text for the sensor, as [`spelled`] reads them.
fn spelled_for(sensor: &'static str, bytes: &[u8]) -> Result<String, Refusal> {
    String::from_utf8(bytes.to_vec()).map_err(|_| Refusal {
        sensor,
        reason: "name",
        message: format!(
            "'{}': a name git gives that is no UTF-8, which nuc cannot hand back to git",
            String::from_utf8_lossy(bytes)
        ),
    })
}

/// What `plan-frontier` needs: the tickets as `find` lists them, with their texts; the ticket
/// branches, local or remote; and which blockers of each are still files.
fn plan_frontier(tree: &Tree) -> Result<Vec<u8>, Refusal> {
    let root = &tree.root;
    let sensor = plan_frontier::SENSOR.name;
    let tickets = tree.read_tickets().map_err(|message| Refusal {
        sensor,
        reason: "file",
        message: unwrapped(message),
    })?;
    let refs = git::run_bytes(
        root,
        &[
            "for-each-ref",
            "--format=%(refname)",
            "refs/heads/ticket",
            "refs/remotes",
        ],
    )
    .map_err(git_refusal(sensor))?;
    let claims = plan_frontier::claims(&refs);
    let (mut placed, mut errors) = (Vec::new(), Vec::new());
    for (path, text) in &tickets {
        let ticket = match plan_frontier::read(path, text) {
            Ok(ticket) => ticket,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        if claims.contains(ticket.branch.as_bytes()) {
            placed.push((ticket, plan_frontier::Place::Claimed));
            continue;
        }
        let open: Vec<Vec<u8>> = ticket
            .blockers
            .iter()
            .filter(|p| p.as_slice() != path.as_bytes() && is_file(root, p))
            .cloned()
            .collect();
        let place = if open.is_empty() {
            plan_frontier::Place::Frontier
        } else {
            plan_frontier::Place::Blocked(open)
        };
        placed.push((ticket, place));
    }
    plan_frontier::report(tickets.len(), &placed, &errors)
}

/// Whether the path under the root is a file, as the bash's `-f` tests it: it follows a symbolic
/// link and reads nothing (Q-12).
fn is_file(root: &Path, relative: &[u8]) -> bool {
    std::str::from_utf8(relative)
        .map(|relative| root.join(relative).is_file())
        .unwrap_or(false)
}

/// The message of a refusal the tree or git gave, without the words `Refusal` adds itself.
fn unwrapped(message: String) -> String {
    message
        .strip_suffix(" — not decided")
        .map(str::to_owned)
        .unwrap_or(message)
}

/// Whether the path under the root exists, as the bash's `-e` tests it: it follows a symbolic link
/// and reads nothing (Q-12).
fn exists(root: &Path, relative: &[u8]) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let mut path = root.as_os_str().to_owned();
        path.push("/");
        path.push(std::ffi::OsStr::from_bytes(relative));
        Path::new(&path).exists()
    }
    #[cfg(not(unix))]
    {
        std::str::from_utf8(relative)
            .map(|relative| Path::new(&format!("{}/{relative}", root.display())).exists())
            .unwrap_or(false)
    }
}
