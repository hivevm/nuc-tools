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
    self, Refusal, action_refs, backlog, binding_adrs, change, plain_rules, plan_frontier,
    review_binding, revision_due, tests_kept,
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
    // `plain-rules`, as the bash of v0.10.0 on refuses them. `binding-adrs` and `review-binding`
    // read their own options first, and refuse the rest by `sense::positional`, as the dispatch
    // below does for `tests-kept` and `plain-rules`.
    let takes_require = [action_refs::SENSOR.name, plain_rules::SENSOR.name].contains(&name);
    let flag = |(at, arg): (usize, &&str)| {
        arg.starts_with('-') && !(takes_require && at == 0 && *arg == "--require")
    };
    let own_options = [binding_adrs::SENSOR.name, review_binding::SENSOR.name].contains(&name);
    if !own_options && args.iter().enumerate().any(flag) {
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
    refuse_environment(sensor.name, &tree.root)?;
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
    if sensor.name == binding_adrs::SENSOR.name {
        let parsed = binding_adrs::parse(args, release).ok_or_else(super::usage)?;
        let report =
            binding_adrs(&tree, release, &parsed).map_err(|refusal| refusal.to_string())?;
        write_report(&report)?;
        return Ok(PASSED);
    }
    if sensor.name == review_binding::SENSOR.name {
        let parsed = review_binding::parse(args, release).ok_or_else(super::usage)?;
        let report =
            review_binding(&tree, release, &parsed).map_err(|refusal| refusal.to_string())?;
        write_report(&report)?;
        return Ok(PASSED);
    }
    if sensor.name == plain_rules::SENSOR.name {
        let (flag, rest) = args.split_at(usize::from(args.first() == Some(&"--require")));
        let positional = sense::positional(rest, 2, release >= plain_rules::ARGUMENTS)
            .ok_or_else(super::usage)?;
        let args = [flag, positional].concat();
        let outcome = plain_rules(&tree, release, &args).map_err(|refusal| refusal.to_string())?;
        write_report(&outcome.report)?;
        // A failed `--require` writes its report on the standard output, as the bash does, and
        // exits 1.
        return Ok(if outcome.failed { FOUND } else { PASSED });
    }
    let report = match (sensor.name, args) {
        ("backlog", []) => backlog(&tree),
        ("plan-frontier", []) => plan_frontier(&tree),
        ("revision-due", []) => revision_due(&tree, release),
        ("tests-kept", _) => {
            let change = sense::positional(args, 2, release >= tests_kept::ARGUMENTS)
                .ok_or_else(super::usage)?;
            tests_kept(&tree, release, change)
        }
        _ => return Err(super::usage()),
    }
    .map_err(|refusal| refusal.to_string())?;
    write_report(&report)?;
    Ok(PASSED)
}

/// Not decided, `:environment`, where the caller's environment names a command git would run that
/// nuc's git does not see (SEC-repository-commands-neutralized-or-refused).
fn refuse_environment(sensor: &'static str, root: &Path) -> Result<(), String> {
    let refuse = |message: String| {
        Refusal {
            sensor,
            reason: sense::ENVIRONMENT,
            message: unwrapped(message),
        }
        .to_string()
    };
    match git::environment_driver(root).map_err(refuse)? {
        Some(why) => Err(refuse(why)),
        None => Ok(()),
    }
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
            // A textconv driver on the overview has the bash's `-G` search what it prints.
            let drivers = diff_drivers(root, revision_due::SENSOR.name)?;
            let overview = [revision_due::OVERVIEW.as_bytes()];
            refuse_driven(root, revision_due::SENSOR.name, &drivers, &overview, true)?;
            // No textconv driver of the repository runs on the diffs `-G` reads (Q-7): the bash's
            // git would run one, so on a repository that sets one for the overview the two may
            // find another commit.
            let log = git::diff(
                root,
                &[],
                git::Diff::Log,
                &[
                    "--reverse",
                    "--format=%H",
                    &format!("-G{pattern}"),
                    "HEAD",
                    "--",
                    revision_due::OVERVIEW,
                ],
            )
            .map_err(undecided)?;
            String::from_utf8(log)
                .map_err(|_| undecided("git log wrote no UTF-8".to_owned()))?
                .lines()
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

/// What `tests-kept` needs, given the base and the head it reads: the change, its deletions and
/// renames, and its diff without the paths that carry the markers as data; git asked as the bash
/// of the release asks it, and never through a textconv driver or an external diff of the
/// repository (Q-7).
fn tests_kept(tree: &Tree, release: config::Version, change: &[&str]) -> Result<Vec<u8>, Refusal> {
    let (base, head) = (change.first().copied(), change.get(1).copied());
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
    let name_status = git::diff(
        root,
        &[],
        git::Diff::Diff,
        &["--name-status", "-M", "--diff-filter=DR", &base, &head],
    )
    .map_err(undecided)?;
    let config: &[&str] = if release >= tests_kept::BY_EXTENSION {
        &["core.quotePath=false"]
    } else {
        &[]
    };
    let mut args = vec!["-U0", "--no-color", &base, &head, "--", "."];
    args.extend(tests_kept::LEFT_OUT);
    let diff = git::diff(root, config, git::Diff::Diff, &args).map_err(undecided)?;
    refuse_driven_markers(root, &base, &head, &diff)?;
    let mut findings = tests_kept::removals(&name_status);
    findings.extend(tests_kept::markers(&diff, release));
    Ok(tests_kept::report(
        &commit(&base, true)?,
        &commit(&head, true)?,
        &findings,
    ))
}

/// Refused where the bash's git reads the hunks of a path of the change through a driver of the
/// repository, or prints the hunks of a submodule the change moves. Every path's hunks decide: the
/// bash's awk takes any line that starts with `+++ ` for the head of a file, and an added line
/// that starts with `++ ` reads so, whatever the extension of the path it is added to.
fn refuse_driven_markers(root: &Path, base: &str, head: &str, diff: &[u8]) -> Result<(), Refusal> {
    let sensor = tests_kept::SENSOR.name;
    let drivers = diff_drivers(root, sensor)?;
    refuse_moved_submodule(sensor, &drivers, diff)?;
    if !drivers.any() {
        return Ok(());
    }
    let mut args = vec!["-z", "--name-only", "--no-renames", base, head, "--", "."];
    args.extend(tests_kept::LEFT_OUT);
    let changed = git::diff(root, &[], git::Diff::Diff, &args).map_err(git_refusal(sensor))?;
    let paths: Vec<&[u8]> = changed
        .split(|&b| b == 0)
        .filter(|path| !path.is_empty())
        .collect();
    refuse_driven(root, sensor, &drivers, &paths, false)
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
/// under an active one, for the sensor. git runs a filter only for a driver that any entry of the
/// configuration gives a `clean` or `process` command, not empty, and the attribute's value names
/// the driver. It runs the filter of a stat-dirty file anywhere in the tree, whatever the
/// pathspec, so every active driver gets an empty `process` command, which keeps git from running
/// both it and the `clean` command, and `required` off, so that the diff does not fail for want of
/// either (Q-7). A driver no entry makes active runs no command and is left as the bash's git reads
/// it. An active driver whose name holds a `=`, which `-c` would split there and so neutralize
/// another key, or is no UTF-8, is not decided. `check-attr` prints `set` both for a bare `filter`,
/// which names no driver, and for a driver named `set`, so such a path counts as under a filter
/// only where a driver of that name is active, the side that never passes.
fn neutralized_filters(root: &Path, sensor: &'static str) -> Result<Filters, Refusal> {
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
        if active.contains(*value) {
            filters.paths.insert(path.to_vec());
        }
    }
    Ok(filters)
}

/// The diff drivers of the repository's configuration, read once a run for the sensor
/// (SEC-repository-commands-neutralized-or-refused).
fn diff_drivers(root: &Path, sensor: &'static str) -> Result<sense::DiffDrivers, Refusal> {
    let undecided = git_refusal(sensor);
    // `<key> LF <value> NUL` an entry; none, and exit 1, where the configuration has none.
    let (code, config) =
        git::status_and_output(root, &["config", "-z", "--get-regexp", r"^diff\."])
            .map_err(undecided)?;
    if code > 1 {
        return Err(undecided("git config --get-regexp failed".to_owned()));
    }
    Ok(sense::diff_drivers(&config))
}

/// The diff drivers, read on the first call of a run and kept in `cache`.
fn cached<'c>(
    cache: &'c mut Option<sense::DiffDrivers>,
    root: &Path,
    sensor: &'static str,
) -> Result<&'c sense::DiffDrivers, Refusal> {
    match cache {
        Some(drivers) => Ok(drivers),
        None => Ok(cache.insert(diff_drivers(root, sensor)?)),
    }
}

/// Refused where the bash's git reads one of the paths, each one whose hunks decide the report,
/// through a driver of the repository: where `diff.external` is set, or where its `diff`
/// attribute names a driver with a textconv or a diff command. Where `log` says the paths are read
/// by `git log -G`, only a textconv driver counts: it runs neither an external diff nor a diff
/// command. `check-attr` prints
/// `set` both for a bare `diff` and for a driver named `set`, so such a path counts as under a
/// driver where one of that name is active, the side that never passes.
fn refuse_driven(
    root: &Path,
    sensor: &'static str,
    drivers: &sense::DiffDrivers,
    paths: &[&[u8]],
    log: bool,
) -> Result<(), Refusal> {
    let refuse = |path: &[u8], through: String| Refusal {
        sensor,
        reason: sense::DRIVER,
        message: format!(
            "{}: its diff decides the report, and the bash's git reads it through {through}",
            String::from_utf8_lossy(path)
        ),
    };
    if drivers.external
        && !log
        && let Some(path) = paths.first()
    {
        return Err(refuse(path, "diff.external".to_owned()));
    }
    let active: std::collections::BTreeSet<&[u8]> = if log {
        drivers.textconv.iter().map(Vec::as_slice).collect()
    } else {
        drivers
            .textconv
            .iter()
            .chain(&drivers.command)
            .map(Vec::as_slice)
            .collect()
    };
    if active.is_empty() || paths.is_empty() {
        return Ok(());
    }
    let mut input = Vec::new();
    for path in paths {
        input.extend_from_slice(path);
        input.push(0);
    }
    let attrs = git::run_bytes_with_input(root, &["check-attr", "-z", "--stdin", "diff"], &input)
        .map_err(git_refusal(sensor))?;
    // The attributes the caller's environment has git read as well, for a path the index lacks.
    let theirs = git::attributes_under_caller(root, &["diff"], &input)
        .map_err(git_refusal(sensor))?
        .unwrap_or_default();
    // `<path> NUL diff NUL <value> NUL` for each path.
    for attrs in [&attrs, &theirs] {
        let fields: Vec<&[u8]> = attrs.split(|&b| b == 0).collect();
        for entry in fields.chunks(3) {
            let [path, _, value] = entry else {
                continue;
            };
            if active.contains(value) {
                return Err(refuse(
                    path,
                    format!("the diff driver '{}'", String::from_utf8_lossy(value)),
                ));
            }
        }
    }
    Ok(())
}

/// Refused where the patch moves a submodule and the configuration's `diff.submodule` has the
/// bash's git print the submodule's own hunks, which `nuc`'s `short` leaves out.
fn refuse_moved_submodule(
    sensor: &'static str,
    drivers: &sense::DiffDrivers,
    diff: &[u8],
) -> Result<(), Refusal> {
    match sense::moved_submodule(diff).filter(|_| drivers.submodule) {
        Some(path) => Err(Refusal {
            sensor,
            reason: sense::DRIVER,
            message: format!(
                "{}: a submodule the change moves, whose own hunks the bash's git prints under \
                 the configuration's diff.submodule",
                String::from_utf8_lossy(&path)
            ),
        }),
        None => Ok(()),
    }
}

/// A file of rules under an active driver is not decided: the bash's git reads it through the
/// filter, nuc's would read it raw, and the two can tell another change.
fn refuse_filtered_rule_files(filters: &Filters, release: config::Version) -> Result<(), Refusal> {
    match filters
        .paths
        .iter()
        .find(|path| plain_rules::is_rule_file(path, release))
    {
        Some(path) => Err(Refusal {
            sensor: plain_rules::SENSOR.name,
            reason: "filter",
            message: format!(
                "{}: its attributes name a filter, through which the bash's git reads it",
                String::from_utf8_lossy(path)
            ),
        }),
        None => Ok(()),
    }
}

/// The untracked files `ls-files` names, unquoted, where the change ends in the working tree.
fn untracked(root: &Path, change: &Change, sensor: &'static str) -> Result<Vec<u8>, Refusal> {
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
    git::run_bytes(root, &args).map_err(git_refusal(sensor))
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
    let mut submodules = Vec::new();
    let filters = if change.worktree {
        submodules.push("--ignore-submodules=all");
        let filters = neutralized_filters(root, sensor)?;
        refuse_filtered_rule_files(&filters, release)?;
        Some(filters)
    } else {
        None
    };
    // The keys that neutralize the filters, for each diff that may read the working tree.
    let unfiltered: Vec<&str> = filters
        .iter()
        .flat_map(|filters| &filters.keys)
        .map(String::as_str)
        .collect();
    let mut config = vec!["core.quotePath=false"];
    config.extend(&unfiltered);
    let mut args = submodules.clone();
    args.extend([listing, renames]);
    args.extend(&refs);
    let diff = git::diff(root, &config, git::Diff::Diff, &args).map_err(undecided)?;
    if let Some(filters) = &filters {
        refuse_filtered_renames(&diff, filters, release)?;
    }
    let untracked = untracked(root, change, sensor)?;
    let mut options = submodules.clone();
    options.extend(["--no-color", "-U0", renames]);
    options.extend(&refs);
    let mut drivers = None;
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
            let drivers = cached(&mut drivers, root, sensor)?;
            let hunks = Hunks {
                config: &unfiltered,
                options: &options,
                drivers,
            };
            let added = rule_file_added(root, &hunks, &from, &path)?;
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

/// How `plain-rules` diffs a tracked file of rules: the configuration keys, the options before the
/// paths, and the diff drivers of the repository.
struct Hunks<'a> {
    config: &'a [&'a str],
    options: &'a [&'a str],
    drivers: &'a sense::DiffDrivers,
}

/// The lines the change adds to a tracked file of rules, by the hunks of its diff, renamed from
/// `from` where that is not empty. The hunks of both paths decide which lines the report reads, so
/// where the bash's git reads them through a driver, the file is not decided.
fn rule_file_added(
    root: &Path,
    hunks: &Hunks,
    from: &str,
    path: &str,
) -> Result<std::collections::BTreeSet<usize>, Refusal> {
    let sensor = plain_rules::SENSOR.name;
    let decisive = [path.as_bytes(), from.as_bytes()];
    let decisive = &decisive[..if from.is_empty() { 1 } else { 2 }];
    refuse_driven(root, sensor, hunks.drivers, decisive, false)?;
    let mut args = hunks.options.to_vec();
    args.push("--");
    if !from.is_empty() {
        args.push(from);
    }
    args.push(path);
    let diff =
        git::diff(root, hunks.config, git::Diff::Diff, &args).map_err(git_refusal(sensor))?;
    Ok(plain_rules::added_lines(&diff))
}

/// What `binding-adrs` needs: the paths named, or the change and the paths it touches; the ADRs
/// and the index as the change has them, where the report reads them; and, for a file named with
/// checks of its own, the file and its diff. git is asked as the bash of the release asks it, but
/// never through a textconv driver, an external diff, or a filter of the repository or of a
/// submodule (Q-7). Where the change ends in the working tree, a submodule checked out is not
/// decided, since the bash's git may ask its own status; the diffs neutralize every filter driver,
/// and a path under a filter that is stat-dirty or racily clean is not decided, since the bash's
/// git hashes it through the filter. A submodule not checked out is read under the repository's own
/// `ignore` settings, as the bash's git reads it.
fn binding_adrs(
    tree: &Tree,
    release: config::Version,
    args: &binding_adrs::Args,
) -> Result<Vec<u8>, Refusal> {
    let root = &tree.root;
    let sensor = binding_adrs::SENSOR.name;
    let undecided = git_refusal(sensor);
    let mut unfiltered: Vec<String> = Vec::new();
    let (listing, change) = if args.named.is_empty() {
        let given = change::given(args.base, args.head);
        match resolve_change(root, sensor, binding_adrs::LABEL, args.base, args.head)? {
            Err(_) => (
                binding_adrs::Listing::Nothing,
                Change {
                    base: String::new(),
                    head: given.head.to_owned(),
                    worktree: given.worktree,
                },
            ),
            Ok(change) => {
                let (paths, keys) = changed_paths(root, &change)?;
                unfiltered = keys;
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
                (binding_adrs::Listing::Changed { range, paths }, change)
            }
        }
    } else {
        let named = args.named.iter().map(|path| path.as_bytes().to_vec());
        (
            binding_adrs::Listing::Named(named.collect()),
            Change {
                base: String::new(),
                head: "HEAD".to_owned(),
                worktree: true,
            },
        )
    };
    let adrs = if binding_adrs::reads_adrs(&listing, args.sections, release) {
        adrs_of(tree, &change, release)?
    } else {
        Vec::new()
    };
    let index = if binding_adrs::reads_index(args.sections, release) {
        shown(tree, &change, binding_adrs::INDEX)?
    } else {
        Vec::new()
    };
    let record = binding_adrs::Record { adrs, index };
    let mut drivers = None;
    let mut diffed = |path: &[u8]| -> Result<binding_adrs::Diffed, Refusal> {
        // The hunks of a file named with checks decide whether it binds.
        let drivers = cached(&mut drivers, root, sensor)?;
        refuse_driven(root, sensor, drivers, &[path], false)?;
        let path = spelled_for(sensor, path)?;
        let file = shown(tree, &change, &path)?;
        let config: Vec<&str> = unfiltered.iter().map(String::as_str).collect();
        let mut args = vec!["--no-color", "-U0", "--no-renames", &change.base];
        if !change.worktree {
            args.push(&change.head);
        }
        args.extend(["--", &path]);
        let diff = git::diff(root, &config, git::Diff::Diff, &args).map_err(undecided)?;
        refuse_moved_submodule(sensor, drivers, &diff)?;
        Ok(binding_adrs::Diffed { file, diff })
    };
    binding_adrs::report(release, args.sections, &listing, &record, &mut diffed)
}

/// The paths a change touches, as `git diff --name-only --no-renames` names them unquoted under
/// `core.quotePath=false`, with the untracked files where it ends in the working tree, sorted and
/// unique; and the keys that keep a diff against the working tree from running a filter.
fn changed_paths(root: &Path, change: &Change) -> Result<(Vec<Vec<u8>>, Vec<String>), Refusal> {
    let sensor = binding_adrs::SENSOR.name;
    let mut keys: Vec<String> = Vec::new();
    if change.worktree {
        refuse_checked_out_submodule(root)?;
        let filters = neutralized_filters(root, sensor)?;
        keys.clone_from(&filters.keys);
        refuse_filtered_dirty(root, &filters, &keys)?;
    }
    let mut config = vec!["core.quotePath=false"];
    config.extend(keys.iter().map(String::as_str));
    let mut args = vec!["--name-only", "--no-renames", &change.base];
    if !change.worktree {
        args.push(&change.head);
    }
    let mut out = git::diff(root, &config, git::Diff::Diff, &args).map_err(git_refusal(sensor))?;
    if change.worktree {
        out.extend(untracked(root, change, sensor)?);
    }
    let mut paths: Vec<Vec<u8>> = crate::bash::read_lines(&out)
        .into_iter()
        .filter(|path| !path.is_empty())
        .map(<[u8]>::to_vec)
        .collect();
    paths.sort();
    paths.dedup();
    Ok((paths, keys))
}

/// Refused where a tracked path under an active filter may differ from the index to the bash's git,
/// which hashes it through the filter where nuc's runs none
/// (SEC-repository-commands-neutralized-or-refused). That is a path whose stat differs from the
/// index, whatever its raw bytes, as a file committed before its filter was set and then touched;
/// and a racily clean one, whose stat matches but whose modification time is no older than the
/// index's, since git then compares its content. `diff-files` names the first by their stat alone,
/// under the keys that keep it from running a filter on a racily clean entry it hashes; the second
/// it names only where the raw bytes differ, so each path's modification time is compared with the
/// index's in whole seconds, as git compares them where it does not compare finer, which errs
/// toward counting.
fn refuse_filtered_dirty(root: &Path, filters: &Filters, keys: &[String]) -> Result<(), Refusal> {
    let sensor = binding_adrs::SENSOR.name;
    if filters.paths.is_empty() {
        return Ok(());
    }
    let undecided = git_refusal(sensor);
    let mut args: Vec<&str> = keys.iter().flat_map(|key| ["-c", key.as_str()]).collect();
    args.extend(["diff-files", "-z", "--name-only"]);
    let dirty = git::run_bytes(root, &args).map_err(undecided)?;
    let dirty: std::collections::BTreeSet<&[u8]> = dirty.split(|&b| b == 0).collect();
    let index = git::run(root, &["rev-parse", "--git-path", "index"]).map_err(undecided)?;
    let index = mtime_secs(&root.join(index.trim_end_matches(['\n', '\r'])));
    let refuse = |path: &[u8], why: &str| Refusal {
        sensor,
        reason: "filter",
        message: format!(
            "{}: {why}, so the bash's git reads it through its filter",
            String::from_utf8_lossy(path)
        ),
    };
    for path in &filters.paths {
        if dirty.contains(path.as_slice()) {
            return Err(refuse(path, "changed in the working tree under a filter"));
        }
        let racy = match (
            relative_path(root, path).and_then(|p| mtime_secs(&p)),
            index,
        ) {
            (Some(file), Some(index)) => file >= index,
            (Some(_), None) => true,
            (None, _) => false,
        };
        if racy {
            return Err(refuse(
                path,
                "racily clean under a filter, its modification time no older than the index's",
            ));
        }
    }
    Ok(())
}

/// The modification time of the file at `path` in whole seconds since the epoch, of the link where
/// it is one, as git's `lstat` reads it; none where it has none.
fn mtime_secs(path: &Path) -> Option<u64> {
    let modified = std::fs::symlink_metadata(path).ok()?.modified().ok()?;
    modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|since| since.as_secs())
}

/// The path of the repository's file at `relative`, as git names it; none where the platform cannot
/// spell it.
fn relative_path(root: &Path, relative: &[u8]) -> Option<PathBuf> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Some(root.join(std::ffi::OsStr::from_bytes(relative)))
    }
    #[cfg(not(unix))]
    {
        std::str::from_utf8(relative)
            .ok()
            .map(|relative| root.join(relative))
    }
}

/// Refused where the index records a submodule that is checked out, its directory holding a
/// `.git`: whether the bash's git lists it can depend on the submodule's own working tree, which
/// git reads by running the submodule's status, and so its filters (Q-7). A submodule that is not
/// checked out is read from the index alone, under the repository's own `ignore` settings, as the
/// bash's git reads it.
fn refuse_checked_out_submodule(root: &Path) -> Result<(), Refusal> {
    let sensor = binding_adrs::SENSOR.name;
    let staged =
        git::run_bytes(root, &["ls-files", "-z", "--stage"]).map_err(git_refusal(sensor))?;
    // '<mode> <object> <stage>\t<path>\0' for each entry.
    for entry in staged.split(|&b| b == 0) {
        let Some(tab) = entry.iter().position(|&b| b == b'\t') else {
            continue;
        };
        if !entry.starts_with(b"160000 ") {
            continue;
        }
        let path = &entry[tab + 1..];
        let mut dot_git = path.to_vec();
        dot_git.extend(b"/.git");
        if exists(root, &dot_git) {
            return Err(Refusal {
                sensor,
                reason: "submodule",
                message: format!(
                    "{}: a submodule checked out, whose state the bash's git may read through its \
                     own status, which runs its filters",
                    String::from_utf8_lossy(path)
                ),
            });
        }
    }
    Ok(())
}

/// The ADR files as the change has them, as `adr_list` lists them, with their bytes: in the
/// working tree each regular file directly in `docs/adr/` whose name the release's glob matches
/// and holds no newline, as `find -type f` lists them, in byte order; else each the head lists
/// there, as `ADR_FILE_RE` matches the lines of `git ls-tree`. A file that is no blob at the head
/// reads as empty.
fn adrs_of(
    tree: &Tree,
    change: &Change,
    release: config::Version,
) -> Result<Vec<binding_adrs::Adr>, Refusal> {
    let sensor = binding_adrs::SENSOR.name;
    let paths: Vec<String> = if change.worktree {
        let dir = tree.root.join(binding_adrs::DIR);
        let names = tree
            .regular_files(&dir, &|name| binding_adrs::is_adr_name(name, release))
            .map_err(|message| Refusal {
                sensor,
                reason: "file",
                message: unwrapped(message),
            })?;
        // `find | while read` splits a name at its newline into paths that name no ADR.
        names
            .into_iter()
            .filter(|name| !name.contains('\n'))
            .map(|name| format!("{}/{name}", binding_adrs::DIR))
            .collect()
    } else {
        let ls_tree = git::run_bytes(
            &tree.root,
            &[
                "ls-tree",
                "-r",
                "--name-only",
                &change.head,
                "--",
                "docs/adr/",
            ],
        )
        .map_err(git_refusal(sensor))?;
        binding_adrs::listed_adrs(&ls_tree, release)
            .iter()
            .map(|path| spelled_for(sensor, path))
            .collect::<Result<_, _>>()?
    };
    paths
        .into_iter()
        .map(|path: String| {
            let text = shown(tree, change, &path)?;
            Ok(binding_adrs::Adr {
                path: path.into_bytes(),
                text,
            })
        })
        .collect()
}

/// A file as `binding-adrs` shows it, as `change_file` gives it: in the working tree as `cat`
/// reads it where it is a file, else at the head as `git show` shows a blob; empty where neither
/// has one. A NUL byte is not decided (Q-1).
fn shown(tree: &Tree, change: &Change, path: &str) -> Result<Vec<u8>, Refusal> {
    let sensor = binding_adrs::SENSOR.name;
    let bytes = if change.worktree {
        let at = tree.root.join(path);
        if at.is_file() {
            tree.read_bytes(&at).map_err(|message| Refusal {
                sensor,
                reason: "file",
                message: unwrapped(message),
            })?
        } else {
            None
        }
    } else {
        git::blob_at(&tree.root, &change.head, path).map_err(git_refusal(sensor))?
    };
    let bytes = bytes.unwrap_or_default();
    if bytes.contains(&0) {
        return Err(binding_adrs::with_nul(path));
    }
    Ok(bytes)
}

/// The lines of git's output as `while IFS= read -r` reads them.
fn lines_of(
    root: &Path,
    args: &[&str],
    undecided: impl Fn(String) -> Refusal,
) -> Result<Vec<Vec<u8>>, Refusal> {
    let out = git::run_bytes(root, args).map_err(undecided)?;
    Ok(output_lines(&out))
}

/// The lines of git's output, as `read` gives them.
fn output_lines(out: &[u8]) -> Vec<Vec<u8>> {
    crate::bash::read_lines(out)
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect()
}

/// What `review-binding` needs: the hand-off, a file named from where `nuc` runs, as the bash names
/// it from where it runs, refused before the change is resolved where it is no file; and git asked
/// as the bash of the release asks it, in its order, through [`ReviewRepo`].
fn review_binding(
    tree: &Tree,
    release: config::Version,
    args: &review_binding::Args,
) -> Result<Vec<u8>, Refusal> {
    let sensor = review_binding::SENSOR.name;
    let hand_off = match args.hand_off {
        None => None,
        Some(path) => {
            let named = if path.is_empty() {
                None
            } else {
                let caller = env::current_dir().map_err(|e| {
                    hand_off_refusal(format!("the current directory is unreadable: {e}"))
                })?;
                Some(caller.join(path))
            };
            // A file as `-f` finds it, a symbolic link followed; one that resolves outside the
            // repository is refused before any file there is tested or read.
            let file = match &named {
                Some(named) => tree.is_file_inside(named).map_err(hand_off_refusal)?,
                None => false,
            };
            match named {
                Some(named) if file => Some(named),
                _ => return Err(hand_off_refusal(format!("hand-off '{path}' is no file"))),
            }
        }
    };
    let root = &tree.root;
    let change = match resolve_change(root, sensor, review_binding::LABEL, args.base, args.head)? {
        Ok(change) => change,
        Err(report) => return Ok(report),
    };
    let given = hand_off.is_some();
    let mut repo = ReviewRepo { tree, hand_off };
    review_binding::report(&mut repo, &change.base, &change.head, given, release)
}

/// A refusal of `review-binding` keyed `file`, for the hand-off.
fn hand_off_refusal(message: String) -> Refusal {
    Refusal {
        sensor: review_binding::SENSOR.name,
        reason: "file",
        message: unwrapped(message),
    }
}

/// The repository as `review-binding` asks it: git as the bash asks it, but never through a
/// textconv driver or an external diff of the repository (Q-7). Its diffs compare commits and read
/// no working tree, so no filter runs. The diffstat keeps the colour the repository's
/// configuration gives it, since the report shows it as the bash's git writes it.
struct ReviewRepo<'a> {
    tree: &'a Tree,
    hand_off: Option<PathBuf>,
}

impl ReviewRepo<'_> {
    fn git(&self, args: &[&str]) -> Result<Vec<u8>, Refusal> {
        git::run_bytes(&self.tree.root, args).map_err(git_refusal(review_binding::SENSOR.name))
    }

    fn lines(&self, args: &[&str]) -> Result<Vec<Vec<u8>>, Refusal> {
        lines_of(
            &self.tree.root,
            args,
            git_refusal(review_binding::SENSOR.name),
        )
    }

    /// The first line of git's output as text, as a command substitution gives it.
    fn text(&self, args: &[&str]) -> Result<String, Refusal> {
        let out = self.git(args)?;
        String::from_utf8(crate::bash::substituted(&out).to_vec()).map_err(|_| {
            git_refusal(review_binding::SENSOR.name)(format!(
                "git {} wrote no UTF-8",
                args.join(" ")
            ))
        })
    }

    /// The standard output of the diff `command` with `args`, with every guard.
    fn diff(&self, command: git::Diff, args: &[&str]) -> Result<Vec<u8>, Refusal> {
        git::diff(&self.tree.root, &[], command, args)
            .map_err(git_refusal(review_binding::SENSOR.name))
    }

    /// The exit code of git, for a call the bash tests by its status alone.
    fn status(&self, args: &[&str]) -> Result<(i32, Vec<u8>), Refusal> {
        git::status_and_output(&self.tree.root, args)
            .map_err(git_refusal(review_binding::SENSOR.name))
    }
}

/// A message the sensor reads a `Reviewed at:` line from, as `$(…)` gives it to grep: its NUL
/// bytes dropped, as bash drops them, and every other byte read as itself, as grep reads it in the
/// C locale.
fn reviewable(bytes: Vec<u8>) -> Vec<u8> {
    bytes.into_iter().filter(|&b| b != 0).collect()
}

impl review_binding::Repo for ReviewRepo<'_> {
    fn commit(&mut self, rev: &str) -> Result<Option<String>, Refusal> {
        let named = format!("{rev}^{{commit}}");
        let (code, out) = self.status(&["rev-parse", "--verify", "-q", &named])?;
        Ok((code == 0)
            .then(|| String::from_utf8_lossy(crate::bash::substituted(&out)).into_owned()))
    }

    fn full(&mut self, rev: &str) -> Result<String, Refusal> {
        self.text(&["rev-parse", rev])
    }

    fn short(&mut self, rev: &str) -> Result<String, Refusal> {
        self.text(&["rev-parse", "--short", rev])
    }

    fn is_ancestor(&mut self, ancestor: &str, rev: &str) -> Result<bool, Refusal> {
        let (code, _) = self.status(&["merge-base", "--is-ancestor", ancestor, rev])?;
        Ok(code == 0)
    }

    fn rev_list(&mut self, options: &[&str], range: &str) -> Result<Vec<Vec<u8>>, Refusal> {
        let mut args = vec!["rev-list"];
        args.extend(options);
        args.push(range);
        self.lines(&args)
    }

    fn count(&mut self, options: &[&str], range: &str) -> Result<u64, Refusal> {
        let mut args = vec!["rev-list", "--count"];
        args.extend(options);
        args.push(range);
        let count = self.text(&args)?;
        count.parse().map_err(|_| {
            git_refusal(review_binding::SENSOR.name)(format!(
                "git rev-list --count wrote '{count}'"
            ))
        })
    }

    fn subjects(&mut self, range: &str) -> Result<Vec<Vec<u8>>, Refusal> {
        self.lines(&["log", "--format=%h %s", range])
    }

    fn subject(&mut self, commit: &str) -> Result<Vec<u8>, Refusal> {
        let out = self.git(&["log", "-1", "--format=%h %s", commit])?;
        Ok(crate::bash::substituted(&out).to_vec())
    }

    fn message(&mut self, commit: &str) -> Result<Vec<u8>, Refusal> {
        Ok(reviewable(self.git(&[
            "log",
            "-1",
            "--format=%B",
            commit,
        ])?))
    }

    fn stat(&mut self, from: &str, to: &str) -> Result<Vec<Vec<u8>>, Refusal> {
        self.diff(git::Diff::Diff, &["--stat", from, to])
            .map(|out| output_lines(&out))
    }

    fn patch(&mut self, commit: &str, pathspecs: &[&str]) -> Result<Vec<u8>, Refusal> {
        let mut args = vec![
            "-r",
            "--root",
            "--no-commit-id",
            "-p",
            "-U0",
            "--no-color",
            commit,
            "--",
        ];
        args.extend(pathspecs);
        self.diff(git::Diff::DiffTree, &args)
    }

    fn outside_record(&mut self, commit: &str) -> Result<Vec<Vec<u8>>, Refusal> {
        let mut args = vec![
            "-r",
            "--root",
            "--no-commit-id",
            "--name-only",
            commit,
            "--",
        ];
        args.extend(review_binding::OUTSIDE_RECORD);
        self.diff(git::Diff::DiffTree, &args)
            .map(|out| output_lines(&out))
    }

    fn hand_off(&mut self) -> Result<Vec<u8>, Refusal> {
        let path = self
            .hand_off
            .as_ref()
            .expect("read only where one is given");
        let bytes = self
            .tree
            .read_bytes(path)
            .map_err(hand_off_refusal)?
            .ok_or_else(|| hand_off_refusal(format!("{}: no longer a file", path.display())))?;
        Ok(reviewable(bytes))
    }
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
        let added = git::diff(
            root,
            &[],
            git::Diff::Diff,
            &[
                "--no-renames",
                "--name-only",
                "--diff-filter=A",
                &base,
                &name,
                "--",
                &dir,
            ],
        )
        .map_err(undecided)?;
        let added = output_lines(&added);
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
