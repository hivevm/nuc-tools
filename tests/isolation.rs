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
//! Linux, x64 and arm64.

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
