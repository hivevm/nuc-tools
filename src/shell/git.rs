//! git, spawned with a cleared environment into which an allowlist is passed, the repository
//! named by `-C`, and the keys that would run a command neutralized by `-c`
//! (SEC-git-cleared-without-lazy-fetch).

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
/// (Q-7, Q-11).
#[cfg(not(windows))]
pub const NEUTRALIZED: &[&str] = &[
    "core.fsmonitor=false",
    "core.hooksPath=/dev/null",
    "log.showSignature=false",
    "protocol.allow=never",
];
#[cfg(windows)]
pub const NEUTRALIZED: &[&str] = &[
    "core.fsmonitor=false",
    "core.hooksPath=NUL",
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
