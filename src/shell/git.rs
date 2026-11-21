//! git, spawned with a cleared environment into which an allowlist is passed, the repository
//! named by `-C`, and the keys that would run a command neutralized by `-c`
//! (SEC-git-cleared-without-lazy-fetch); every diff through [`diff`], with the flags that keep the
//! repository's diff drivers from running (SEC-repository-commands-neutralized-or-refused).

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};

/// The variables passed from the parent, where it has them.
#[cfg(not(windows))]
const PASSED: &[&str] = &["PATH", "HOME"];
#[cfg(windows)]
const PASSED: &[&str] = &["PATH", "HOME", "SystemRoot"];

/// The variables `nuc` sets itself.
const SET: &[(&str, &str)] = &[
    ("LC_ALL", "C"),
    ("GIT_TERMINAL_PROMPT", "0"),
    ("GIT_OPTIONAL_LOCKS", "0"),
];

/// The configuration keys that would make a git command `nuc` runs start another command, set on
/// every call. The hooks path is the null device, which can hold no hook. No transport is allowed,
/// so that a partial clone's lazy fetch of a missing object, which `show`, `diff`, and `cat-file`
/// would start, runs no command the repository's configuration names, such as `core.sshCommand`
/// (Q-7, Q-11). A diff shows a submodule's change as one line, git's default: `diff.submodule=diff`
/// or `log` would have it start a git in the submodule, which reads no flag of nuc's diff and
/// runs the external diff and the textconv drivers the configuration names.
#[cfg(not(windows))]
pub const NEUTRALIZED: &[&str] = &[
    "core.fsmonitor=false",
    "core.hooksPath=/dev/null",
    "diff.submodule=short",
    "log.showSignature=false",
    "protocol.allow=never",
];
#[cfg(windows)]
pub const NEUTRALIZED: &[&str] = &[
    "core.fsmonitor=false",
    "core.hooksPath=NUL",
    "diff.submodule=short",
    "log.showSignature=false",
    "protocol.allow=never",
];

/// The earliest git that knows `--no-lazy-fetch` (SEC-git-cleared-without-lazy-fetch).
pub const NO_LAZY_FETCH: (u64, u64, u64) = (2, 45, 0);

/// The git command for `args` on the repository at `repo`, ready to spawn. A git that knows
/// `--no-lazy-fetch` gets it on every call, so that a partial clone fetches no object it lacks
/// through a transport the repository configures (Q-7, Q-11). An older git runs only on a
/// repository that names no promisor remote, the one kind of repository that fetches lazily; on
/// one that does, the run is not decided (SEC-git-cleared-without-lazy-fetch).
pub fn command(repo: &Path, args: &[&str]) -> Result<Command, String> {
    let blocks = blocks_lazy_fetch()?;
    if !blocks {
        refuse_promisor(repo)?;
    }
    let mut git = bare(repo, blocks);
    git.args(args).stdin(Stdio::null());
    Ok(git)
}

/// git with the cleared environment, the repository, and the neutralized keys, its arguments to
/// come; `--no-lazy-fetch` first where `blocks` says git knows it.
fn bare(repo: &Path, blocks: bool) -> Command {
    let mut git = cleared();
    if blocks {
        git.arg("--no-lazy-fetch");
    }
    git.arg("-C").arg(repo);
    for key in NEUTRALIZED {
        git.arg("-c").arg(key);
    }
    git
}

/// git under the cleared environment: the allowlist passed, the variables `nuc` sets set, and
/// nothing else, for every call and for the probe of its version.
fn cleared() -> Command {
    let mut git = Command::new("git");
    git.env_clear();
    for key in PASSED {
        if let Some(value) = env::var_os(key) {
            git.env(key, value);
        }
    }
    git.envs(SET.iter().copied());
    git
}

/// Whether the git on the `PATH` knows `--no-lazy-fetch`, asked once a run: `git --version`, read
/// as its first three numbers, as git for Windows and Apple's git print them too.
fn blocks_lazy_fetch() -> Result<bool, String> {
    static BLOCKS: OnceLock<Result<bool, String>> = OnceLock::new();
    BLOCKS
        .get_or_init(|| {
            let mut git = cleared();
            git.arg("--version").stdin(Stdio::null());
            let output = git.output().map_err(|e| {
                format!("git could not be run ({e}) — install git and put it on the PATH")
            })?;
            let text = String::from_utf8_lossy(&output.stdout);
            let version = parse_version(&text)
                .ok_or_else(|| format!("git --version wrote '{}' — not decided", text.trim()))?;
            Ok(version >= NO_LAZY_FETCH)
        })
        .clone()
}

/// The first three numbers of `git version X.Y.Z…`.
fn parse_version(text: &str) -> Option<(u64, u64, u64)> {
    let rest = text.trim().strip_prefix("git version ")?;
    let mut parts = rest.split(|c: char| !c.is_ascii_digit());
    let mut next = || parts.next()?.parse().ok();
    Some((next()?, next()?, next()?))
}

/// Not decided where the repository names a promisor remote, for a git that cannot be kept from
/// fetching what such a clone lacks.
fn refuse_promisor(repo: &Path) -> Result<(), String> {
    // Each repository is asked once a run.
    static ASKED: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
    let mut asked = ASKED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if asked.iter().any(|known| known == repo) {
        return Ok(());
    }
    let output = bare(repo, false)
        .args([
            "config",
            "--get-regexp",
            r"^(extensions\.partialclone|remote\..*\.promisor)$",
        ])
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("git could not be run ({e}) — install git and put it on the PATH"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let promisor = text.lines().any(|line| {
        let (key, value) = line.split_once(' ').unwrap_or((line, ""));
        key == "extensions.partialclone" || value.trim() != "false"
    });
    if promisor {
        let (x, y, z) = NO_LAZY_FETCH;
        return Err(format!(
            "{}: a partial clone, whose objects this git would fetch through a transport the \
             repository configures — nuc needs git {x}.{y}.{z} or later for it, which knows \
             --no-lazy-fetch — not decided",
            repo.display()
        ));
    }
    asked.push(repo.to_path_buf());
    Ok(())
}

/// The variables that hand git configuration on, or move where git reads it: what `git -c` sets,
/// and the global and system files (SEC-repository-commands-neutralized-or-refused).
const CONFIGURING: &[&str] = &[
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "XDG_CONFIG_HOME",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_CONFIG_NOSYSTEM",
];

/// Why the caller's environment names a command git would run on a diff, a filtered file, or a
/// hash of the working tree that `nuc`'s git, under the cleared environment, does not see; none
/// where it names none (SEC-repository-commands-neutralized-or-refused). Asked once a run.
///
/// `GIT_EXTERNAL_DIFF` names one wherever it is set. `GIT_ATTR_SOURCE` does as well: it moves the
/// attributes to a tree, which `nuc` does not read, a wider refusal the ADR allows. For the rest,
/// git itself reads its configuration twice, under the variables of [`CONFIGURING`] the caller set
/// and without them, its includes followed: an entry of a filter or diff driver, `diff.external`,
/// or `diff.submodule` that only one of the two lists names one. So does a tracked path whose
/// `filter` or `diff` attribute git tells otherwise under them, naming a driver with a command on
/// either side; a file the index lacks is held where a command reads it, by
/// [`attributes_under_caller`] and [`refuse_attributes_under_caller`]. A configuration git cannot
/// read under the caller's variables names one too, the side that never passes. The answer is kept
/// for the run, which reads one repository.
pub fn environment_driver(repo: &Path) -> Result<Option<String>, String> {
    static ASKED: OnceLock<Result<Option<String>, String>> = OnceLock::new();
    ASKED.get_or_init(|| ask_environment(repo)).clone()
}

/// Not decided, with `why`, where the caller's environment names a driver nuc's git does not see.
pub fn refuse_environment(repo: &Path) -> Result<(), String> {
    match environment_driver(repo)? {
        Some(why) => Err(format!("{why} — not decided")),
        None => Ok(()),
    }
}

fn ask_environment(repo: &Path) -> Result<Option<String>, String> {
    for name in ["GIT_EXTERNAL_DIFF", "GIT_ATTR_SOURCE"] {
        if env::var_os(name).is_some() {
            return Ok(Some(format!(
                "{name} is set in the environment, which the bash's git reads and nuc's does not"
            )));
        }
    }
    let carried = carried();
    if carried.is_empty() {
        return Ok(None);
    }
    let names = carried
        .iter()
        .map(|(name, _)| name.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(", ");
    let ours = configured(repo, &[])?
        .ok_or_else(|| "git config could not read nuc's own configuration".to_owned())?;
    let Some(theirs) = configured(repo, &carried)? else {
        return Ok(Some(format!(
            "{names}: a git configuration of the environment that git cannot read"
        )));
    };
    let mut seen = ours.clone();
    for (key, value) in &theirs {
        if let Some(at) = seen
            .iter()
            .position(|entry| entry == &(key.clone(), value.clone()))
        {
            seen.remove(at);
            continue;
        }
        if names_driver(key) {
            return Ok(Some(format!(
                "{}: configured by {names} in the environment, which the bash's git reads and \
                 nuc's does not",
                String::from_utf8_lossy(key)
            )));
        }
    }
    // A filter nuc's git sees that the caller's variables take away: the bash's git hashes raw
    // where nuc's runs the filter. A diff driver taken away changes no answer, since nuc's diffs
    // run none.
    if let Some((key, _)) = seen
        .iter()
        .find(|(key, _)| key.starts_with(b"filter.") && names_driver(key))
    {
        return Ok(Some(format!(
            "{}: taken away by {names} in the environment, which the bash's git reads and nuc's \
             does not",
            String::from_utf8_lossy(key)
        )));
    }
    let tracked = run_bytes(repo, &["ls-files", "-z"])?;
    if let Some(path) = attributed_driver(repo, &carried, &ours, &theirs, &tracked)? {
        return Ok(Some(format!(
            "{}: {names} in the environment give it an attribute that names a filter or a diff \
             driver, which the bash's git runs and nuc's does not see",
            String::from_utf8_lossy(&path)
        )));
    }
    Ok(None)
}

/// The variables of [`CONFIGURING`] and the entries of `GIT_CONFIG_COUNT` the caller set.
fn carried() -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    env::vars_os()
        .filter(|(name, _)| {
            let name = name.to_string_lossy();
            CONFIGURING.contains(&name.as_ref())
                || name.starts_with("GIT_CONFIG_KEY_")
                || name.starts_with("GIT_CONFIG_VALUE_")
        })
        .collect()
}

/// The output of `git check-attr -z --stdin <attributes>` for the paths of `input`, NUL-ended, as
/// git tells them under the caller's variables of [`CONFIGURING`]; none where the caller set none.
/// A path a range of commits reads that the index lacks is told here, where
/// [`environment_driver`], which reads the tracked paths, does not reach it.
pub fn attributes_under_caller(
    repo: &Path,
    attributes: &[&str],
    input: &[u8],
) -> Result<Option<Vec<u8>>, String> {
    let carried = carried();
    if carried.is_empty() {
        return Ok(None);
    }
    let mut git = bare(repo, blocks_lazy_fetch()?);
    git.envs(carried.iter().map(|(name, value)| (name, value)));
    let mut args = vec!["check-attr", "-z", "--stdin"];
    args.extend(attributes);
    git.args(&args);
    with_input(git, &args, input).map(Some)
}

/// Entries of git's configuration, `(key, value)` in the order git lists them.
type Entries = Vec<(Vec<u8>, Vec<u8>)>;

/// Whether a configuration key names a command git runs on a diff, a filtered file, or a hash of
/// the working tree, as git writes the key, its section and variable in lower case.
fn names_driver(key: &[u8]) -> bool {
    if key.starts_with(b"filter.") || key == b"diff.external" || key == b"diff.submodule" {
        return true;
    }
    key.starts_with(b"diff.") && (key.ends_with(b".textconv") || key.ends_with(b".command"))
}

/// The entries of git's configuration that name a filter or a diff driver, `(key, value)` in
/// the order git lists them, read under the cleared environment and the variables of `carried`;
/// none where git cannot read it.
fn configured(
    repo: &Path,
    carried: &[(std::ffi::OsString, std::ffi::OsString)],
) -> Result<Option<Entries>, String> {
    let mut git = bare(repo, blocks_lazy_fetch()?);
    git.envs(carried.iter().map(|(name, value)| (name, value)));
    git.args(["config", "-z", "--get-regexp", r"^(filter\.|diff\.)"])
        .stdin(Stdio::null());
    let output = git
        .output()
        .map_err(|e| format!("git could not be run ({e}) — install git and put it on the PATH"))?;
    match output.status.code() {
        Some(0 | 1) => {}
        _ if carried.is_empty() => return Err(failure(&["config"], &output.stderr)),
        _ => return Ok(None),
    }
    // `<key> LF <value> NUL` an entry.
    let entries = output
        .stdout
        .split(|&b| b == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| match entry.iter().position(|&b| b == b'\n') {
            Some(at) => (entry[..at].to_vec(), entry[at + 1..].to_vec()),
            None => (entry.to_vec(), Vec::new()),
        })
        .collect();
    Ok(Some(entries))
}

/// Not decided where git tells a `filter` or `diff` attribute of one of `paths` otherwise under the
/// caller's variables than without them, naming a driver with a command on either side, as for
/// the files `nuc check` hashes, which the index may lack (SEC-repository-commands-neutralized-or-
/// refused).
pub fn refuse_attributes_under_caller(repo: &Path, paths: &[&str]) -> Result<(), String> {
    let carried = carried();
    if carried.is_empty() || paths.is_empty() {
        return Ok(());
    }
    let ours = configured(repo, &[])?
        .ok_or_else(|| "git config could not read nuc's own configuration".to_owned())?;
    let Some(theirs) = configured(repo, &carried)? else {
        return Err(
            "a git configuration of the environment that git cannot read — not decided".to_owned(),
        );
    };
    let mut input = Vec::new();
    for path in paths {
        input.extend_from_slice(path.as_bytes());
        input.push(0);
    }
    match attributed_driver(repo, &carried, &ours, &theirs, &input)? {
        Some(path) => Err(format!(
            "{}: the caller's environment gives it another attribute for a filter or a diff \
             driver, so the bash's git hashes it otherwise — not decided",
            String::from_utf8_lossy(&path)
        )),
        None => Ok(()),
    }
}

/// The first of the NUL-ended `paths` whose `filter` or `diff` attribute git tells otherwise under
/// the caller's variables of `carried` than without them, where either value names a driver its
/// side's configuration, `ours` or `theirs`, gives a command: an attributes file they have git
/// read or take away, its macros resolved by git as git resolves them. A value that names no
/// driver with a command, as `binary` does, runs no command; what it changes in a diff is
/// configuration that names none, which the ADR leaves out of scope.
fn attributed_driver(
    repo: &Path,
    carried: &[(std::ffi::OsString, std::ffi::OsString)],
    ours_config: &Entries,
    theirs: &Entries,
    tracked: &[u8],
) -> Result<Option<Vec<u8>>, String> {
    if tracked.is_empty() {
        return Ok(None);
    }
    let attributes = |carried: &[(std::ffi::OsString, std::ffi::OsString)]| {
        let mut git = bare(repo, blocks_lazy_fetch()?);
        git.envs(carried.iter().map(|(name, value)| (name, value)));
        let args = ["check-attr", "-z", "--stdin", "filter", "diff"];
        git.args(args);
        with_input(git, &args, tracked)
    };
    let ours = attributes(&[])?;
    let theirs_attributes = attributes(carried)?;
    // `<path> NUL <attribute> NUL <value> NUL` for each path and attribute, in the order given.
    let ours: Vec<&[u8]> = ours.split(|&b| b == 0).collect();
    let mine: Vec<&[u8]> = theirs_attributes.split(|&b| b == 0).collect();
    for (our, their) in ours.chunks(3).zip(mine.chunks(3)) {
        let ([_, _, our_value], [path, attribute, value]) = (our, their) else {
            continue;
        };
        if our_value == value {
            continue;
        }
        let variables: &[&[u8]] = match *attribute {
            b"filter" => &[b"clean", b"process"],
            b"diff" => &[b"textconv", b"command"],
            _ => continue,
        };
        let names = |config: &Entries, value: &[u8]| {
            config.iter().any(|(key, _)| {
                variables
                    .iter()
                    .any(|variable| *key == [&attribute[..], b".", value, b".", variable].concat())
            })
        };
        if names(theirs, value) || names(ours_config, our_value) {
            return Ok(Some(path.to_vec()));
        }
    }
    Ok(None)
}

/// The active filters of the repository: the keys that keep git from running one, and the tracked
/// paths under one that git would run (SEC-repository-commands-neutralized-or-refused).
#[derive(Debug, Default)]
pub struct Filters {
    pub keys: Vec<String>,
    pub paths: std::collections::BTreeSet<Vec<u8>>,
}

/// The keys that keep git from running a filter, and the tracked paths under an active one. git
/// runs a filter only for a driver that any entry of the configuration gives a `clean` or `process`
/// command, not empty, and the attribute's value names the driver. It runs the filter of a
/// stat-dirty file anywhere in the tree, whatever the pathspec, so every active driver gets an
/// empty `process` command, which keeps git from running both it and the `clean` command, and
/// `required` off, so that git does not fail for want of either (Q-7). A driver no entry makes
/// active runs no command and is left as the bash's git reads it. An active driver whose name holds
/// a `=`, which `-c` would split there and so neutralize another key, or is no UTF-8, is not
/// decided: the inner error names it. `check-attr` prints `set` both for a bare `filter`, which
/// names no driver, and for a driver named `set`, so such a path counts as under a filter only
/// where a driver of that name is active, the side that never passes.
pub fn filters(root: &Path) -> Result<Result<Filters, String>, String> {
    let mut filters = Filters::default();
    // `<key> LF <value> NUL` for each key; none, and exit 1, where the configuration has none.
    let (code, config) = status_and_output(root, &["config", "-z", "--get-regexp", r"^filter\."])?;
    if code > 1 {
        return Err("git config --get-regexp failed".to_owned());
    }
    let active = crate::sense::plain_rules::filter_drivers(&config);
    for name in &active {
        let Some(name) = String::from_utf8(name.clone())
            .ok()
            .filter(|name| !name.contains('='))
        else {
            return Ok(Err(format!(
                "'{}': a filter driver whose name nuc cannot neutralize",
                String::from_utf8_lossy(name)
            )));
        };
        for key in ["process=", "required=false"] {
            filters.keys.push(format!("filter.{name}.{key}"));
        }
    }
    if active.is_empty() {
        return Ok(Ok(filters));
    }
    let tracked = run_bytes(root, &["ls-files", "-z"])?;
    if tracked.is_empty() {
        return Ok(Ok(filters));
    }
    let attrs = run_bytes_with_input(root, &["check-attr", "-z", "--stdin", "filter"], &tracked)?;
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
    Ok(Ok(filters))
}

/// How a tracked path under an active filter may differ from the index to a git that runs the
/// filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dirty {
    /// Its stat differs from the index, whatever its raw bytes.
    Changed,
    /// Its stat matches, but its modification time is no older than the index's, so git compares
    /// its content.
    Racy,
}

/// The first tracked path under an active filter that a git running the filter may read otherwise
/// than the index, and how. `diff-files` names the stat-dirty ones by their stat alone, under the
/// keys that keep it from running a filter on a racily clean entry it hashes; a racily clean one
/// it names only where the raw bytes differ, so each path's modification time is compared with the
/// index's in whole seconds, as git compares them where it does not compare finer, which errs
/// toward counting (SEC-repository-commands-neutralized-or-refused).
pub fn dirty_filtered(root: &Path, filters: &Filters) -> Result<Option<(Vec<u8>, Dirty)>, String> {
    if filters.paths.is_empty() {
        return Ok(None);
    }
    let mut args: Vec<&str> = filters
        .keys
        .iter()
        .flat_map(|key| ["-c", key.as_str()])
        .collect();
    args.extend(["diff-files", "-z", "--name-only"]);
    let dirty = run_bytes(root, &args)?;
    let dirty: std::collections::BTreeSet<&[u8]> = dirty.split(|&b| b == 0).collect();
    let index = run(root, &["rev-parse", "--git-path", "index"])?;
    let index = mtime_secs(&root.join(index.trim_end_matches(['\n', '\r'])));
    for path in &filters.paths {
        if dirty.contains(path.as_slice()) {
            return Ok(Some((path.clone(), Dirty::Changed)));
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
            return Ok(Some((path.clone(), Dirty::Racy)));
        }
    }
    Ok(None)
}

/// The first submodule the index records that is checked out, its directory holding a `.git`,
/// asked before any git looks into it: a status would ask the submodule's own, which runs its
/// filters (Q-7). One whose path the platform cannot spell counts as checked out, the side that
/// never passes.
pub fn checked_out_submodule(root: &Path) -> Result<Option<Vec<u8>>, String> {
    let staged = run_bytes(root, &["ls-files", "-z", "--stage"])?;
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
        if relative_path(root, &dot_git).is_none_or(|dot_git| dot_git.exists()) {
            return Ok(Some(path.to_vec()));
        }
    }
    Ok(None)
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

/// The standard output of git for `args` on `repo`, or why it could not be had.
pub fn run(repo: &Path, args: &[&str]) -> Result<String, String> {
    String::from_utf8(run_bytes(repo, args)?)
        .map_err(|_| format!("git {} wrote no UTF-8", args.join(" ")))
}

/// The standard output of git for `args` on `repo` as bytes, or why it could not be had.
pub fn run_bytes(repo: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = command(repo, args)?
        .output()
        .map_err(|e| format!("git could not be run ({e}) — install git and put it on the PATH"))?;
    if !output.status.success() {
        return Err(failure(args, &output.stderr));
    }
    Ok(output.stdout)
}

/// The git commands that show a diff, which the repository's own commands could otherwise produce
/// (SEC-repository-commands-neutralized-or-refused).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Diff {
    /// `git diff`.
    Diff,
    /// `git diff-tree`.
    DiffTree,
    /// `git log`, run with `-p` or `-G`.
    Log,
}

/// The flags every diff of nuc carries: without them, git would run the external diff and the
/// textconv drivers the repository's configuration names (Q-7).
const DIFF_GUARDS: &[&str] = &["--no-ext-diff", "--no-textconv"];

/// The arguments of a diff call: `-c` and each key of `config` before the command, so that one
/// that would run a filter driver is neutralized, the command, the guards, then `args`.
fn diff_args<'a>(config: &[&'a str], command: Diff, args: &[&'a str]) -> Vec<&'a str> {
    let mut all: Vec<&str> = config.iter().flat_map(|key| ["-c", key]).collect();
    all.push(match command {
        Diff::Diff => "diff",
        Diff::DiffTree => "diff-tree",
        Diff::Log => "log",
    });
    all.extend(DIFF_GUARDS);
    all.extend(args);
    all
}

/// The standard output of the diff `command` with `args` on `repo` as bytes, under the
/// configuration keys of `config`, or why it could not be had. Every diff, `diff-tree`, `log -p`,
/// and `log -G` of nuc starts here, so that each carries the guards
/// (SEC-repository-commands-neutralized-or-refused).
pub fn diff(repo: &Path, config: &[&str], command: Diff, args: &[&str]) -> Result<Vec<u8>, String> {
    run_bytes(repo, &diff_args(config, command, args))
}

/// The standard output of git for `args` on `repo` as bytes, the input written to its standard
/// input, as `--stdin` reads it; or why it could not be had.
pub fn run_bytes_with_input(repo: &Path, args: &[&str], input: &[u8]) -> Result<Vec<u8>, String> {
    with_input(command(repo, args)?, args, input)
}

/// The standard output of the git command `git`, its arguments `args`, the input written to its
/// standard input; or why it could not be had.
fn with_input(mut git: Command, args: &[&str], input: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Write;
    let unrun = |e: std::io::Error| {
        format!("git could not be run ({e}) — install git and put it on the PATH")
    };
    let mut child = git
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(unrun)?;
    let mut stdin = child.stdin.take().expect("the standard input is piped");
    let input = input.to_vec();
    // Written beside the read, so that neither side waits on a full pipe.
    let writer = std::thread::spawn(move || stdin.write_all(&input));
    let output = child.wait_with_output().map_err(unrun)?;
    let written = writer.join().expect("the writer of the input ends");
    if !output.status.success() {
        return Err(failure(args, &output.stderr));
    }
    written.map_err(|e| format!("git {} could not read its input: {e}", args.join(" ")))?;
    Ok(output.stdout)
}

/// The root of the working tree that contains `dir`.
pub fn toplevel(dir: &Path) -> Result<PathBuf, String> {
    // A refusal of the repository itself, as of a partial clone on an older git, says so alone.
    let out = run(dir, &["rev-parse", "--show-toplevel"]).map_err(|e| {
        if e.ends_with("not decided") {
            e
        } else {
            format!("{} is not inside a git working tree: {e}", dir.display())
        }
    })?;
    Ok(PathBuf::from(out.trim_end_matches(['\n', '\r'])))
}

/// The standard output of git for `args` on `repo` where it exits 0, none where it exits 1, as
/// `symbolic-ref -q` on a detached `HEAD` and `rev-parse -q --verify` on a missing ref do, or why
/// it could not be had.
pub fn probe(repo: &Path, args: &[&str]) -> Result<Option<String>, String> {
    let output = command(repo, args)?
        .output()
        .map_err(|e| format!("git could not be run ({e}) — install git and put it on the PATH"))?;
    match output.status.code() {
        Some(0) => String::from_utf8(output.stdout)
            .map(Some)
            .map_err(|_| format!("git {} wrote no UTF-8", args.join(" "))),
        Some(1) => Ok(None),
        _ => Err(failure(args, &output.stderr)),
    }
}

/// The exit code of git for `args` on `repo` and its standard output as bytes, whatever the code,
/// as for `git merge-file`, which exits with the count of its conflicts; or why git could not be
/// run or was ended by a signal.
pub fn status_and_output(repo: &Path, args: &[&str]) -> Result<(i32, Vec<u8>), String> {
    let output = command(repo, args)?
        .output()
        .map_err(|e| format!("git could not be run ({e}) — install git and put it on the PATH"))?;
    let code = output
        .status
        .code()
        .ok_or_else(|| format!("git {} was ended by a signal", args.join(" ")))?;
    Ok((code, output.stdout))
}

/// Why a git call failed, from what git wrote on its standard error.
fn failure(args: &[&str], stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    format!("git {} failed: {}", args.join(" "), stderr.trim())
}

/// The blob at the path in the revision, as `git show <rev>:<path>` shows it: none where the
/// revision's tree has no such path or the path names no blob, such as a directory or a submodule;
/// not decided where it names a blob the repository lacks, as a partial clone does, since nuc
/// fetches none (SEC-git-cleared-without-lazy-fetch).
pub fn blob_at(repo: &Path, rev: &str, path: &str) -> Result<Option<Vec<u8>>, String> {
    let entry = run_bytes(repo, &["ls-tree", "-z", "--full-tree", rev, "--", path])?;
    // '<mode> <type> <object>\t<path>\0', the one entry the path names.
    let entry = entry.split(|&b| b == 0).next().unwrap_or_default();
    let head = entry.split(|&b| b == b'\t').next().unwrap_or_default();
    let fields: Vec<&[u8]> = head.split(|&b| b == b' ').collect();
    let [_, kind, object] = fields.as_slice() else {
        return Ok(None);
    };
    if *kind != b"blob" {
        return Ok(None);
    }
    let object = String::from_utf8_lossy(object).into_owned();
    let (code, _) = status_and_output(repo, &["cat-file", "-e", &object])?;
    if code != 0 {
        return Err(format!(
            "{rev}:{path}: the object {object} is not in the repository, as in a partial clone, and \
             nuc fetches none — fetch it first — not decided"
        ));
    }
    run_bytes(repo, &["cat-file", "blob", &object]).map(Some)
}
