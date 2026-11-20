//! git under a cleared environment. Q-9
//! Verifies: SEC-git-cleared-without-lazy-fetch
//!
//! `nuc hook` decides on the repository it runs in as well, and does not decide where git hands it
//! another git directory.
//!
//! With `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and `GIT_CONFIG_PARAMETERS` pointing at a
//! decoy repository, `nuc` reports on the repository it runs in and leaves the decoy untouched.
//! A git stand-in, first on the `PATH`, records the environment and the arguments `nuc` gives git:
//! nothing outside the allowlist, `-C` on every call, and each key of `git_cli.neutralized`.

mod support;

use std::process::{Command, Output};

use support::{CONFIG, Fixture, NUC, README, snapshot, stderr, stdout};

/// `nuc check` in the fixture, with the parent's environment and the decoy variables set, as an
/// inherited environment would carry them, and the tests' `HOME` over the parent's
/// (TEST-fixtures-with-own-home).
fn check_with_decoy(fixture: &Fixture, decoy: &Fixture) -> Output {
    let git_dir = decoy.root().join(".git");
    Command::new(NUC)
        .current_dir(fixture.root())
        .arg("check")
        .env("HOME", support::home())
        .env("GIT_DIR", &git_dir)
        .env("GIT_WORK_TREE", decoy.root())
        .env("GIT_INDEX_FILE", git_dir.join("index"))
        .env(
            "GIT_CONFIG_PARAMETERS",
            "'core.hooksPath'='/nonexistent-decoy-hooks'",
        )
        .output()
        .expect("nuc runs")
}

#[test]
fn a_decoy_in_the_environment_is_neither_read_nor_touched() {
    let target = Fixture::project("decoy-target", CONFIG, README);
    let decoy = Fixture::project("decoy", CONFIG, "no release line\n");
    let before = snapshot(decoy.root());
    let output = check_with_decoy(&target, &decoy);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let after = snapshot(decoy.root());
    assert!(
        after == before,
        "the decoy changed: {:?}",
        support::changed(&before, &after)
    );
}

#[test]
fn the_repository_run_in_is_the_one_reported() {
    let target = Fixture::project("decoy-reported", CONFIG, "no release line\n");
    let decoy = Fixture::project("decoy-clean", CONFIG, README);
    let before = snapshot(decoy.root());
    let output = check_with_decoy(&target, &decoy);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert!(stdout(&output).starts_with("README.md: docs/template-release: "));
    let after = snapshot(decoy.root());
    assert!(
        after == before,
        "the decoy changed: {:?}",
        support::changed(&before, &after)
    );
}

/// `nuc hook pre-commit` in the fixture, with the decoy variables set as given and the tests'
/// `HOME` over the parent's.
fn hook_with(fixture: &Fixture, vars: &[(&str, std::ffi::OsString)]) -> Output {
    let mut command = Command::new(NUC);
    command
        .current_dir(fixture.root())
        .args(["hook", "pre-commit"])
        .env("HOME", support::home());
    for (key, value) in vars {
        command.env(key, value);
    }
    command.output().expect("nuc runs")
}

#[test]
fn nuc_hook_decides_on_the_repository_run_in_or_not_at_all() {
    // No nuc.toml, so every hook rule is on.
    let target = Fixture::new("decoy-hook");
    target.git(&["commit", "-q", "--allow-empty", "-m", "init"]);
    let decoy = Fixture::project("decoy-hook-decoy", CONFIG, README);
    decoy.git(&["switch", "-q", "-c", "feature"]);
    let git_dir = decoy.root().join(".git");
    let before = snapshot(decoy.root());
    // The index and the configuration of another repository: the hook still decides on its own.
    let output = hook_with(
        &target,
        &[
            ("GIT_INDEX_FILE", git_dir.join("index").into_os_string()),
            (
                "GIT_CONFIG_PARAMETERS",
                "'core.hooksPath'='/nonexistent-decoy-hooks'".into(),
            ),
        ],
    );
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert!(stdout(&output).starts_with("refs/heads/main: hook/commit-on-main: "));
    // Another git directory: the decoy's branch would pass, so the hook does not decide.
    let output = hook_with(
        &target,
        &[
            ("GIT_DIR", git_dir.clone().into_os_string()),
            ("GIT_WORK_TREE", decoy.root().as_os_str().to_owned()),
        ],
    );
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("not decided"),
        "{}",
        stderr(&output)
    );
    let after = snapshot(decoy.root());
    assert!(
        after == before,
        "the decoy changed: {:?}",
        support::changed(&before, &after)
    );
}

#[cfg(unix)]
mod stand_in {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::process::Command;

    use super::support::{CONFIG, Fixture, NUC, README, stderr};

    const NEUTRALIZED: &str = include_str!("git_cli.neutralized");

    /// What `nuc` passes; and what `/bin/sh` adds to the environment of the commands it starts.
    const ALLOWED: &[&str] = &[
        "PATH",
        "HOME",
        "LC_ALL",
        "GIT_TERMINAL_PROMPT",
        "GIT_OPTIONAL_LOCKS",
    ];
    const SHELL_ADDED: &[&str] = &["PWD", "OLDPWD", "SHLVL", "_"];

    /// The separator the stand-in writes after each call, a line no environment or argument has.
    const END: &str = "--- end of call ---";

    /// A `git` in `dir` that answers `--version` as git 2.45.0 does, appends the environment and
    /// arguments of every other call beside itself, each call ended by [`END`], and answers
    /// `rev-parse --show-toplevel` with `root`.
    fn stand_in(dir: &Path, root: &Path) {
        let d = dir.display();
        let script = format!(
            "#!/bin/sh\n[ \"$1\" = --version ] && {{ echo 'git version 2.45.0'; exit 0; }}\n{{ /usr/bin/env; printf '%s\\n' '{END}'; }} >> '{d}/env'\n{{ for arg in \"$@\"; do printf '%s\\n' \"$arg\"; done; printf '%s\\n' '{END}'; }} >> '{d}/args'\nprintf '%s\\n' '{}'\n",
            root.display()
        );
        let git = dir.join("git");
        fs::write(&git, script).expect("the stand-in is written");
        fs::set_permissions(&git, fs::Permissions::from_mode(0o755)).expect("it is executable");
    }

    /// What the stand-in wrote to `file`, one list of lines per call.
    fn calls(dir: &Path, file: &str) -> Vec<Vec<String>> {
        let text = fs::read_to_string(dir.join(file)).expect("the stand-in ran");
        let mut calls = vec![Vec::new()];
        for line in text.lines() {
            if line == END {
                calls.push(Vec::new());
            } else {
                calls
                    .last_mut()
                    .expect("one call is open")
                    .push(line.to_owned());
            }
        }
        calls.pop();
        calls
    }

    #[test]
    fn git_gets_the_allowlist_minus_c_and_every_neutralized_key() {
        let fixture = Fixture::project("stand-in", CONFIG, README);
        let bin = Fixture::new("stand-in-bin");
        stand_in(bin.root(), fixture.root());
        let path = format!(
            "{}:{}",
            bin.root().display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let output = Command::new(NUC)
            .current_dir(fixture.root())
            .arg("check")
            .env("HOME", super::support::home())
            .env("PATH", path)
            .env("GIT_DIR", "/decoy")
            .env("CANARY_TOKEN", "do-not-pass")
            .output()
            .expect("nuc runs");
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

        let envs = calls(bin.root(), "env");
        let argses = calls(bin.root(), "args");
        assert!(!envs.is_empty(), "nuc made no git call");
        assert_eq!(envs.len(), argses.len());
        let keys: Vec<&str> = NEUTRALIZED
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();
        assert!(!keys.is_empty());
        for (env, args) in envs.iter().zip(&argses) {
            for line in env {
                let name = line.split('=').next().unwrap_or_default();
                assert!(
                    ALLOWED.contains(&name) || SHELL_ADDED.contains(&name),
                    "git got '{name}', outside the allowlist, in the call {args:?}"
                );
            }
            for set in ["LC_ALL=C", "GIT_TERMINAL_PROMPT=0", "GIT_OPTIONAL_LOCKS=0"] {
                assert!(
                    env.iter().any(|line| line == set),
                    "git did not get {set}: {args:?}"
                );
            }
            assert_eq!(
                args.get(..2).map(|first| first.join(" ")).as_deref(),
                Some("--no-lazy-fetch -C"),
                "the call does not open with --no-lazy-fetch and -C: {args:?}"
            );
            for key in &keys {
                assert!(
                    args.windows(2)
                        .any(|pair| pair[0] == "-c" && pair[1] == *key),
                    "the call lacks -c {key}: {args:?}"
                );
            }
        }
    }

    /// A git before 2.45.0, which refuses `--no-lazy-fetch` as an unknown option, gets no such
    /// option: it runs on a repository that names no promisor remote, and on one that does, the
    /// run is not decided and names the version `nuc` needs for it.
    /// Verifies: SEC-git-cleared-without-lazy-fetch
    #[test]
    fn a_git_without_no_lazy_fetch_runs_only_where_no_object_is_fetched_lazily() {
        let fixture = Fixture::project("old-git", CONFIG, README);
        let bin = Fixture::new("old-git-bin");
        let real = String::from_utf8(
            Command::new("sh")
                .args(["-c", "command -v git"])
                .output()
                .expect("sh runs")
                .stdout,
        )
        .expect("UTF-8");
        let git = bin.root().join("git");
        fs::write(
            &git,
            format!(
                "#!/bin/sh\n[ \"$1\" = --version ] && {{ echo 'git version 2.43.0'; exit 0; }}\nfor arg in \"$@\"; do [ \"$arg\" = --no-lazy-fetch ] && {{ echo 'unknown option: --no-lazy-fetch' >&2; exit 129; }}; done\nexec '{}' \"$@\"\n",
                real.trim()
            ),
        )
        .expect("the stand-in is written");
        fs::set_permissions(&git, fs::Permissions::from_mode(0o755)).expect("it is executable");
        let path = format!(
            "{}:{}",
            bin.root().display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let run = |args: &[&str]| {
            Command::new(NUC)
                .current_dir(fixture.root())
                .args(args)
                .env("HOME", super::support::home())
                .env("PATH", &path)
                .output()
                .expect("nuc runs")
        };
        let output = run(&["check"]);
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
        fixture.git(&["config", "remote.origin.promisor", "true"]);
        for args in [&["check"][..], &["sense", "revision-due"]] {
            let output = run(args);
            assert_eq!(output.status.code(), Some(2), "{args:?}");
            assert!(
                stderr(&output).contains("a partial clone")
                    && stderr(&output).contains("nuc needs git 2.45.0 or later"),
                "{args:?}: {}",
                stderr(&output)
            );
        }
    }
}
