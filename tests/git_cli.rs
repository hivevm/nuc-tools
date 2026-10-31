//! git under a cleared environment. Verifies: ADR-draft-git-cli-cleared-environment Q-9
//!
//! With `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and `GIT_CONFIG_PARAMETERS` pointing at a
//! decoy repository, `nuc` reports on the repository it runs in and leaves the decoy untouched.
//! A git stand-in, first on the `PATH`, records the environment and the arguments `nuc` gives git:
//! nothing outside the allowlist, `-C` on every call, and each key of `git_cli.neutralized`.

mod support;

use std::process::{Command, Output};

use support::{CONFIG, Fixture, NUC, README, snapshot, stderr, stdout};

/// `nuc check` in the fixture, with the parent's environment and the decoy variables set, as an
/// inherited environment would carry them.
fn check_with_decoy(fixture: &Fixture, decoy: &Fixture) -> Output {
    let git_dir = decoy.root().join(".git");
    Command::new(NUC)
        .current_dir(fixture.root())
        .arg("check")
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
    assert_eq!(snapshot(decoy.root()), before, "the decoy changed");
}

#[test]
fn the_repository_run_in_is_the_one_reported() {
    let target = Fixture::project("decoy-reported", CONFIG, "no release line\n");
    let decoy = Fixture::project("decoy-clean", CONFIG, README);
    let before = snapshot(decoy.root());
    let output = check_with_decoy(&target, &decoy);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert!(stdout(&output).starts_with("README.md: docs/template-release: "));
    assert_eq!(snapshot(decoy.root()), before, "the decoy changed");
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

    /// A `git` in `dir` that appends its environment and arguments beside itself, each call ended
    /// by [`END`], and answers `rev-parse --show-toplevel` with `root`.
    fn stand_in(dir: &Path, root: &Path) {
        let d = dir.display();
        let script = format!(
            "#!/bin/sh\n{{ /usr/bin/env; printf '%s\\n' '{END}'; }} >> '{d}/env'\n{{ for arg in \"$@\"; do printf '%s\\n' \"$arg\"; done; printf '%s\\n' '{END}'; }} >> '{d}/args'\nprintf '%s\\n' '{}'\n",
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
                args.first().map(String::as_str),
                Some("-C"),
                "no -C: {args:?}"
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
}
