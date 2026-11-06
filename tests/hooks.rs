//! `nuc hook` and the reference shims, through git. Verifies: ADR-0020 ADR-0021 ADR-0023 Q-1 Q-7 Q-10 Q-11
//!
//! Each case builds a fixture repository whose hooks path is the repository's `shims/`, puts a
//! `nuc` on the `PATH` of the git command, the one cargo built or a stand-in that answers as told,
//! and runs a real `git commit`. The shim refuses every answer but 0 and 3, a missing `nuc`
//! included, runs the full check only on 3, and refuses a red or unrunnable check; `nuc hook
//! pre-commit` refuses a commit on `main` at its ref, and runs no script itself. Without
//! `nuc.toml` every `hook/` rule is on; with one, its list governs and it is read fail closed. A
//! canary token in the environment and the git configuration shows in no output and no file, and
//! on Linux the network blocked changes nothing.

mod support;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use support::{Fixture, NUC, cleared, snapshot, stderr, stdout};

/// The reference shims of this repository.
fn shims() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("shims")
}

/// The `PATH` of the parent without any directory that holds a `nuc`, so that only the one a case
/// puts first is found, or none.
fn path_without_nuc() -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|dir| !dir.join("nuc").exists() && !dir.join("nuc.exe").exists())
        .collect()
}

/// A `PATH` with `first` before the parent's directories that hold no `nuc`.
fn path_with(first: Option<&Path>) -> std::ffi::OsString {
    let dirs = first
        .map(Path::to_path_buf)
        .into_iter()
        .chain(path_without_nuc());
    std::env::join_paths(dirs).expect("the PATH joins")
}

/// The directory of the `nuc` cargo built.
fn built() -> PathBuf {
    Path::new(NUC)
        .parent()
        .expect("nuc has a directory")
        .to_path_buf()
}

/// A directory holding a stand-in `nuc` that runs the shell code, beside the fixture.
fn stand_in(fixture: &Fixture, code: &str) -> PathBuf {
    let dir = fixture.root().with_extension("bin");
    std::fs::create_dir_all(&dir).expect("the stand-in's directory");
    let path = dir.join("nuc");
    std::fs::write(&path, format!("#!/bin/sh\n{code}\n")).expect("the stand-in is written");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("made executable");
    }
    dir
}

/// A fixture with one commit on `main` and the shims as its hooks.
fn shimmed(name: &str) -> Fixture {
    let fixture = Fixture::new(name);
    fixture.git(&["commit", "-q", "--allow-empty", "-m", "init"]);
    fixture.git(&[
        "config",
        "core.hooksPath",
        &shims().display().to_string().replace('\\', "/"),
    ]);
    fixture
}

/// Give the fixture a `scripts/check-all.sh` that prints its arguments and exits as told.
fn checks(fixture: &Fixture, code: u8) {
    fixture.write(
        "scripts/check-all.sh",
        &format!(
            "#!/usr/bin/env bash\necho \"check output for the test, args: $*\"\nexit {code}\n"
        ),
    );
}

/// A merge into `feat/x` stopped on a conflict, resolved and staged, as the template's self-test
/// builds it.
fn conflicted(fixture: &Fixture) {
    fixture.git(&["switch", "-q", "-c", "feature"]);
    fixture.write("f", "feature\n");
    fixture.git(&["add", "f"]);
    fixture.git(&["commit", "-q", "-m", "feature"]);
    fixture.git(&["switch", "-q", "-c", "feat/x", "main"]);
    fixture.write("f", "other\n");
    fixture.git(&["add", "f"]);
    fixture.git(&["commit", "-q", "-m", "other"]);
    let merge = cleared(&mut Command::new("git"))
        .arg("-C")
        .arg(fixture.root())
        .arg("-c")
        .arg(format!("core.hooksPath={}", support::NULL_DEVICE))
        .args(["merge", "-q", "-m", "m", "feature"])
        .output()
        .expect("git runs");
    assert!(
        !merge.status.success(),
        "the merge did not stop on a conflict"
    );
    fixture.write("f", "resolved\n");
    fixture.git(&["add", "f"]);
}

/// git with `args` in the fixture, its hooks run, with `nuc` looked up in `first` before the rest
/// of the `PATH`, and the variables given.
fn git_hooked(
    fixture: &Fixture,
    first: Option<&Path>,
    vars: &[(&str, &str)],
    args: &[&str],
) -> Output {
    cleared(&mut Command::new("git"))
        .env("PATH", path_with(first))
        .envs(vars.iter().copied())
        .current_dir(fixture.root())
        .args(args)
        .output()
        .expect("git runs")
}

fn commit(fixture: &Fixture, first: Option<&Path>) -> Output {
    git_hooked(
        fixture,
        first,
        &[],
        &["commit", "-q", "--allow-empty", "-m", "x"],
    )
}

fn both(output: &Output) -> String {
    format!("{}{}", stdout(output), stderr(output))
}

#[test]
fn a_commit_on_main_is_refused_at_its_ref_and_one_on_a_branch_passes() {
    let fixture = shimmed("hooks-main");
    let output = commit(&fixture, Some(&built()));
    assert!(!output.status.success(), "a commit on main passed");
    assert!(
        both(&output).contains("refs/heads/main: hook/commit-on-main: the checkout is on 'main'"),
        "{}",
        both(&output)
    );
    assert!(both(&output).contains("git switch -c <name>"));
    fixture.git(&["switch", "-q", "-c", "feature"]);
    let output = commit(&fixture, Some(&built()));
    assert!(output.status.success(), "{}", both(&output));
}

#[test]
fn a_concluded_merge_runs_the_check_with_changed_and_refuses_it_red() {
    for (code, passes) in [(0, true), (1, false)] {
        let fixture = shimmed(&format!("hooks-merge-{code}"));
        checks(&fixture, code);
        conflicted(&fixture);
        let output = git_hooked(
            &fixture,
            Some(&built()),
            &[],
            &["commit", "-q", "--no-edit"],
        );
        assert_eq!(output.status.success(), passes, "{}", both(&output));
        assert!(
            both(&output).contains("check output for the test, args: --changed"),
            "{}",
            both(&output)
        );
        if !passes {
            assert!(
                both(&output)
                    .contains("pre-commit: refused — the checks are red on the resolved merge"),
                "{}",
                both(&output)
            );
        }
    }
}

#[test]
fn a_git_dir_handed_from_elsewhere_is_not_decided_and_a_linked_worktree_is() {
    // A repository on main whose git directory lies apart from its work tree, and the work tree
    // inside another repository on a branch, whose branch a hook must not decide on.
    let outer = Fixture::new("hooks-outer");
    outer.git(&["commit", "-q", "--allow-empty", "-m", "init"]);
    outer.git(&["switch", "-q", "-c", "feature"]);
    let inner = shimmed("hooks-separate");
    let work_tree = outer.root().join("wt");
    std::fs::create_dir_all(&work_tree).expect("the work tree");
    let git_dir = inner.root().join(".git").display().to_string();
    let output = git_hooked(
        &outer,
        Some(&built()),
        &[],
        &[
            "--git-dir",
            &git_dir,
            "--work-tree",
            &work_tree.display().to_string(),
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "x",
        ],
    );
    assert!(
        !output.status.success(),
        "a commit on main passed through another git directory"
    );
    assert!(both(&output).contains("not decided"), "{}", both(&output));

    let fixture = shimmed("hooks-worktree");
    let linked = fixture.root().with_extension("linked");
    fixture.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "feature",
        &linked.display().to_string(),
    ]);
    let output = cleared(&mut Command::new("git"))
        .env("PATH", path_with(Some(&built())))
        .current_dir(&linked)
        .args(["commit", "-q", "--allow-empty", "-m", "x"])
        .output()
        .expect("git runs");
    let _ = std::fs::remove_dir_all(&linked);
    assert!(output.status.success(), "{}", both(&output));
}

#[test]
fn the_shim_refuses_every_answer_but_0_and_3_and_a_missing_nuc() {
    for code in [1, 2, 101, 7] {
        let fixture = shimmed(&format!("hooks-answer-{code}"));
        fixture.git(&["switch", "-q", "-c", "feature"]);
        let bin = stand_in(&fixture, &format!("exit {code}"));
        let output = commit(&fixture, Some(&bin));
        assert!(!output.status.success(), "the shim passed exit {code}");
        let _ = std::fs::remove_dir_all(bin);
    }
    let fixture = shimmed("hooks-answer-0");
    fixture.git(&["switch", "-q", "-c", "feature"]);
    let bin = stand_in(&fixture, "exit 0");
    let output = commit(&fixture, Some(&bin));
    assert!(output.status.success(), "{}", both(&output));
    let _ = std::fs::remove_dir_all(bin);

    let output = commit(&fixture, None);
    assert!(!output.status.success(), "the shim passed without nuc");
    assert!(
        both(&output).contains("Install it from https://github.com/hivevm/nuc-tools/releases"),
        "{}",
        both(&output)
    );
}

#[test]
fn the_shim_runs_the_check_only_on_3_and_refuses_one_that_cannot_run() {
    let fixture = shimmed("hooks-run");
    fixture.git(&["switch", "-q", "-c", "feature"]);
    checks(&fixture, 0);
    let bin = stand_in(&fixture, "exit 0");
    let output = commit(&fixture, Some(&bin));
    assert!(output.status.success(), "{}", both(&output));
    assert!(
        !both(&output).contains("check output"),
        "the check ran on 0"
    );

    let bin = stand_in(&fixture, "exit 3");
    let output = commit(&fixture, Some(&bin));
    assert!(output.status.success(), "{}", both(&output));
    assert!(both(&output).contains("check output for the test, args: --changed"));

    std::fs::remove_file(fixture.root().join("scripts/check-all.sh")).expect("removed");
    let output = commit(&fixture, Some(&bin));
    assert!(!output.status.success(), "a check that cannot run passed");
    let _ = std::fs::remove_dir_all(bin);
}

#[test]
fn nuc_hook_pre_commit_runs_no_script_of_the_repository() {
    let fixture = shimmed("hooks-canary");
    let canary = fixture.root().join("canary-ran");
    fixture.write(
        "scripts/check-all.sh",
        &format!(
            "#!/bin/sh\necho ran >> \"{}\"\n",
            canary.display().to_string().replace('\\', "/")
        ),
    );
    conflicted(&fixture);
    let output = fixture.nuc(&["hook", "pre-commit"]);
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    assert!(!canary.exists(), "nuc hook pre-commit ran the check itself");
}

/// `nuc hook pre-commit` in the fixture, called directly.
fn decide(fixture: &Fixture) -> Output {
    fixture.nuc(&["hook", "pre-commit"])
}

#[test]
fn without_nuc_toml_every_hook_rule_is_on() {
    let fixture = shimmed("hooks-defaults");
    let output = decide(&fixture);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(
        stdout(&output)
            .lines()
            .next()
            .and_then(|line| line.split(": ").nth(1)),
        Some("hook/commit-on-main")
    );
    assert!(
        stdout(&output).starts_with("refs/heads/main: "),
        "{}",
        stdout(&output)
    );
    fixture.git(&["switch", "-q", "-c", "feature"]);
    let output = decide(&fixture);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), "");
}

#[test]
fn with_nuc_toml_its_list_governs_the_hook_rules() {
    let fixture = shimmed("hooks-listed-none");
    fixture.write(
        "nuc.toml",
        "minimum = \"0.0.0\"\nrules = [\"docs/template-release\"]\n",
    );
    assert_eq!(decide(&fixture).status.code(), Some(0), "on main");
    checks(&fixture, 1);
    conflicted(&fixture);
    let output = decide(&fixture);
    assert_eq!(
        output.status.code(),
        Some(0),
        "a concluded merge: {}",
        stderr(&output)
    );

    let fixture = shimmed("hooks-listed");
    fixture.write(
        "nuc.toml",
        "minimum = \"0.0.0\"\nrules = [\"docs/template-release\", \"hook/commit-on-main\"]\n",
    );
    assert_eq!(decide(&fixture).status.code(), Some(1), "on main");
}

#[test]
fn a_nuc_toml_that_cannot_be_held_is_not_decided() {
    for (name, config, needle) in [
        (
            "hooks-unreadable",
            "minimum = \"0.0.0\"\n[rules]\n",
            "unreadable",
        ),
        (
            "hooks-too-old",
            "minimum = \"99.0.0\"\nrules = []\n",
            "requires nuc 99.0.0",
        ),
        (
            "hooks-unknown",
            "minimum = \"0.0.0\"\nrules = [\"hook/no-such-rule\"]\n",
            "does not know",
        ),
    ] {
        let fixture = shimmed(name);
        fixture.git(&["switch", "-q", "-c", "feature"]);
        fixture.write("nuc.toml", config);
        let output = decide(&fixture);
        assert_eq!(output.status.code(), Some(2), "{name}");
        assert!(
            stderr(&output).contains(needle),
            "{name}: {}",
            stderr(&output)
        );
        let output = commit(&fixture, Some(&built()));
        assert!(!output.status.success(), "{name}: the shim passed exit 2");
    }
}

#[test]
fn nuc_check_knows_the_hook_rules_and_runs_none() {
    let fixture = Fixture::project(
        "hooks-check",
        "minimum = \"0.0.0\"\nrules = [\"docs/template-release\", \"hook/commit-on-main\", \"hook/merge-checks\"]\n",
        support::README,
    );
    let output = fixture.nuc(&["check"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(stderr(&output).contains("(1 rules)"), "{}", stderr(&output));
    for id in ["hook/commit-on-main", "hook/merge-checks"] {
        let output = fixture.nuc(&["explain", id]);
        assert_eq!(output.status.code(), Some(0), "{id}");
        assert!(
            stdout(&output).starts_with(&format!("{id}: ")),
            "{}",
            stdout(&output)
        );
    }
}

/// The token planted in the environment and the git configuration.
const TOKEN: &str = "nuc-canary-token-5f3a9c";

/// The output of one run and the files it wrote, by path and contents.
type Run = (Output, Vec<(PathBuf, Vec<u8>)>);

/// A case: its name, its setup, and the git command.
type Case<'a> = (&'a str, &'a dyn Fn(&Fixture), &'a [&'a str]);

/// The pre-commit cases through the shim and the built `nuc`, each in a fixture of its own with
/// the token in the git configuration, run by `run`: the outputs, and the files each run changed.
fn pre_commit_cases(name: &str, run: &dyn Fn(&Fixture, &[&str]) -> Output) -> Vec<Run> {
    let cases: [Case; 3] = [
        (
            "main",
            &|_| {},
            &["commit", "-q", "--allow-empty", "-m", "x"],
        ),
        (
            "branch",
            &|f| {
                f.git(&["switch", "-q", "-c", "feature"]);
            },
            &["commit", "-q", "--allow-empty", "-m", "x"],
        ),
        (
            "merge",
            &|f| {
                checks(f, 1);
                conflicted(f);
            },
            &["commit", "-q", "--no-edit"],
        ),
    ];
    cases
        .iter()
        .map(|(case, setup, args)| {
            let fixture = shimmed(&format!("{name}-{case}"));
            setup(&fixture);
            fixture.git(&[
                "config",
                "http.extraHeader",
                &format!("Authorization: Bearer {TOKEN}"),
            ]);
            let before = snapshot(fixture.root());
            let output = run(&fixture, args);
            let after = snapshot(fixture.root());
            let written = after
                .into_iter()
                .filter(|entry| !before.contains(entry))
                .collect();
            (output, written)
        })
        .collect()
}

/// The token's variables, as CI and a developer's shell carry them.
const TOKEN_VARS: [(&str, &str); 3] = [
    ("GITHUB_TOKEN", TOKEN),
    ("GH_TOKEN", TOKEN),
    ("NUC_TOKEN", TOKEN),
];

#[test]
fn a_canary_token_shows_in_no_output_and_no_file() {
    let runs = pre_commit_cases("hooks-token", &|fixture, args| {
        git_hooked(fixture, Some(&built()), &TOKEN_VARS, args)
    });
    for (output, written) in runs {
        assert!(!both(&output).contains(TOKEN), "{}", both(&output));
        for (path, contents) in written {
            assert!(
                !String::from_utf8_lossy(&contents).contains(TOKEN),
                "{} holds the token",
                path.display()
            );
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn the_network_blocked_changes_no_pre_commit_case() {
    let offline = |fixture: &Fixture, args: &[&str]| {
        cleared(&mut Command::new("unshare"))
            .args(["--net", "--map-current-user", "--", "git"])
            .args(args)
            .env("PATH", path_with(Some(&built())))
            .envs(TOKEN_VARS)
            .current_dir(fixture.root())
            .output()
            .expect("unshare runs")
    };
    let online = pre_commit_cases("hooks-online", &|fixture, args| {
        git_hooked(fixture, Some(&built()), &TOKEN_VARS, args)
    });
    let blocked = pre_commit_cases("hooks-offline", &offline);
    for ((on, _), (off, _)) in online.iter().zip(&blocked) {
        assert_eq!(off.status.code(), on.status.code(), "{}", both(off));
        assert_eq!(both(off), both(on));
        assert!(!both(off).contains(TOKEN));
    }
}
