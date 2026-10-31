//! git, spawned with a cleared environment into which an allowlist is passed, the repository
//! named by `-C`, and the keys that would run a command neutralized by `-c`
//! (ADR-0010).

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
/// every call. The hooks path is the null device, which can hold no hook.
#[cfg(not(windows))]
pub const NEUTRALIZED: &[&str] = &["core.fsmonitor=false", "core.hooksPath=/dev/null"];
#[cfg(windows)]
pub const NEUTRALIZED: &[&str] = &["core.fsmonitor=false", "core.hooksPath=NUL"];

/// The git command for `args` on the repository at `repo`, ready to spawn.
pub fn command(repo: &Path, args: &[&str]) -> Command {
    let mut git = Command::new("git");
    git.env_clear();
    for key in PASSED {
        if let Some(value) = env::var_os(key) {
            git.env(key, value);
        }
    }
    git.envs(SET.iter().copied());
    git.arg("-C").arg(repo);
    for key in NEUTRALIZED {
        git.arg("-c").arg(key);
    }
    git.args(args).stdin(Stdio::null());
    git
}

/// The standard output of git for `args` on `repo`, or why it could not be had.
pub fn run(repo: &Path, args: &[&str]) -> Result<String, String> {
    String::from_utf8(run_bytes(repo, args)?)
        .map_err(|_| format!("git {} wrote no UTF-8", args.join(" ")))
}

/// The standard output of git for `args` on `repo` as bytes, or why it could not be had.
pub fn run_bytes(repo: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = command(repo, args)
        .output()
        .map_err(|e| format!("git could not be run ({e}) — install git and put it on the PATH"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git {} failed: {}", args.join(" "), stderr.trim()));
    }
    Ok(output.stdout)
}

/// The root of the working tree that contains `dir`.
pub fn toplevel(dir: &Path) -> Result<PathBuf, String> {
    let out = run(dir, &["rev-parse", "--show-toplevel"])
        .map_err(|e| format!("{} is not inside a git working tree: {e}", dir.display()))?;
    Ok(PathBuf::from(out.trim_end_matches(['\n', '\r'])))
}
