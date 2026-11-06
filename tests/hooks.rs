//! `nuc hook` and the reference shims, through git. Verifies: ADR-0020 ADR-0021 ADR-0023 G-5 Q-1 Q-4 Q-7 Q-10 Q-11
//!
//! Each case builds a fixture repository whose hooks path is the repository's `shims/`, puts a
//! `nuc` on the `PATH` of the git command, the one cargo built or a stand-in that answers as told,
//! and runs a real `git commit` or `git merge`. The shim refuses every answer but 0 and 3, a
//! missing `nuc` included, runs the hook's check only on 3, and refuses a red or unrunnable check;
//! `nuc hook` refuses a commit and a merge on `main` at its ref, and runs no script itself. Every
//! shim stops without `nuc`, with a `nuc` older than `nuc.toml` requires, on a panic, and where git
//! names no work tree, and decides with no bash on the `PATH` (G-5, Q-4). Without
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
fn every_shim_refuses_every_answer_but_0_and_3() {
    for hook in every_shim() {
        for code in [1, 2, 101, 7, 4] {
            let fixture = shimmed(&format!("hooks-answer-{hook}-{code}"));
            let args = trigger(&fixture, &hook);
            let bin = stand_in(&fixture, &format!("exit {code}"));
            let output = git_hooked(&fixture, Some(&bin), &[], args);
            assert!(!output.status.success(), "{hook} passed exit {code}");
            let _ = std::fs::remove_dir_all(bin);
        }
        let fixture = shimmed(&format!("hooks-answer-{hook}-0"));
        let args = trigger(&fixture, &hook);
        let bin = stand_in(&fixture, "exit 0");
        let output = git_hooked(&fixture, Some(&bin), &[], args);
        assert!(output.status.success(), "{hook}: {}", both(&output));
        let _ = std::fs::remove_dir_all(bin);
    }
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

    let fixture = shimmed("hooks-merge-listed-none");
    fixture.write(
        "nuc.toml",
        "minimum = \"0.0.0\"\nrules = [\"hook/merge-on-main\"]\n",
    );
    docs(&fixture, 1);
    merging(&fixture);
    let output = git_hooked(&fixture, Some(&built()), &[], MERGE);
    assert!(
        output.status.success(),
        "the documentation check ran: {}",
        both(&output)
    );
    assert!(!both(&output).contains("docs output"), "{}", both(&output));

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

/// Give the fixture a `scripts/check-docs.sh` that prints a line and exits as told.
fn docs(fixture: &Fixture, code: u8) {
    fixture.write(
        "scripts/check-docs.sh",
        &format!("#!/usr/bin/env bash\necho \"docs output for the test\"\nexit {code}\n"),
    );
}

/// A branch `feature` one commit ahead of `main`, and `feat/x` at `main` checked out.
fn merging(fixture: &Fixture) {
    fixture.git(&["switch", "-q", "-c", "feature"]);
    fixture.write("f", "feature\n");
    fixture.git(&["add", "f"]);
    fixture.git(&["commit", "-q", "-m", "feature"]);
    fixture.git(&["switch", "-q", "-c", "feat/x", "main"]);
}

const MERGE: &[&str] = &["merge", "-q", "--no-ff", "-m", "m", "feature"];

#[test]
fn a_merge_on_main_is_refused_and_one_elsewhere_passes_the_documentation_check() {
    let fixture = shimmed("hooks-merge-main");
    docs(&fixture, 0);
    merging(&fixture);
    fixture.git(&["switch", "-q", "main"]);
    let output = git_hooked(&fixture, Some(&built()), &[], MERGE);
    assert!(!output.status.success(), "a merge on main passed");
    assert!(
        both(&output).contains("refs/heads/main: hook/merge-on-main: the checkout is on 'main'"),
        "{}",
        both(&output)
    );

    for (code, passes) in [(0, true), (1, false)] {
        let fixture = shimmed(&format!("hooks-merge-docs-{code}"));
        docs(&fixture, code);
        merging(&fixture);
        let output = git_hooked(&fixture, Some(&built()), &[], MERGE);
        assert_eq!(output.status.success(), passes, "{}", both(&output));
        assert!(
            both(&output).contains("docs output for the test"),
            "{}",
            both(&output)
        );
        if !passes {
            assert!(
                both(&output)
                    .contains("pre-merge-commit: refused — the documentation check is red"),
                "{}",
                both(&output)
            );
        }
    }

    let fixture = shimmed("hooks-merge-no-docs");
    merging(&fixture);
    let output = git_hooked(&fixture, Some(&built()), &[], MERGE);
    assert!(output.status.success(), "{}", both(&output));
}

/// The fixture set up for the shim of `hook` to run, on a branch, and the git command that runs
/// it. A shim this does not know fails the test, so that each new shim is held to fail closed.
fn trigger(fixture: &Fixture, hook: &str) -> &'static [&'static str] {
    match hook {
        "pre-commit" => {
            fixture.git(&["switch", "-q", "-c", "feature"]);
            &["commit", "-q", "--allow-empty", "-m", "x"]
        }
        "pre-merge-commit" => {
            merging(fixture);
            MERGE
        }
        other => panic!("no git command runs the shim '{other}': add it here"),
    }
}

/// Every reference shim by its name.
fn every_shim() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(shims())
        .expect("shims/ is readable")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

#[test]
fn every_shim_stops_without_nuc_and_with_a_nuc_too_old() {
    for hook in every_shim() {
        let fixture = shimmed(&format!("hooks-absent-{hook}"));
        let args = trigger(&fixture, &hook);
        let output = git_hooked(&fixture, None, &[], args);
        assert!(!output.status.success(), "{hook} passed without nuc");
        assert!(
            both(&output).contains(&format!(
                "{hook}: refused — nuc is not on the PATH, so no hook decides. Install it from https://github.com/hivevm/nuc-tools/releases"
            )),
            "{hook}: {}",
            both(&output)
        );

        let fixture = shimmed(&format!("hooks-old-{hook}"));
        let canary = fixture.root().join("canary-ran");
        let script = format!(
            "#!/bin/sh\necho ran >> \"{}\"\n",
            canary.display().to_string().replace('\\', "/")
        );
        fixture
            .write("scripts/check-all.sh", &script)
            .write("scripts/check-docs.sh", &script)
            .write(
                "nuc.toml",
                "minimum = \"99.0.0\"\nrules = [\"hook/merge-checks\", \"hook/merge-docs\"]\n",
            );
        let args = trigger(&fixture, &hook);
        let output = git_hooked(&fixture, Some(&built()), &[], args);
        assert!(!output.status.success(), "{hook} passed with a nuc too old");
        assert!(
            both(&output).contains("requires nuc 99.0.0 or later")
                && both(&output).contains("install or update nuc"),
            "{hook}: {}",
            both(&output)
        );
        assert!(!canary.exists(), "{hook} ran a check before it stopped");
    }
}

/// A directory that holds git and no bash: git's exec path where it does, else, on Unix, one with
/// a link to the git on the `PATH`.
fn git_alone(beside: &Fixture) -> PathBuf {
    let exec = cleared(&mut Command::new("git"))
        .arg("--exec-path")
        .output()
        .expect("git runs");
    let exec = PathBuf::from(stdout(&exec).trim_end());
    let has = |dir: &Path, name: &str| {
        dir.join(name).exists() || dir.join(format!("{name}.exe")).exists()
    };
    if has(&exec, "git") && !has(&exec, "bash") {
        return exec;
    }
    #[cfg(unix)]
    {
        let dir = beside.root().with_extension("git");
        std::fs::create_dir_all(&dir).expect("the directory of git");
        let git = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .map(|dir| dir.join("git"))
            .find(|git| git.is_file())
            .expect("git is on the PATH");
        let _ = std::fs::remove_file(dir.join("git"));
        std::os::unix::fs::symlink(git, dir.join("git")).expect("git linked");
        dir
    }
    #[cfg(not(unix))]
    {
        let _ = beside;
        panic!("git's exec path {} holds no git, or a bash", exec.display());
    }
}

/// git with `args` in the fixture, its hooks run, with a `PATH` of `first` and git alone.
fn git_without_bash(fixture: &Fixture, first: &Path, args: &[&str]) -> Output {
    let dirs = [first.to_path_buf(), git_alone(fixture)];
    for dir in &dirs {
        for bash in ["bash", "bash.exe"] {
            assert!(!dir.join(bash).exists(), "{} holds a bash", dir.display());
        }
    }
    let git = dirs[1].join(if cfg!(windows) { "git.exe" } else { "git" });
    cleared(&mut Command::new(git))
        .env("PATH", std::env::join_paths(&dirs).expect("the PATH joins"))
        .current_dir(fixture.root())
        .args(args)
        .output()
        .expect("git runs")
}

#[test]
fn the_shims_decide_with_no_bash_on_the_path_and_refuse_a_check_that_needs_it() {
    let fixture = shimmed("hooks-no-bash");
    let output = git_without_bash(
        &fixture,
        &built(),
        &["commit", "-q", "--allow-empty", "-m", "x"],
    );
    assert!(!output.status.success(), "a commit on main passed");
    assert!(
        both(&output).contains("hook/commit-on-main"),
        "{}",
        both(&output)
    );
    fixture.git(&["switch", "-q", "-c", "feature"]);
    let output = git_without_bash(
        &fixture,
        &built(),
        &["commit", "-q", "--allow-empty", "-m", "x"],
    );
    assert!(output.status.success(), "{}", both(&output));

    let fixture = shimmed("hooks-no-bash-merge");
    merging(&fixture);
    let output = git_without_bash(&fixture, &built(), MERGE);
    assert!(
        output.status.success(),
        "a merge with no check to run: {}",
        both(&output)
    );

    let fixture = shimmed("hooks-no-bash-docs");
    docs(&fixture, 0);
    merging(&fixture);
    let output = git_without_bash(&fixture, &built(), MERGE);
    assert!(!output.status.success(), "a check that cannot run passed");
    assert!(
        both(&output).contains("pre-merge-commit: refused — the documentation check is red"),
        "{}",
        both(&output)
    );
}

#[test]
fn a_panic_of_nuc_hook_is_refused_and_named() {
    for hook in every_shim() {
        let fixture = shimmed(&format!("hooks-panic-{hook}"));
        let args = trigger(&fixture, &hook);
        let output = git_hooked(&fixture, Some(&built()), &[("NUC_TEST_PANIC", "1")], args);
        assert!(!output.status.success(), "{hook} passed a panic");
        assert!(
            both(&output).contains(&format!("{hook}: refused — nuc hook {hook} exited 101"))
                && both(&output).contains("nuc: internal error"),
            "{hook}: {}",
            both(&output)
        );
    }
}

/// On Unix, where `sh` is on the `PATH` to run a shim by hand.
#[cfg(unix)]
#[test]
fn a_shim_refuses_where_git_names_no_work_tree() {
    let fixture = shimmed("hooks-no-work-tree");
    let bin = stand_in(&fixture, "exit 3");
    let outside = fixture.root().with_extension("outside");
    std::fs::create_dir_all(&outside).expect("a directory outside any repository");
    for hook in every_shim() {
        let output = cleared(&mut Command::new("sh"))
            .arg(shims().join(&hook))
            .env("PATH", path_with(Some(&bin)))
            .env(
                "GIT_CEILING_DIRECTORIES",
                outside.parent().expect("a parent"),
            )
            .current_dir(&outside)
            .output()
            .expect("sh runs");
        assert!(!output.status.success(), "{hook} passed");
        assert!(
            both(&output).contains(&format!("{hook}: refused — git names no work tree")),
            "{hook}: {}",
            both(&output)
        );
    }
    let _ = std::fs::remove_dir_all(outside);
    let _ = std::fs::remove_dir_all(bin);
}
