//! The fixture repositories of the tests (ADR-0011): each a fresh directory
//! of its own under the system's temporary directory, outside this repository, so that no git
//! command of a test can reach it, built through the real git CLI.
//! git and `nuc` run under a cleared environment with the allowlist the shell passes, a fixed
//! author, committer, and date, and the hooks path on the null device, so that neither the
//! parent's `GIT_*` variables nor the user's identity or hooks change a fixture.

// Each test crate includes this module and uses part of it.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The variables passed from the parent, as the shell passes them.
const PASSED: &[&str] = &["PATH", "HOME", "SystemRoot"];

/// The variables set for every process a fixture runs.
const SET: &[(&str, &str)] = &[
    ("LC_ALL", "C"),
    ("GIT_TERMINAL_PROMPT", "0"),
    ("GIT_OPTIONAL_LOCKS", "0"),
    ("GIT_AUTHOR_NAME", "Fixture"),
    ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
    ("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z"),
    ("GIT_COMMITTER_NAME", "Fixture"),
    ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z"),
];

#[cfg(not(windows))]
pub const NULL_DEVICE: &str = "/dev/null";
#[cfg(windows)]
pub const NULL_DEVICE: &str = "NUL";

/// The `nuc` binary cargo built for the tests.
pub const NUC: &str = env!("CARGO_BIN_EXE_nuc");

/// A `nuc.toml` that turns on every rule this `nuc` knows, at the lowest minimum.
pub const CONFIG: &str = "minimum = \"0.0.0\"\nrules = [\"docs/template-release\"]\n";

/// A README whose Template release line names a release.
pub const README: &str = "# Fixture\n\n- **Template release:** v0.5.0\n";

/// The command with the environment cleared and the allowlist and the fixed values set.
pub fn cleared(command: &mut Command) -> &mut Command {
    command.env_clear();
    for key in PASSED {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command.envs(SET.iter().copied()).stdin(Stdio::null())
}

/// git for `args` on the repository at `dir`, which must succeed.
pub fn git_in(dir: &Path, args: &[&str]) -> Output {
    let output = cleared(&mut Command::new("git"))
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            &format!("core.hooksPath={NULL_DEVICE}"),
            "-c",
            "core.fsmonitor=false",
            // No maintenance in the background, which would change a fixture after its snapshot.
            "-c",
            "maintenance.auto=false",
            "-c",
            "gc.auto=0",
        ])
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        stderr(&output)
    );
    output
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A git repository of its own, removed when it is dropped.
pub struct Fixture {
    root: PathBuf,
}

impl Fixture {
    /// An empty repository on `main`, under a directory named after `name`.
    pub fn new(name: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir()
            .join("nuc-tests")
            .join(format!("{name}-{}-{n}", process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("the fixture directory is created");
        let fixture = Self { root };
        fixture.git(&["init", "-q", "-b", "main"]);
        fixture
    }

    /// A repository with a `nuc.toml` and a README, committed.
    pub fn project(name: &str, config: &str, readme: &str) -> Self {
        let fixture = Self::new(name);
        fixture
            .write("nuc.toml", config)
            .write("README.md", readme)
            .commit("Initial commit");
        fixture
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Write the file at the path relative to the root, its directories included.
    pub fn write(&self, path: &str, contents: &str) -> &Self {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().expect("a file has a parent")).expect("directories");
        fs::write(path, contents).expect("the file is written");
        self
    }

    pub fn git(&self, args: &[&str]) -> Output {
        git_in(&self.root, args)
    }

    /// Stage everything and commit it.
    pub fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]);
    }

    /// `nuc` with `args`, run in `dir` under the root, under the cleared environment.
    pub fn nuc_in(&self, dir: &str, args: &[&str]) -> Output {
        cleared(&mut Command::new(NUC))
            .current_dir(self.root.join(dir))
            .args(args)
            .output()
            .expect("nuc runs")
    }

    /// `nuc` with `args`, run at the root.
    pub fn nuc(&self, args: &[&str]) -> Output {
        self.nuc_in("", args)
    }

    /// `nuc` with `args`, run at the root under the cleared environment and the variables given.
    pub fn nuc_with(&self, vars: &[(&str, &str)], args: &[&str]) -> Output {
        cleared(&mut Command::new(NUC))
            .current_dir(&self.root)
            .envs(vars.iter().copied())
            .args(args)
            .output()
            .expect("nuc runs")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Every file under `dir` with its contents, sorted, to compare a tree before and after.
pub fn snapshot(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in fs::read_dir(&next).expect("the directory is readable") {
            let path = entry.expect("an entry is readable").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let contents = fs::read(&path).expect("the file is readable");
                files.push((path, contents));
            }
        }
    }
    files.sort();
    files
}

/// The paths that differ between two snapshots: added, removed, or changed.
pub fn changed(before: &[(PathBuf, Vec<u8>)], after: &[(PathBuf, Vec<u8>)]) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = before
        .iter()
        .filter(|entry| !after.contains(entry))
        .chain(after.iter().filter(|entry| !before.contains(entry)))
        .map(|(path, _)| path.clone())
        .collect();
    paths.sort();
    paths.dedup();
    paths
}
