//! The fixture helpers under an inherited decoy. Verifies: ADR-draft-fixture-repositories
//!
//! The test runs itself again as a child process with `GIT_DIR`, `GIT_WORK_TREE`, and
//! `GIT_CONFIG_PARAMETERS` set to a decoy repository, and the child builds a fixture through the
//! helpers: the fixture holds its commit where the helper said, with the fixed identity, and the
//! decoy is untouched. The variables are set on the child, never on this process, whose tests
//! run side by side.

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
    assert_eq!(snapshot(decoy.root()), before, "the decoy changed");
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
