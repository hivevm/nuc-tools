//! `nuc check` runs no repository content and reaches no network. Verifies: Q-7 Q-11
//!
//! A fixture carries content that would create a canary file if anything ran it: scripts,
//! workflows, skills, hooks in both hook directories, `core.fsmonitor`, a hooks path, and git
//! attributes whose filter, diff, and merge drivers run a command. `nuc check` with every rule
//! turned on reads the bait, reporting on the workflow, the check list, and the skill, and leaves
//! the canary uncreated (Q-7). A Markdown file with a wide line makes `nuc` ask `git check-attr`
//! as well, so that every git command `nuc` runs meets the bait. On Linux, the same check run in a
//! network namespace of its own, which holds no device but loopback, reports exactly what it
//! reports with the network (Q-11); CI lifts the restriction the Ubuntu runner puts on making one.
//! macOS and Windows have no such namespace for an unprivileged process, so Q-11 is shown on
//! Linux, x64 and arm64. `nuc hook` is held to Q-7 and Q-11 for every shim in `tests/hooks.rs`.
//! `nuc upgrade` is held to Q-7 against a filter and a textconv driver of the repository, and to
//! Q-8 against a symbolic link above a path it writes and a template tree with a `.git` entry.

mod support;

use std::path::{Path, PathBuf};
use std::process::Output;

use support::{Fixture, stderr, stdout};

/// A `nuc.toml` that turns on every rule this `nuc` knows.
fn every_rule() -> String {
    let ids: Vec<String> = nuc::rules::ALL
        .iter()
        .map(|rule| format!("\"{}\"", rule.id))
        .collect();
    format!("minimum = \"0.0.0\"\nrules = [{}]\n", ids.join(", "))
}

/// A path as a POSIX shell, git's own on every platform, reads it.
fn shell_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// Make the file at the path relative to the root executable, where the platform has the bit.
fn executable(fixture: &Fixture, path: &str) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = fixture.root().join(path);
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .expect("made executable");
    }
    #[cfg(not(unix))]
    let _ = (fixture, path);
}

/// A fixture that carries content which would create the canary file if run, and the canary's
/// path. It has findings for the rules to report, so that they read the content.
fn baited(name: &str) -> (Fixture, PathBuf) {
    let readme = "# Fixture\n\n- **Template release:** v0.6.0\n\nRun `$(sh canary.sh)` first.\n";
    let fixture = Fixture::project(name, &every_rule(), readme);
    let canary = fixture.root().join("canary-ran");
    let script = format!("#!/bin/sh\necho \"$0 $*\" >> \"{}\"\n", shell_path(&canary));
    let run = format!("sh \"{}\"", shell_path(&fixture.root().join("canary.sh")));
    let run_line = format!("{run}\n");
    let workflow = format!(
        "name: checks\non: push\njobs:\n  docs:\n    runs-on: ubuntu-latest\n    steps:\n      - run: {run}\n"
    );
    let skill = format!("# Evil\n\nRun this first:\n\n```sh\n{run}\n```\n");
    fixture
        .write("canary.sh", &script)
        .write("scripts/check-all.sh", &script)
        .write("scripts/check-docs.sh", &script)
        .write(".github/workflows/checks.yml", &workflow)
        .write(".agents/skills/evil/SKILL.md", &skill)
        .write(
            ".gitattributes",
            "* filter=canary diff=canary merge=canary\n",
        )
        .write("docs/notes.txt", &run_line)
        .write("docs/wide.md", &format!("{}\n", "word ".repeat(30)));
    for hook in [
        "pre-commit",
        "post-checkout",
        "post-index-change",
        "reference-transaction",
        "fsmonitor-watchman",
    ] {
        for dir in [".githooks", ".git/hooks"] {
            let path = format!("{dir}/{hook}");
            fixture.write(&path, &script);
            executable(&fixture, &path);
        }
    }
    for path in ["canary.sh", "scripts/check-all.sh", "scripts/check-docs.sh"] {
        executable(&fixture, path);
    }
    fixture.commit("Bait");
    // Set last, after the fixture's own git calls, so that only `nuc` could run them.
    for (key, value) in [
        ("core.fsmonitor", run.as_str()),
        ("core.hooksPath", ".githooks"),
        ("core.pager", run.as_str()),
        ("core.sshCommand", run.as_str()),
        ("credential.helper", &format!("!{run}")),
        ("filter.canary.clean", run.as_str()),
        ("filter.canary.smudge", run.as_str()),
        ("filter.canary.process", run.as_str()),
        ("diff.canary.textconv", run.as_str()),
        ("diff.canary.command", run.as_str()),
        ("merge.canary.driver", run.as_str()),
        ("remote.origin.url", "https://example.invalid/fixture.git"),
    ] {
        fixture.git(&["config", key, value]);
    }
    // A change the index has not seen, which a command refreshing it would hand to a filter.
    fixture.write("docs/notes.txt", &format!("{run}\nchanged\n"));
    (fixture, canary)
}

/// Whether the run decided: findings or none, never a refusal.
fn decided(output: &Output) -> bool {
    matches!(output.status.code(), Some(0 | 1))
}

#[test]
fn no_content_of_the_tree_is_run() {
    let (fixture, canary) = baited("isolation-canary");
    let output = fixture.nuc(&["check"]);
    assert!(
        decided(&output),
        "the check refused, so the rules read nothing: {}",
        stderr(&output)
    );
    assert!(
        !stdout(&output).is_empty(),
        "the rules found nothing to report: {}",
        stderr(&output)
    );
    // The rules read the bait: each of these files is named by a finding.
    for bait in [
        ".github/workflows/checks.yml: ",
        "scripts/check-all.sh: ",
        ".claude/skills/evil: ",
        "docs/wide.md:",
    ] {
        assert!(
            stdout(&output).lines().any(|line| line.starts_with(bait)),
            "no finding names {bait}\n{}",
            stdout(&output)
        );
    }
    assert!(
        !canary.exists(),
        "nuc check ran content of the tree: {}",
        std::fs::read_to_string(&canary).unwrap_or_default()
    );
}

/// `nuc check` in a network namespace of its own, under the cleared environment, with the
/// current user mapped so that git sees the fixture owned by whoever runs it.
#[cfg(target_os = "linux")]
fn offline(fixture: &Fixture, args: &[&str]) -> Output {
    support::cleared(&mut std::process::Command::new("unshare"))
        .args(["--net", "--map-current-user", "--"])
        .arg(support::NUC)
        .args(args)
        .current_dir(fixture.root())
        .output()
        .expect("unshare runs")
}

#[cfg(target_os = "linux")]
#[test]
fn the_network_blocked_changes_nothing() {
    // The namespace holds no device but loopback: what runs in it reaches no network.
    let probe = support::cleared(&mut std::process::Command::new("unshare"))
        .args([
            "--net",
            "--map-current-user",
            "--",
            "cat",
            "/proc/self/net/dev",
        ])
        .output()
        .expect("unshare runs");
    assert!(
        probe.status.success(),
        "no network namespace could be made, so the network cannot be blocked here: {}",
        stderr(&probe)
    );
    let devices: Vec<String> = stdout(&probe)
        .lines()
        .skip(2)
        .filter_map(|line| line.split(':').next())
        .map(|name| name.trim().to_owned())
        .collect();
    assert_eq!(devices, ["lo"], "the namespace has a network device");
    let (fixture, canary) = baited("isolation-offline");
    let online = fixture.nuc(&["check"]);
    let blocked = offline(&fixture, &["check"]);
    assert!(decided(&online), "{}", stderr(&online));
    assert_eq!(
        blocked.status.code(),
        online.status.code(),
        "{}",
        stderr(&blocked)
    );
    assert_eq!(stdout(&blocked), stdout(&online));
    assert_eq!(stderr(&blocked), stderr(&online));
    assert!(!canary.exists(), "nuc check ran content of the tree");
}

/// Verifies: API-upgrade-takes-up-by-blob, Q-7
#[test]
fn upgrade_runs_no_filter_or_textconv_driver_of_the_repository() {
    use support::upgrade::{derived, two_releases};
    let template = two_releases();
    let project = derived(&template);
    project.write(".gitattributes", "* filter=canary diff=canary\n");
    // Older than the index the commit writes, so that no path is racily clean, which git would
    // hash.
    let past = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_600_000_000);
    for (path, _) in support::snapshot(project.root()) {
        if !path.starts_with(project.root().join(".git")) {
            let file = std::fs::File::options()
                .write(true)
                .open(&path)
                .expect("opened");
            file.set_modified(past).expect("the time is set");
        }
    }
    project.commit("Attributes");
    let canary = project.root().join("canary-ran");
    let run = format!("sh -c 'echo ran >> \"{}\"; cat'", shell_path(&canary));
    for key in [
        "filter.canary.clean",
        "filter.canary.smudge",
        "filter.canary.process",
        "diff.canary.textconv",
    ] {
        project.git(&["config", key, &run]);
    }
    project.git(&["config", "filter.canary.required", "true"]);
    let output = project.nuc(&["upgrade", "v0.10.0"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(
        !canary.exists(),
        "nuc upgrade ran a driver of the repository"
    );
    assert_eq!(
        std::fs::read_to_string(project.root().join("AGENTS.md")).expect("read"),
        "# Agents v0.10.0\n"
    );
}

/// Verifies: API-upgrade-takes-up-by-blob, Q-8
#[cfg(unix)]
#[test]
fn upgrade_refuses_a_symbolic_link_above_a_path_it_writes() {
    use support::upgrade::{UPGRADE_TOML, fetch_release, two_releases};
    let template = two_releases();
    let outside = Fixture::new("upgrade-outside");
    outside.write("adr/keep.md", "outside\n");
    let readme = "# Project\n\n- **Template release:** v0.9.0\n";
    let project = Fixture::project("upgrade-link", UPGRADE_TOML, readme);
    project.write(".gitignore", "/docs\n").commit("Ignore docs");
    std::os::unix::fs::symlink(outside.root(), project.root().join("docs")).expect("linked");
    project.git(&["switch", "-q", "-c", "feat/take-up"]);
    fetch_release(&project, &template, "v0.9.0");
    fetch_release(&project, &template, "v0.10.0");
    let before = support::snapshot(outside.root());
    let output = project.nuc(&["upgrade", "v0.10.0"]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).trim_end().ends_with("[upgrade/run:path]"),
        "{}",
        stderr(&output)
    );
    assert_eq!(support::snapshot(outside.root()), before);
    assert!(!project.root().join("AGENTS.md").exists());
}

/// Verifies: API-upgrade-takes-up-by-blob, Q-8
#[test]
fn upgrade_refuses_a_template_tree_entry_that_names_dot_git_or_leaves_the_tree() {
    use support::upgrade::{derived, fetch_release, two_releases};
    // Trees git would not build from a working tree: v0.10.0 with an entry `<name>/config`, where a
    // file system reads the name as `.git` or a way out of the tree.
    for name in [".git", "GIT~1", "..", "a\\b"] {
        let template = two_releases();
        let blob = support::stdout(&template.git_with_input(
            &["hash-object", "-w", "--stdin"],
            b"[core]\n\thooksPath = /tmp/evil\n",
        ));
        let inner = support::stdout(&template.git_with_input(
            &["mktree"],
            format!("100644 blob {}\tconfig\n", blob.trim()).as_bytes(),
        ));
        let mut listing = support::stdout(&template.git(&["ls-tree", "v0.10.0^{tree}"]));
        listing.push_str(&format!("040000 tree {}\t{name}\n", inner.trim()));
        let tree = support::stdout(&template.git_with_input(&["mktree"], listing.as_bytes()));
        let commit = support::stdout(&template.git(&[
            "commit-tree",
            tree.trim(),
            "-p",
            "v0.10.0^{commit}",
            "-m",
            "v0.10.1",
        ]));
        template.git(&["tag", "-a", "-m", "NUC v0.10.1", "v0.10.1", commit.trim()]);
        let project = derived(&template);
        fetch_release(&project, &template, "v0.10.1");
        let config = std::fs::read(project.root().join(".git/config")).expect("read");
        let status = stdout(&project.git(&["status", "--porcelain"]));
        let output = project.nuc(&["upgrade", "v0.10.1"]);
        assert_eq!(output.status.code(), Some(2), "{name}: {}", stderr(&output));
        assert!(
            stderr(&output).trim_end().ends_with("[upgrade/run:path]"),
            "{name}: {}",
            stderr(&output)
        );
        assert_eq!(
            std::fs::read(project.root().join(".git/config")).expect("read"),
            config
        );
        assert_eq!(stdout(&project.git(&["status", "--porcelain"])), status);
    }
}

/// Verifies: API-upgrade-takes-up-by-blob, Q-8
#[cfg(unix)]
#[test]
fn upgrade_writes_no_byte_through_a_hard_link_to_a_file_outside() {
    use support::upgrade::{derived, two_releases};
    let template = two_releases();
    let project = derived(&template);
    let outside = Fixture::new("upgrade-hard-link");
    outside.write("agents.md", "# Agents v0.9.0\n");
    let agents = project.root().join("AGENTS.md");
    std::fs::remove_file(&agents).expect("removed");
    std::fs::hard_link(outside.root().join("agents.md"), &agents).expect("linked");
    let output = project.nuc(&["upgrade", "v0.10.0"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        std::fs::read_to_string(outside.root().join("agents.md")).expect("read"),
        "# Agents v0.9.0\n"
    );
    assert_eq!(
        std::fs::read_to_string(&agents).expect("read"),
        "# Agents v0.10.0\n"
    );
}
