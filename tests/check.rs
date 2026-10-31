//! `nuc check` and `nuc explain` end to end, on fixture repositories, the golden path.
//! Verifies: ADR-0013 ADR-0008
//!
//! The golden path of the project: a fixture built through git, `nuc` run as a process on it, its
//! exit code and output asserted. A run that could not decide exits 2 and names its cause on the
//! standard error; a finding is a line `<file>[:<line>]: <rule ID>: <message>` on the standard
//! output, and exits 1.

mod support;

use support::{CONFIG, Fixture, README, stderr, stdout};

fn code(output: &std::process::Output) -> Option<i32> {
    output.status.code()
}

#[test]
fn a_project_that_keeps_its_rules_passes() {
    let fixture = Fixture::project("passes", CONFIG, README);
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("nuc check passed (1 rules)"));
}

#[test]
fn a_finding_names_its_file_line_and_rule_and_fails() {
    let fixture = Fixture::project(
        "finding",
        CONFIG,
        "# Fixture\n\n- **Template release:** latest\n",
    );
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(1));
    assert!(
        stdout(&output).starts_with("README.md:3: docs/template-release: "),
        "{}",
        stdout(&output)
    );
}

#[test]
fn the_repository_is_found_from_a_subdirectory_and_by_repo() {
    let fixture = Fixture::project("found", CONFIG, "no release line\n");
    fixture.write("docs/notes.md", "notes\n");
    assert_eq!(code(&fixture.nuc_in("docs", &["check"])), Some(1));
    let elsewhere = Fixture::new("elsewhere");
    let repo = fixture.root().to_str().expect("a UTF-8 path");
    assert_eq!(code(&elsewhere.nuc(&["check", "--repo", repo])), Some(1));
}

#[test]
fn a_missing_nuc_toml_refuses() {
    let fixture = Fixture::new("missing");
    fixture.write("README.md", README).commit("Initial commit");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2));
    assert!(
        stderr(&output).contains("nuc.toml: not found"),
        "{}",
        stderr(&output)
    );
    assert!(
        stderr(&output).contains("install or update nuc"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn an_unreadable_nuc_toml_refuses() {
    for (name, config) in [
        ("garbage", "this is not toml\n"),
        (
            "unknown-key",
            "minimum = \"0.0.0\"\nrules = []\nlevel = \"strict\"\n",
        ),
        ("no-rules", "minimum = \"0.0.0\"\n"),
    ] {
        let fixture = Fixture::project(name, config, README);
        let output = fixture.nuc(&["check"]);
        assert_eq!(code(&output), Some(2), "{name}");
        assert!(
            stderr(&output).contains("nuc.toml:"),
            "{name}: {}",
            stderr(&output)
        );
        assert!(
            stderr(&output).contains("unreadable"),
            "{name}: {}",
            stderr(&output)
        );
        assert!(stderr(&output).contains("install or update nuc"), "{name}");
    }
}

#[test]
fn a_nuc_toml_that_is_no_utf8_refuses() {
    let fixture = Fixture::project("binary", CONFIG, README);
    std::fs::write(fixture.root().join("nuc.toml"), [0xff, 0xfe, 0x00]).expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2));
    assert!(stderr(&output).contains("not UTF-8"), "{}", stderr(&output));
}

#[test]
fn a_higher_minimum_refuses_and_names_how_to_update() {
    let config = "minimum = \"99.0.0\"\nrules = [\"docs/template-release\"]\n";
    let fixture = Fixture::project("too-old", config, README);
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2));
    assert!(
        stderr(&output).contains("requires nuc 99.0.0 or later"),
        "{}",
        stderr(&output)
    );
    assert!(
        stderr(&output).contains("install or update nuc"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn an_unknown_rule_id_refuses_and_names_how_to_update() {
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/template-release\", \"docs/unknown\"]\n";
    let fixture = Fixture::project("unknown", config, README);
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2));
    assert!(
        stderr(&output).contains("'docs/unknown'"),
        "{}",
        stderr(&output)
    );
    assert!(
        stderr(&output).contains("install or update nuc"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_known_rule_that_is_not_listed_reports_nothing() {
    let config = "minimum = \"0.0.0\"\nrules = []\n";
    let fixture = Fixture::project("off", config, "no release line\n");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert_eq!(stdout(&output), "");
}

#[test]
fn outside_a_repository_it_refuses() {
    // A directory of its own beside the fixtures, without a repository.
    let dir = std::env::temp_dir().join(format!("nuc-not-a-repo-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the directory is created");
    let output = support::cleared(&mut std::process::Command::new(support::NUC))
        .current_dir(&dir)
        .arg("check")
        .output()
        .expect("nuc runs");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(code(&output), Some(2));
    assert!(
        stderr(&output).contains("is not inside a git working tree"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn explain_gives_the_reason_of_a_known_rule_and_refuses_an_unknown_one() {
    let fixture = Fixture::new("explain");
    let output = fixture.nuc(&["explain", "docs/template-release"]);
    assert_eq!(code(&output), Some(0));
    assert!(stdout(&output).starts_with("docs/template-release: README.md names the release"));
    assert_eq!(code(&fixture.nuc(&["explain", "docs/unknown"])), Some(2));
}

#[test]
fn an_unknown_command_refuses_with_the_usage() {
    let fixture = Fixture::new("usage");
    for args in [
        &[][..],
        &["chek"],
        &["check", "--repo"],
        &["check", "extra"],
    ] {
        let output = fixture.nuc(args);
        assert_eq!(code(&output), Some(2), "{args:?}");
        assert!(stderr(&output).contains("usage: nuc check"), "{args:?}");
    }
}

#[test]
fn a_readme_that_is_no_utf8_is_read_byte_by_byte() {
    let fixture = Fixture::project("latin-1", CONFIG, README);
    let bytes = b"Caf\xe9\n\n- **Template release:** v0.5.0\n";
    std::fs::write(fixture.root().join("README.md"), bytes).expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
}

#[test]
fn a_readme_with_a_nul_byte_is_not_decided() {
    let fixture = Fixture::project("nul", CONFIG, README);
    let bytes = b"- **Template release:** v0.5.0\n\0\n";
    std::fs::write(fixture.root().join("README.md"), bytes).expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2));
    assert!(stderr(&output).contains("NUL byte"), "{}", stderr(&output));
}

#[test]
fn an_adr_with_a_nul_byte_is_not_decided_and_names_the_file() {
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/adr-header\"]\n";
    let fixture = Fixture::project("adr-nul", config, README);
    fixture.write(
        "docs/adr/0001-a.md",
        "# ADR-0001: A\n\n- **Status:** 🟢 accepted\n",
    );
    std::fs::write(
        fixture.root().join("docs/adr/0002-b.md"),
        b"# ADR-0002\n\0\n",
    )
    .expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2));
    assert!(
        stderr(&output).contains("0002-b.md: holds a NUL byte"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn the_rules_of_the_record_report_end_to_end() {
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/adr-index\", \"docs/adr-numbering\", \"docs/adr-header\"]\n";
    let fixture = Fixture::project("adr-record", config, README);
    fixture.write("docs/adr/README.md", "| ADR | Title | Applies to | Status |\n|---|---|---|---|\n| [0001](0001-a.md) | A | every commit | 🟢 accepted |\n");
    fixture.write(
        "docs/adr/0001-a.md",
        "# ADR-0001: A\n\n- **Status:** 🟢 proposed\n- **Applies to:** every commit\n",
    );
    fixture.write(
        "docs/adr/0003-c.md",
        "# ADR-0003: C\n\n- **Status:** 🟢 accepted\n- **Applies to:** x\n",
    );
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(1), "{}", stderr(&output));
    let out = stdout(&output);
    for line in [
        "docs/adr/README.md: docs/adr-index: ADR index: file '0003-c.md' exists but is not listed in the index",
        "docs/adr: docs/adr-numbering: ADR numbering: expected '0002-*.md' at position 2, found '0003-c.md'",
        "docs/adr/0001-a.md: docs/adr-header: status line reads '🟢 proposed', but 🟢 means 'accepted'",
    ] {
        assert!(
            out.lines().any(|l| l.starts_with(line)),
            "missing {line}:\n{out}"
        );
    }
}

/// A project whose nuc.toml turns on the rules of the record, with the index and one ADR.
fn record(name: &str, row_target: &str) -> Fixture {
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/adr-index\", \"docs/adr-numbering\", \"docs/adr-header\"]\n";
    let fixture = Fixture::project(name, config, README);
    let row = format!("| [0001]({row_target}) | A | every commit | 🟢 accepted |\n");
    fixture.write(
        "docs/adr/README.md",
        &format!("| ADR | Title | Applies to | Status |\n|---|---|---|---|\n{row}"),
    );
    fixture.write(
        "docs/adr/0001-a.md",
        "# ADR-0001: A\n\n- **Status:** 🟢 accepted\n- **Applies to:** every commit\n",
    );
    fixture
}

#[test]
fn a_binary_file_beside_the_adrs_is_not_read() {
    let fixture = record("adr-png", "0001-a.md");
    std::fs::write(
        fixture.root().join("docs/adr/diagram.png"),
        b"\x89PNG\r\n\x1a\n\0\0",
    )
    .expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(
        code(&output),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
}

#[test]
fn a_row_target_resolves_as_the_bash_s_file_test_resolves_it() {
    let fixture = record("adr-dot", "./0001-a.md");
    let output = fixture.nuc(&["check"]);
    let out = stdout(&output);
    assert!(!out.contains("no such ADR file exists"), "{out}");
    assert!(out.contains("row for 0001 links to './0001-a.md'"), "{out}");
}

#[test]
fn a_row_target_outside_the_repository_is_not_decided() {
    // A file beside the fixture, outside its repository, which a row reaches from docs/adr/.
    let probe = Fixture::new("adr-outside-probe");
    let outside = probe.root().join("outside.md");
    std::fs::write(&outside, "- **Status:** 🟢 accepted\n").expect("written");
    let up = format!(
        "../../../{}/outside.md",
        probe
            .root()
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a name")
    );
    let fixture = record("adr-outside", &up);
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("outside the repository"),
        "{}",
        stderr(&output)
    );
}
