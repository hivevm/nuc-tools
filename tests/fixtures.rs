//! The fixture helpers under an inherited decoy and an inherited `HOME`.
//! Verifies: ADR-0026
//!
//! The test runs itself again as a child process with `GIT_DIR`, `GIT_WORK_TREE`, and
//! `GIT_CONFIG_PARAMETERS` set to a decoy repository, and the child builds a fixture through the
//! helpers: the fixture holds its commit where the helper said, with the fixed identity, and the
//! decoy is untouched. The variables are set on the child, never on this process, whose tests
//! run side by side. A fixture checks out the bytes it commits, whatever `core.autocrlf` the
//! system's configuration sets, and whatever line endings the parent's global configuration asks
//! for, which reaches no fixture.

mod support;

use std::env;
use std::process::Command;

use support::{Fixture, snapshot, stdout};

/// Set in the child, which then builds the fixture instead of starting another child.
const CHILD: &str = "NUC_FIXTURES_DECOY_CHILD";

#[test]
fn the_helpers_build_where_they_say_under_an_inherited_decoy() {
    if env::var_os(CHILD).is_some() {
        build_in_child();
        return;
    }
    let decoy = Fixture::new("helpers-decoy");
    decoy.write("decoy.txt", "decoy\n").commit("Decoy commit");
    let before = snapshot(decoy.root());
    let output = Command::new(env::current_exe().expect("the test binary is known"))
        .args([
            "--exact",
            "the_helpers_build_where_they_say_under_an_inherited_decoy",
        ])
        .args(["--nocapture", "--test-threads=1"])
        .env(CHILD, "1")
        .env("GIT_DIR", decoy.root().join(".git"))
        .env("GIT_WORK_TREE", decoy.root())
        .env("GIT_CONFIG_PARAMETERS", "'user.name'='Decoy'")
        .output()
        .expect("the child runs");
    assert!(
        output.status.success(),
        "the child failed:\n{}",
        support::stderr(&output)
    );
    assert!(
        stdout(&output).contains("1 passed"),
        "the child ran no test:\n{}",
        stdout(&output)
    );
    let after = snapshot(decoy.root());
    assert!(
        after == before,
        "the decoy changed: {:?}",
        support::changed(&before, &after)
    );
}

fn build_in_child() {
    let fixture = Fixture::new("helpers-child");
    fixture
        .write("file.txt", "fixture\n")
        .commit("Fixture commit");
    assert!(
        fixture.root().join(".git").is_dir(),
        "the fixture has no git directory of its own"
    );
    let log = stdout(&fixture.git(&["log", "--format=%an %ae %ad %s", "--date=iso-strict"]));
    assert_eq!(
        log.trim(),
        "Fixture fixture@example.invalid 2026-01-01T00:00:00Z Fixture commit"
    );
    let files = stdout(&fixture.git(&["ls-files"]));
    assert_eq!(files.trim(), "file.txt");
}

#[test]
fn a_fixture_checks_out_the_bytes_it_commits() {
    let fixture = Fixture::new("helpers-bytes");
    let local = stdout(&fixture.git(&["config", "--local", "core.autocrlf"]));
    assert_eq!(local.trim(), "false");
    fixture.write("file.txt", "one\ntwo\n").commit("Bytes");
    std::fs::remove_file(fixture.root().join("file.txt")).expect("the file is removed");
    fixture.git(&["checkout", "--", "file.txt"]);
    let bytes = std::fs::read(fixture.root().join("file.txt")).expect("the file is read");
    assert_eq!(bytes, b"one\ntwo\n");
}

#[test]
fn a_global_attributes_file_of_the_parent_reaches_no_fixture() {
    if std::env::var_os(support::HOME_CHILD).is_none() {
        // git expands `~` to the `HOME` it read the configuration from: the parent's.
        support::rerun_under_a_home(
            "a_global_attributes_file_of_the_parent_reaches_no_fixture",
            "[core]\n\tattributesFile = ~/attributes\n",
            &[("attributes", "* eol=crlf\n")],
        );
        return;
    }
    // The fixture sets no attributes of its own, so a CR would come from the parent's.
    let fixture = Fixture::new("helpers-home-attributes");
    fixture.write("file.txt", "one\ntwo\n").commit("Bytes");
    std::fs::remove_file(fixture.root().join("file.txt")).expect("the file is removed");
    fixture.git(&["checkout", "--", "file.txt"]);
    let bytes = std::fs::read(fixture.root().join("file.txt")).expect("the file is read");
    assert_eq!(bytes, b"one\ntwo\n");
}

#[test]
fn a_home_left_behind_is_removed_and_nothing_else() {
    let dir = Fixture::new("helpers-sweep");
    for name in ["home-1", "home-2/kept", "other"] {
        std::fs::create_dir_all(dir.root().join(name)).expect("a directory is created");
    }
    support::sweep_homes(dir.root(), std::time::Duration::ZERO);
    assert!(!dir.root().join("home-1").exists(), "an empty HOME stayed");
    assert!(
        dir.root().join("home-2/kept").is_dir(),
        "a HOME with content went"
    );
    assert!(dir.root().join("other").is_dir(), "another directory went");
    // A HOME younger than the age stays.
    std::fs::create_dir(dir.root().join("home-3")).expect("a directory is created");
    support::sweep_homes(dir.root(), std::time::Duration::from_secs(3600));
    assert!(
        dir.root().join("home-3").is_dir(),
        "a HOME of a run beside this one went"
    );
}

#[test]
fn a_test_binary_built_from_another_checkout_names_both_and_how_to_build_it_again() {
    let built = Fixture::new("helpers-built-here");
    let running = Fixture::new("helpers-runs-there");
    let panic = std::panic::catch_unwind(|| {
        support::sources::same_checkout(built.root(), running.root());
    })
    .expect_err("two checkouts are not one");
    let message = panic
        .downcast_ref::<String>()
        .expect("the panic carries its message");
    for part in [
        &built.root().display().to_string(),
        &running.root().display().to_string(),
        "cargo clean -p nuc-tools",
    ] {
        assert!(message.contains(part), "{part} not in: {message}");
    }
}

/// The build-time root as the macro names it, assembled from pieces so that this source does not
/// name it itself.
fn build_time_root() -> String {
    format!("{}!(\"{}_{}\")", "env", "CARGO_MANIFEST", "DIR")
}

/// Each line of the sources, as `(path, text)`, that takes the build-time root from the macro,
/// named `<path>:<line>`, but in the one helper that checks the checkout, as the test pattern of
/// `docs/ARCHITECTURE.md` asks.
fn stray_build_time_roots(sources: &[(String, String)]) -> Vec<String> {
    let needle = build_time_root();
    sources
        .iter()
        .filter(|(path, _)| path != "tests/support/sources.rs")
        .flat_map(|(path, text)| {
            text.lines()
                .enumerate()
                .filter(|(_, line)| line.contains(&needle))
                .map(move |(n, _)| format!("{path}:{}", n + 1))
        })
        .collect()
}

#[test]
fn only_the_helper_that_checks_the_checkout_names_the_build_time_root() {
    let root = support::sources::root();
    let sources: Vec<(String, String)> = ["tests", "src"]
        .iter()
        .flat_map(|dir| support::sources::rust_files(&root.join(dir), None))
        .map(|path| {
            let name = path
                .strip_prefix(root)
                .expect("a source under the root")
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let text = std::fs::read_to_string(&path).expect("a source is readable");
            (name, text)
        })
        .collect();
    assert!(
        sources
            .iter()
            .any(|(path, text)| path == "tests/support/sources.rs"
                && text.contains(&build_time_root())),
        "the helper no longer names the build-time root, so this test watches nothing"
    );
    let stray = stray_build_time_roots(&sources);
    assert!(
        stray.is_empty(),
        "take the root from support::sources::root: {stray:?}"
    );

    // A test that takes its root from the macro is named by its file and line.
    let fixture = vec![
        ("tests/support/sources.rs".to_owned(), build_time_root()),
        (
            "tests/new.rs".to_owned(),
            format!(
                "use std::path::Path;\nlet root = Path::new({});\n",
                build_time_root()
            ),
        ),
    ];
    assert_eq!(stray_build_time_roots(&fixture), ["tests/new.rs:2"]);
}

#[test]
fn a_fixture_runs_nothing_in_the_background() {
    let fixture = Fixture::new("helpers-background");
    for (key, value) in [
        ("maintenance.auto", "false"),
        ("gc.auto", "0"),
        ("core.fsmonitor", "false"),
    ] {
        let local = stdout(&fixture.git(&["config", "--local", key]));
        assert_eq!(local.trim(), value, "{key}");
    }
}
