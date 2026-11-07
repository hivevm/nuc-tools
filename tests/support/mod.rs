//! The fixture repositories of the tests
//! (ADR-draft-fixture-repositories-with-a-home-of-their-own): each a fresh directory of its own
//! under the system's temporary directory, outside this repository, so that no git command of a
//! test can reach it, built through the real git CLI.
//! git and `nuc` run under a cleared environment with the allowlist the shell passes, `HOME`
//! an empty directory of the test process's own, a fixed author, committer, and date, the hooks
//! path on the null device, and `core.autocrlf`, background maintenance, and the file-system
//! monitor off in each repository, so that neither the parent's `GIT_*` variables nor the user's
//! global configuration, identity, hooks, or line endings change a fixture, and nothing changes
//! it after a command returns.

// Each test crate includes this module and uses part of it.
#![allow(dead_code)]

pub mod sources;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The variables passed from the parent, as the shell passes them, `HOME` excepted, which is
/// [`home`].
const PASSED: &[&str] = &["PATH", "SystemRoot"];

/// The keys that keep git from working in a fixture's background, whose files would come and go
/// while a test takes a snapshot: maintenance and the file-system monitor.
const QUIET: &[(&str, &str)] = &[
    ("maintenance.auto", "false"),
    ("gc.auto", "0"),
    ("core.fsmonitor", "false"),
];

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

/// The `HOME` of every git and `nuc` a test runs: an empty directory of this test process's own,
/// outside any repository, so that no global configuration reaches a fixture. No hook runs when a
/// test process ends, so each one first removes the `HOME`s that earlier processes left.
pub fn home() -> &'static Path {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let tests = std::env::temp_dir().join("nuc-tests");
        sweep_homes(&tests, LEFT_BEHIND);
        let home = tests.join(format!("home-{}", process::id()));
        let _ = fs::remove_dir_all(&home);
        fs::create_dir_all(&home).expect("the tests' HOME is created");
        home
    })
}

/// How old a `HOME` of another test process is before it counts as left behind: longer than any
/// run lasts, so that a run beside this one keeps its own.
const LEFT_BEHIND: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

/// Removes each `home-*` directory in `dir` that is empty and at least `age` old. One that is
/// not empty is no `HOME` of the tests, which write nothing there, and stays.
pub fn sweep_homes(dir: &Path, age: std::time::Duration) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let old = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .is_ok_and(|modified| modified.elapsed().is_ok_and(|elapsed| elapsed >= age));
        if entry.file_name().to_string_lossy().starts_with("home-") && old {
            let _ = fs::remove_dir(entry.path());
        }
    }
}

/// The variable set in a child that [`rerun_under_a_home`] starts, which then runs the test's body.
pub const HOME_CHILD: &str = "NUC_TESTS_HOME_CHILD";

/// The test `name` of this test binary run again as a child, its parent `HOME` a directory whose
/// `.gitconfig` is `gitconfig` and which also holds the files of `files`; the child's output.
/// The child's helpers pass their own [`home`], so nothing of that configuration may reach them.
pub fn rerun_under_a_home(name: &str, gitconfig: &str, files: &[(&str, &str)]) -> Output {
    let parent = std::env::temp_dir()
        .join("nuc-tests")
        .join(format!("parent-home-{name}-{}", process::id()));
    let _ = fs::remove_dir_all(&parent);
    fs::create_dir_all(&parent).expect("the parent HOME is created");
    fs::write(parent.join(".gitconfig"), gitconfig).expect("the global configuration is written");
    for (file, text) in files {
        fs::write(parent.join(file), text).expect("a file of the parent HOME is written");
    }
    let output = Command::new(std::env::current_exe().expect("the test binary is known"))
        .args(["--exact", name, "--nocapture", "--test-threads=1"])
        .env("HOME", &parent)
        .env(HOME_CHILD, "1")
        .output()
        .expect("the child runs");
    let _ = fs::remove_dir_all(&parent);
    assert!(
        output.status.success(),
        "the child failed:\n{}{}",
        stdout(&output),
        stderr(&output)
    );
    assert!(
        stdout(&output).contains("1 passed"),
        "the child ran no test:\n{}",
        stdout(&output)
    );
    output
}

/// The command with the environment cleared and the allowlist, the tests' `HOME`, and the fixed
/// values set.
pub fn cleared(command: &mut Command) -> &mut Command {
    command.env_clear();
    for key in PASSED {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
        .env("HOME", home())
        .envs(SET.iter().copied())
        .stdin(Stdio::null())
}

/// git for `args` on the repository at `dir`, which must succeed.
pub fn git_in(dir: &Path, args: &[&str]) -> Output {
    let output = cleared(&mut Command::new("git"))
        .arg("-C")
        .arg(dir)
        .args(["-c", &format!("core.hooksPath={NULL_DEVICE}")])
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
        // The bytes a fixture commits are the bytes it checks out, whatever `core.autocrlf` the
        // system's configuration sets, as git for Windows' does.
        fixture.git(&["config", "core.autocrlf", "false"]);
        // In its own configuration, so that the git a test runs through its hooks is quiet too.
        for (key, value) in QUIET {
            fixture.git(&["config", key, value]);
        }
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
                let contents = fs::read(&path)
                    .unwrap_or_else(|e| panic!("{} is unreadable: {e}", path.display()));
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

/// The identifier `ADR-<key>`, assembled as `src/testing.rs` assembles it for the unit tests: the
/// gate's bash checks read the tests as text and would take a literal one for a reference.
pub fn id(key: &str) -> String {
    format!("{}-{key}", "ADR")
}

/// A line of a marker that cites the identifiers, assembled for the same reason.
pub fn mark(ids: &str) -> String {
    format!("{}: {ids}\n", "Verifies")
}
