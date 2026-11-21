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
//! A second stand-in passes each call on to git: every diff of every sensor carries the guards, and
//! no command the repository's configuration names runs.

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

    /// The git on the `PATH`, as the tests find it.
    fn real_git() -> String {
        let out = Command::new("sh")
            .args(["-c", "command -v git"])
            .output()
            .expect("sh runs")
            .stdout;
        String::from_utf8(out).expect("UTF-8").trim().to_owned()
    }

    /// A script that creates the canary wherever it runs, and passes its input or its first
    /// argument through, as a filter, a textconv driver, or a diff command of the repository would.
    fn bait(path: &Path, canary: &Path) -> String {
        let script = format!(
            "#!/bin/sh\ntouch '{}'\nif [ -n \"$1\" ]; then cat \"$1\"; else cat; fi\nexit 0\n",
            canary.display()
        );
        fs::write(path, script).expect("the bait is written");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("it is executable");
        path.to_str().expect("UTF-8").to_owned()
    }

    /// The git command of a call, past `--no-lazy-fetch`, `-C`, and each `-c`, and whether the
    /// call names the repository by `-C`.
    fn command_of(args: &[String]) -> (Option<&str>, bool) {
        let mut repo = false;
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            match arg.as_str() {
                "--no-lazy-fetch" => {}
                "-C" => {
                    repo = true;
                    rest.next();
                }
                "-c" => {
                    rest.next();
                }
                command => return (Some(command), repo),
            }
        }
        (None, repo)
    }

    /// Whether a call shows a diff, which a driver of the repository could produce: a `diff`, a
    /// `diff-tree`, a `show`, or a `log` that shows patches or searches them.
    fn shows_a_diff(command: &str, args: &[String]) -> bool {
        match command {
            "diff" | "diff-tree" | "show" => true,
            "log" => args
                .iter()
                .any(|arg| arg == "-p" || arg == "--patch" || arg.starts_with("-G")),
            _ => false,
        }
    }

    /// Every sensor `nuc sense` ports, as it names them for a name it does not know.
    fn sensors(fixture: &Fixture) -> Vec<String> {
        let output = fixture.nuc(&["sense", "no-such-sensor"]);
        let text = stderr(&output);
        let list = text
            .split_once("it ports ")
            .and_then(|(_, rest)| rest.split_once(" — "))
            .map(|(list, _)| list)
            .unwrap_or_else(|| panic!("no list of sensors: {text}"));
        list.split(", ").map(str::to_owned).collect()
    }

    /// A repository whose configuration names a command for the diffs a sensor could start: a
    /// clean and a process filter, a textconv driver, and a diff command, all on the paths
    /// `*.txt`, which no commit changes, and `diff.submodule`, each the bait that creates the
    /// canary. An external diff, and a driver on a path whose hunks decide, under which a sensor is
    /// not decided, have their tests in `tests/contract.rs`. `feature` changes a file of rules, a
    /// test, a source, a backlog entry, a script an ADR binds by one of its checks, and the text
    /// of an ADR together with a source; a commit follows the one the hand-off names as reviewed.
    /// The working tree changes the file of rules and the script again, holds the hand-off, and
    /// has a path under the filter whose stat alone changed.
    fn baited() -> (Fixture, std::path::PathBuf) {
        let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
        let fixture = Fixture::project("stand-in-sensors", CONFIG, readme);
        fixture
            .write(".gitattributes", "*.txt diff=bait filter=bait\n")
            .write("AGENTS.md", "# Agent Guide\n\n- The human decides.\n")
            .write("tests/a_test.rs", "fn a() {}\n")
            .write("data.txt", "data\n")
            .write(
                "docs/ARCHITECTURE.md",
                "# A\n\n**Last design revision:** 2026-01-01, due after 9 changes.\n",
            )
            .write(
                "docs/adr/FIX-a-rule.md",
                "# FIX-a-rule: X\n\n- **Status:** 🟢 accepted\n- **Applies to:** `src/`\n",
            )
            .write(
                "docs/adr/FIX-b-rule.md",
                "# FIX-b-rule: Y\n\n- **Status:** 🟢 accepted\n- **Applies to:** the rule `a/b` of `scripts/check-a.sh`\n",
            )
            .write("scripts/check-a.sh", "x\n")
            .commit("start");
        fixture.git(&["switch", "-q", "-c", "feature"]);
        fixture
            .write(
                "AGENTS.md",
                "# Agent Guide\n\n- The human decides.\n- The session should read it.\n",
            )
            .write("tests/a_test.rs", "#[ignore]\nfn a() {}\n")
            .write("src/a.rs", "a\n")
            .write("docs/backlog/an-entry.md", "# An entry\n")
            .write("scripts/check-a.sh", "x\ny\n")
            .write(
                "docs/adr/FIX-a-rule.md",
                "# FIX-a-rule: X2\n\n- **Status:** 🟢 accepted\n- **Applies to:** `src/`\n",
            )
            .commit("change");
        let reviewed = super::support::stdout(&fixture.git(&["rev-parse", "--short", "HEAD"]));
        fixture.write("src/a.rs", "a\nb\n").commit("fix");
        fixture.write("h.md", &format!("Reviewed at: {}", reviewed.trim_end()));
        fixture.write(
            "AGENTS.md",
            "# Agent Guide\n\n- The human decides.\n- The session should read it.\n- More.\n",
        );
        fixture.write("scripts/check-a.sh", "x\ny\nz\n");
        // Its stat changed, its bytes not: git reads it again, through its filter where one runs.
        let later = std::time::SystemTime::now() + std::time::Duration::from_secs(60);
        fs::File::options()
            .write(true)
            .open(fixture.root().join("data.txt"))
            .and_then(|file| file.set_modified(later))
            .expect("the mtime is set");
        let canary = fixture.root().join(".git").join("canary");
        let script = bait(&fixture.root().join(".git").join("bait.sh"), &canary);
        for key in [
            "filter.bait.clean",
            "filter.bait.process",
            "diff.bait.textconv",
            "diff.bait.command",
        ] {
            fixture.git(&["config", key, &script]);
        }
        fixture.git(&["config", "diff.submodule", "log"]);
        (fixture, canary)
    }

    /// A `git` in `dir` that appends the arguments of every call beside itself, each call ended by
    /// [`END`], and runs the real git with them; and the `PATH` that finds it first.
    fn recording_stand_in(dir: &Path) -> String {
        let d = dir.display();
        let git = dir.join("git");
        fs::write(
            &git,
            format!(
                "#!/bin/sh\n{{ for arg in \"$@\"; do printf '%s\\n' \"$arg\"; done; printf '%s\\n' '{END}'; }} >> '{d}/args'\nexec '{}' \"$@\"\n",
                real_git()
            ),
        )
        .expect("the stand-in is written");
        fs::set_permissions(&git, fs::Permissions::from_mode(0o755)).expect("it is executable");
        format!("{d}:{}", std::env::var("PATH").unwrap_or_default())
    }

    /// The diff calls among `calls`, each checked: every call that names the repository carries
    /// each key of `git_cli.neutralized`, and every diff the guards.
    fn diffs_checked<'c>(run: &str, calls: &'c [Vec<String>]) -> Vec<&'c Vec<String>> {
        let keys: Vec<&str> = NEUTRALIZED
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();
        let mut diffs = Vec::new();
        for call in calls {
            let (command, repo) = command_of(call);
            if !repo {
                continue;
            }
            for key in &keys {
                assert!(
                    call.windows(2)
                        .any(|pair| pair[0] == "-c" && pair[1] == *key),
                    "{run}: the call lacks -c {key}: {call:?}"
                );
            }
            if !command.is_some_and(|command| shows_a_diff(command, call)) {
                continue;
            }
            diffs.push(call);
            for guard in ["--no-ext-diff", "--no-textconv"] {
                assert!(
                    call.iter().any(|arg| arg == guard),
                    "{run}: the diff lacks {guard}: {call:?}"
                );
            }
        }
        diffs
    }

    /// A sensor, its arguments, and the diffs it must start.
    type Run<'a> = (&'a str, &'a [&'a str], &'a [(&'a str, &'a str)]);

    /// Every diff of every sensor carries the guards, with a head and on the working tree, and no
    /// command the repository's configuration names runs. The stand-in records each call and runs
    /// the real git, so that a guard missing from a call runs the bait, and the canary is created.
    /// Verifies: SEC-repository-commands-neutralized-or-refused
    #[test]
    fn every_diff_of_every_sensor_carries_the_guards() {
        let (fixture, canary) = baited();
        let bin = Fixture::new("stand-in-sensors-bin");
        let path = recording_stand_in(bin.root());
        // Each sensor, and each that reads a change with a head and on the working tree, with the
        // diffs it must start: the command, and an argument that starts as given. Together they
        // are every call site of a diff in `shell::sense`.
        let name_and_hunks: &[(&str, &str)] = &[("diff", "--name-status"), ("diff", "-U0")];
        let runs: &[Run] = &[
            ("action-refs", &[], &[]),
            ("backlog", &[], &[("diff", "--diff-filter=A")]),
            (
                "binding-adrs",
                &["main", "feature"],
                &[("diff", "--name-only"), ("diff", "-U0")],
            ),
            (
                "binding-adrs",
                &["main"],
                &[("diff", "-z"), ("diff", "--name-only"), ("diff", "-U0")],
            ),
            ("plain-rules", &["main", "feature"], name_and_hunks),
            ("plain-rules", &["main"], name_and_hunks),
            ("plan-frontier", &[], &[]),
            (
                "review-binding",
                &["--hand-off", "h.md", "main", "feature"],
                &[
                    ("diff", "--stat"),
                    ("diff-tree", "-p"),
                    ("diff-tree", "--name-only"),
                ],
            ),
            ("revision-due", &[], &[("log", "-G")]),
            ("tests-kept", &["main", "feature"], name_and_hunks),
        ];
        for sensor in sensors(&fixture) {
            assert!(
                runs.iter().any(|(name, _, _)| *name == sensor),
                "the test runs no {sensor}"
            );
        }
        for (sensor, args, expected) in runs {
            let run = format!("{sensor} {args:?}");
            let _ = fs::remove_file(bin.root().join("args"));
            let output = fixture.nuc_with(
                &[("PATH", path.as_str())],
                &[&["sense", sensor][..], args].concat(),
            );
            assert!(
                matches!(output.status.code(), Some(0 | 1)),
                "{run}: {}",
                stderr(&output)
            );
            assert!(!canary.exists(), "{run} ran a command of the repository");
            let calls = calls(bin.root(), "args");
            let diffs = diffs_checked(&run, &calls);
            for (command, start) in *expected {
                assert!(
                    diffs.iter().any(|call| {
                        command_of(call).0 == Some(*command)
                            && call.iter().any(|arg| arg.starts_with(start))
                    }),
                    "{run} started no {command} with {start}: {diffs:?}"
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
