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

/// Verifies: Q-1
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

/// Verifies: Q-1
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

/// Verifies: Q-1
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

/// Verifies: Q-1
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

/// Verifies: Q-1
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

/// Verifies: Q-1
#[test]
fn without_git_on_the_path_it_refuses_and_names_how_to_install_it() {
    let fixture = Fixture::project("no-git", CONFIG, README);
    let empty = fixture.root().join("empty-path");
    std::fs::create_dir_all(&empty).expect("created");
    let output = fixture.nuc_with(
        &[("PATH", empty.to_str().expect("a UTF-8 path"))],
        &["check"],
    );
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    let err = stderr(&output);
    assert!(err.contains("git could not be run"), "{err}");
    assert!(err.contains("install git"), "{err}");
}

/// Verifies: Q-1
#[cfg(debug_assertions)]
#[test]
fn a_panic_is_an_internal_error_that_names_its_cause_and_never_passes() {
    let fixture = Fixture::project("panic", CONFIG, README);
    let output = fixture.nuc_with(&[("NUC_TEST_PANIC", "1")], &["check"]);
    // ADR-0019: a panic exits 101, non-zero as the refusals are.
    assert_eq!(code(&output), Some(101), "{}", stderr(&output));
    assert_eq!(stdout(&output), "");
    let err = stderr(&output);
    assert!(err.starts_with("nuc: internal error at "), "{err}");
    assert!(err.contains("mod.rs:"), "{err}");
    assert!(
        err.contains(": NUC_TEST_PANIC is set — not decided"),
        "{err}"
    );
}

/// Verifies: G-6
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

/// Verifies: Q-12
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

/// A project whose nuc.toml turns on the reference rule, which reads every text file.
fn references(name: &str) -> Fixture {
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/adr-refs\"]\n";
    Fixture::project(name, config, README)
}

/// A reference to an ADR that does not exist, assembled so that this file names none itself.
fn missing_reference() -> String {
    format!("See {}-0099.\n", "ADR")
}

#[test]
fn the_text_files_are_those_git_lists_that_grep_reads_as_text() {
    let fixture = references("text-files");
    fixture.write(".gitignore", "ignored.md\n");
    fixture.write("ignored.md", &missing_reference());
    fixture.write("untracked.md", &missing_reference());
    fixture.write("blank.md", "\n\n\n");
    let mut binary = missing_reference().into_bytes();
    binary.extend_from_slice(b"\0");
    std::fs::write(fixture.root().join("early-nul.bin"), binary).expect("written");
    let output = fixture.nuc(&["check"]);
    let out = stdout(&output);
    assert!(out.contains("untracked.md:1: docs/adr-refs: "), "{out}");
    assert!(!out.contains("ignored.md"), "{out}");
    assert!(!out.contains("early-nul.bin"), "{out}");
    assert_eq!(code(&output), Some(1));
}

#[test]
fn a_path_git_quotes_names_no_file_as_for_the_bash() {
    let fixture = references("quoted-path");
    fixture.write("caf\u{e9}.md", &missing_reference());
    fixture.commit("A file with a name git quotes");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(0), "{}", stdout(&output));
}

#[cfg(unix)]
#[test]
fn an_unquoted_path_that_is_no_utf8_is_read_and_refused_only_where_a_finding_names_it() {
    use std::os::unix::ffi::OsStrExt;
    let fixture = references("text-files-bytes");
    fixture.git(&["config", "core.quotePath", "false"]);
    let name = fixture
        .root()
        .join(std::ffi::OsStr::from_bytes(b"n\xffo.txt"));
    // A file system that holds only UTF-8 names, as APFS on macOS, cannot have the path at all.
    if std::fs::write(&name, "nothing here\n").is_err() {
        return;
    }
    // git lists the path unquoted, and the bash reads the file: with nothing to report, it passes.
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    // A finding would have to name the path, which nuc cannot spell as the bash does.
    std::fs::write(&name, missing_reference()).expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    let err = stderr(&output);
    assert!(err.contains("n\u{FFFD}o.txt"), "{err}");
    assert!(err.contains("no UTF-8"), "{err}");
    assert!(err.contains("not decided"), "{err}");
    // The bash resolves the links and the attributes of a Markdown file from its path's bytes,
    // which nuc cannot spell: such a file is not decided, finding or not.
    std::fs::write(&name, "nothing here\n").expect("written");
    let dir = fixture.root().join(std::ffi::OsStr::from_bytes(b"d\xff"));
    std::fs::create_dir_all(&dir).expect("created");
    std::fs::write(dir.join("a.md"), "# A\n").expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    let err = stderr(&output);
    assert!(err.contains("d\u{FFFD}/a.md"), "{err}");
    assert!(err.contains("Markdown"), "{err}");
}

#[test]
fn a_nul_byte_past_grep_s_first_buffer_is_not_decided() {
    let fixture = references("late-nul");
    let mut late = missing_reference().into_bytes();
    late.extend(std::iter::repeat_n(b"line of text\n".as_slice(), 4000).flatten());
    late.push(0);
    std::fs::write(fixture.root().join("late.txt"), late).expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("late.txt: holds a NUL byte past its first"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_repository_without_a_text_file_is_not_decided() {
    // Nothing tracked, and everything new ignored, nuc.toml included.
    let fixture = Fixture::new("no-text");
    fixture.write(
        "nuc.toml",
        "minimum = \"0.0.0\"\nrules = [\"docs/adr-refs\"]\n",
    );
    fixture.write(".gitignore", "*\n");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("no text files found under"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_rule_that_reads_no_text_files_is_not_held_up_by_one() {
    let fixture = Fixture::project("unread", CONFIG, README);
    let mut late = b"x\n".repeat(20_000);
    late.push(0);
    std::fs::write(fixture.root().join("late.txt"), late).expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
}

#[test]
fn links_into_a_submodule_a_directory_and_with_a_title_resolve_as_for_the_bash() {
    let module = Fixture::new("links-module");
    module
        .write("guide.md", "# Guide\n\n## Setup\n")
        .commit("A module");
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/links\"]\n";
    let fixture = Fixture::project("links", config, README);
    let url = module.root().to_str().expect("a UTF-8 path");
    fixture.git(&[
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "add",
        "-q",
        url,
        "vendor/module",
    ]);
    fixture.write("docs/tools/x.sh", "a\nb\n");
    fixture.write(
        "docs/links.md",
        "[m](../vendor/module/guide.md#setup) [d](tools/) [t](tools/x.sh \"A title\") [l](tools/x.sh#L2)\n\n[x](../vendor/module/guide.md#nope) [y](missing/)\n",
    );
    let output = fixture.nuc(&["check"]);
    let out = stdout(&output);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines,
        [
            "docs/links.md:3: docs/links: link '../vendor/module/guide.md#nope' has no matching anchor '#nope' in docs/../vendor/module/guide.md",
            "docs/links.md:3: docs/links: broken relative link -> 'missing/'",
        ],
        "{}",
        stderr(&output)
    );
}

/// A file beside the fixture, outside its repository, and the path from the fixture's root to it.
fn outside(name: &str, file: &str, contents: &str) -> (Fixture, String) {
    let probe = Fixture::new(name);
    std::fs::write(probe.root().join(file), contents).expect("written");
    let dir = probe
        .root()
        .file_name()
        .and_then(|n| n.to_str())
        .expect("a name");
    let up = format!("../{dir}/{file}");
    (probe, up)
}

/// Verifies: Q-12
#[cfg(unix)]
#[test]
fn an_agents_md_that_links_outside_the_repository_is_not_decided() {
    let (_probe, up) = outside("agents-outside-probe", "outside.md", "## 1. Secret\n");
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/section-refs\"]\n";
    let fixture = Fixture::project("agents-outside", config, README);
    std::os::unix::fs::symlink(&up, fixture.root().join("AGENTS.md")).expect("linked");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(!stdout(&output).contains("secret"), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("AGENTS.md: resolves outside the repository"),
        "{}",
        stderr(&output)
    );
}

/// Verifies: Q-12
#[cfg(unix)]
#[test]
fn a_text_file_that_links_outside_the_repository_is_not_decided() {
    let (_probe, up) = outside("text-outside-probe", "outside.md", "Secret\n");
    let fixture = references("text-outside");
    std::os::unix::fs::symlink(&up, fixture.root().join("leak.md")).expect("linked");
    fixture.git(&["add", "leak.md"]);
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("leak.md: resolves outside the repository"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn links_the_bash_reads_no_file_for_are_decided_without_one() {
    let (_md, up_md) = outside("links-unread-md", "outside.md", "# Outside\n");
    let (_txt, up_txt) = outside("links-unread-txt", "outside.txt", "a\n");
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/links\"]\n";
    let fixture = Fixture::project("links-unread", config, README);
    // A superseded ADR is not read; an anchor that names no lines is not checked in a file
    // that is no Markdown, so neither file outside is read.
    fixture.write(
        "docs/adr/0001-a.md",
        &format!("# ADR-0001\n\n- **Status:** ⚪ superseded\n\n[x](../../{up_md}#L1)\n"),
    );
    fixture.write("docs/x.md", &format!("[t](../{up_txt}#main)\n"));
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
fn a_link_path_that_is_no_utf8_is_not_decided() {
    // Refused before the file system is asked, so no such file is needed, nor could every file
    // system hold one.
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/links\"]\n";
    let fixture = Fixture::project("links-bytes", config, README);
    std::fs::write(fixture.root().join("x.md"), b"[c](caf\xe9.txt)\n").expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("x.md:1: a link names a path that is no UTF-8"),
        "{}",
        stderr(&output)
    );
}

/// A project on the template release given, with only the supersession rule turned on, and ADR
/// 0001 flipped for 0002, which is only proposed.
fn flipped_early(name: &str, release_line: &str) -> Fixture {
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/adr-supersessions\"]\n";
    let fixture = Fixture::project(name, config, &format!("# Fixture\n\n{release_line}\n"));
    let by2 = format!("⚪ superseded by {}-0002", "ADR");
    fixture.write(
        "docs/adr/0001-a.md",
        &format!("# A\n\n- **Status:** {by2}\n- **Applies to:** x\n"),
    );
    fixture.write(
        "docs/adr/0002-b.md",
        &format!(
            "# B\n\n- **Status:** 🟡 proposed\n- **Supersedes:** {}-0001\n- **Applies to:** x\n",
            "ADR"
        ),
    );
    fixture
}

/// Verifies: ADR-0021
#[test]
fn a_rule_checks_a_tree_as_the_release_it_carries() {
    let old = flipped_early("release-0.5", "- **Template release:** v0.5.1");
    let output = old.nuc(&["check"]);
    assert_eq!(
        code(&output),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    for (name, line) in [
        ("release-0.6", "- **Template release:** v0.6.0"),
        (
            "release-later",
            "- **Template release:** [v9.0.0](https://example.invalid)",
        ),
        ("release-unreleased", "- **Template release:** unreleased"),
    ] {
        let output = flipped_early(name, line).nuc(&["check"]);
        assert_eq!(code(&output), Some(1), "{line}: {}", stderr(&output));
        assert!(
            stdout(&output).contains("docs/adr/0001-a.md: docs/adr-supersessions: superseded by"),
            "{line}: {}",
            stdout(&output)
        );
    }
}

#[test]
fn a_rule_that_checks_by_the_release_refuses_a_tree_it_cannot_place() {
    for (name, line, why) in [
        (
            "release-none",
            "No release line.",
            "no Template release line",
        ),
        (
            "release-bad",
            "- **Template release:** soon",
            "no Template release line",
        ),
        (
            "release-early",
            "- **Template release:** v0.4.0",
            "v0.4.0 is before v0.5.0",
        ),
    ] {
        let output = flipped_early(name, line).nuc(&["check"]);
        assert_eq!(code(&output), Some(2), "{line}: {}", stdout(&output));
        assert!(stderr(&output).contains(why), "{line}: {}", stderr(&output));
    }
}

/// Verifies: ADR-0021
#[test]
fn without_a_release_line_the_template_release_rule_fails_the_run_and_a_rule_reports_what_every_release_does()
 {
    let fixture = flipped_early("release-reported", "No release line.");
    fixture.write(
        "nuc.toml",
        "minimum = \"0.0.0\"\nrules = [\"docs/template-release\", \"docs/adr-supersessions\"]\n",
    );
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(1), "{}", stderr(&output));
    let out = stdout(&output);
    assert!(
        out.contains("README.md: docs/template-release: no"),
        "{out}"
    );
    // Only v0.6.0 fails an ADR flipped early, so the finding is not one every release reports.
    assert!(!out.contains("docs/adr-supersessions"), "{out}");
}

#[test]
fn without_a_readme_a_rule_that_checks_by_the_release_is_not_decided() {
    let fixture = flipped_early("release-no-readme", "No release line.");
    fixture.write(
        "nuc.toml",
        "minimum = \"0.0.0\"\nrules = [\"docs/template-release\", \"docs/adr-supersessions\"]\n",
    );
    std::fs::remove_file(fixture.root().join("README.md")).expect("removed");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
}

/// A project with only the skill pointer rule on and the skill `review` under `.agents/skills/`.
fn skills(name: &str) -> Fixture {
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/skill-pointers\"]\n";
    let fixture = Fixture::project(name, config, README);
    fixture.write(".agents/skills/review/SKILL.md", "# Review\n");
    fixture
}

#[test]
fn a_skill_pointer_that_is_a_copy_or_missing_fails_as_for_the_bash() {
    let missing = skills("skills-missing");
    let out = stdout(&missing.nuc(&["check"]));
    assert!(
        out.contains(".claude/skills/review: docs/skill-pointers: no pointer — the skill .agents/skills/review is invisible to Claude Code"),
        "{out}"
    );
    let copy = skills("skills-copy");
    copy.write(".claude/skills/review/SKILL.md", "# Review\n");
    copy.write(".claude/skills/stray/SKILL.md", "# Stray\n");
    let out = stdout(&copy.nuc(&["check"]));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        [
            ".claude/skills/review: docs/skill-pointers: is a copy of the skill, not a symlink — a second source drifts; replace it: rm -r .claude/skills/review && ln -s ../../.agents/skills/review .claude/skills/review",
            ".claude/skills/stray: docs/skill-pointers: is not a symlink into .agents/skills/ — a skill lives there, where every agent reads it, and is pointed at from here (ADR-0003)",
        ]
    );
}

#[cfg(unix)]
#[test]
fn a_skill_pointer_that_resolves_passes_and_one_that_dangles_or_strays_fails() {
    use std::os::unix::fs::symlink;
    let fixture = skills("skills-links");
    fixture.write(".agents/skills/other/SKILL.md", "# Other\n");
    let pointers = fixture.root().join(".claude/skills");
    std::fs::create_dir_all(&pointers).expect("created");
    symlink("../../.agents/skills/review", pointers.join("review")).expect("linked");
    symlink("../../.agents/skills/review", pointers.join("other")).expect("linked");
    symlink("../../.agents/skills/gone", pointers.join("gone")).expect("linked");
    let output = fixture.nuc(&["check"]);
    assert_eq!(
        stdout(&output).lines().collect::<Vec<_>>(),
        [
            ".claude/skills/gone: docs/skill-pointers: dangles — its target does not exist; point it at .agents/skills/gone or delete it",
            ".claude/skills/other: docs/skill-pointers: points at '.agents/skills/review', not at the skill .agents/skills/other",
        ],
        "{}",
        stderr(&output)
    );
}

#[test]
fn the_project_layout_is_checked_against_the_tree_and_what_git_tracks() {
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/project-layout\"]\n";
    let readme = format!(
        "{README}\n## Project Layout\n\n```text\nREADME.md\nnuc.toml\nsrc/\nsrc/*.rs   the code\ndocs/*.txt\ngone/\n```\n"
    );
    let fixture = Fixture::project("layout", config, &readme);
    fixture.write("src/lib.rs", "\n");
    fixture.write("extra.txt", "x\n");
    fixture.git(&["add", "-A"]);
    let output = fixture.nuc(&["check"]);
    assert_eq!(
        stdout(&output).lines().collect::<Vec<_>>(),
        [
            "README.md: docs/project-layout: Project Layout lists 'docs/*.txt', which matches nothing in this repository",
            "README.md: docs/project-layout: Project Layout lists 'gone/', which is not a directory of this repository",
            "README.md: docs/project-layout: Project Layout does not name 'extra.txt', a top-level part of this repository",
        ],
        "{}",
        stderr(&output)
    );
}

/// A project with only the rule given on.
fn only(name: &str, rule: &str, readme: &str) -> Fixture {
    let config = format!("minimum = \"0.0.0\"\nrules = [\"{rule}\"]\n");
    Fixture::project(name, &config, readme)
}

const LISTS_README: &str = "# Fixture\n\n- **Template release:** v0.6.0\n\n<!-- required-checks begin -->\n`docs`\n<!-- required-checks end -->\n";

#[test]
fn the_check_lists_are_read_as_the_bash_globs_and_find_list_them() {
    let fixture = only("lists", "docs/check-lists", LISTS_README);
    fixture.write(
        ".github/workflows/checks.yml",
        "jobs:\n  docs:\n    steps:\n      - run: bash scripts/check-docs.sh\n",
    );
    fixture.write(
        "scripts/check-all.sh",
        "run \"Docs\" scripts/check-docs.sh\n",
    );
    fixture.write("scripts/check-docs.sh", "true\n");
    fixture.write("scripts/.dot.sh", "true\n");
    let inline = "jobs:\n  publish:\n    steps:\n      - run: make\n";
    fixture.write(".github/workflows/a.yaml", inline);
    fixture.write(".github/workflows/b.yml", inline);
    fixture.write(".github/workflows/.hidden.yml", inline);
    #[cfg(unix)]
    std::os::unix::fs::symlink("check-docs.sh", fixture.root().join("scripts/link.sh"))
        .expect("linked");
    let output = fixture.nuc(&["check"]);
    assert_eq!(
        stdout(&output).lines().collect::<Vec<_>>(),
        [
            ".github/workflows/b.yml: docs/check-lists: job 'publish' runs no script under scripts/ — the local gate cannot run what it does",
            ".github/workflows/a.yaml: docs/check-lists: job 'publish' runs no script under scripts/ — the local gate cannot run what it does",
            "scripts/.dot.sh: docs/check-lists: no workflow and not scripts/check-all.sh runs it — add it to both lists, or name it on an '# On instruction:' line of scripts/check-all.sh",
        ],
        "{}",
        stderr(&output)
    );
}

const LAYOUT_README: &str =
    "# Fixture\n\n## Project Layout\n\n```\nREADME.md\nnuc.toml\ndocs/plan/\n*.toml\n";

#[test]
fn an_absolute_layout_path_names_a_path_under_the_root_as_for_the_bash() {
    let readme = format!("{LAYOUT_README}/etc/\n/etc/hostname\n```\n");
    let output = only("layout-absolute", "docs/project-layout", &readme).nuc(&["check"]);
    assert_eq!(
        stdout(&output).lines().collect::<Vec<_>>(),
        [
            "README.md: docs/project-layout: Project Layout lists '/etc/', which is not a directory of this repository",
            "README.md: docs/project-layout: Project Layout lists '/etc/hostname', which does not exist",
        ],
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_layout_without_open_work_and_with_a_matching_pattern_passes() {
    let readme = format!("{LAYOUT_README}```\n");
    let output = only("layout-pass", "docs/project-layout", &readme).nuc(&["check"]);
    assert_eq!(
        code(&output),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
}

/// Verifies: Q-12
#[test]
fn a_layout_pattern_outside_the_repository_or_with_a_class_is_not_decided() {
    for (name, pattern, why) in [
        ("layout-up", "../*", "looks outside the repository"),
        (
            "layout-class",
            "[[:alpha:]]*",
            "uses a bracket nuc does not read",
        ),
    ] {
        let readme = format!("{LAYOUT_README}{pattern}\n```\n");
        let output = only(name, "docs/project-layout", &readme).nuc(&["check"]);
        assert_eq!(code(&output), Some(2), "{pattern}: {}", stdout(&output));
        assert!(
            stderr(&output).contains(why),
            "{pattern}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn a_repository_without_skills_passes_and_one_with_pointers_only_fails() {
    let none = only("skills-none", "docs/skill-pointers", README);
    assert_eq!(code(&none.nuc(&["check"])), Some(0));
    let pointers = only("skills-pointers-only", "docs/skill-pointers", README);
    pointers.write(".claude/skills/lonely/SKILL.md", "# Lonely\n");
    assert_eq!(
        stdout(&pointers.nuc(&["check"])).trim_end(),
        ".claude/skills/lonely: docs/skill-pointers: is not a symlink into .agents/skills/ — a skill lives there, where every agent reads it, and is pointed at from here (ADR-0003)"
    );
}

/// Verifies: Q-12
#[cfg(unix)]
#[test]
fn a_skill_directory_outside_the_repository_is_not_listed() {
    let (probe, _) = outside("skills-outside-probe", "evil-SKILL.md", "# Evil\n");
    std::fs::create_dir_all(probe.root().join("skills/evil")).expect("created");
    std::fs::write(probe.root().join("skills/evil/SKILL.md"), "# Evil\n").expect("written");
    let fixture = only("skills-outside", "docs/skill-pointers", README);
    std::fs::create_dir_all(fixture.root().join(".agents")).expect("created");
    std::os::unix::fs::symlink(
        probe.root().join("skills"),
        fixture.root().join(".agents/skills"),
    )
    .expect("linked");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("resolves outside the repository"),
        "{}",
        stderr(&output)
    );
}

/// Verifies: Q-12
#[cfg(unix)]
#[test]
fn a_record_directory_outside_the_repository_is_not_listed() {
    // A directory beside the fixture holding no file the record reads, so only its listing
    // reaches outside.
    let (probe, _) = outside("adr-dir-outside-probe", "notes.txt", "notes\n");
    let fixture = only("adr-dir-outside", "docs/adr-header", README);
    std::fs::create_dir_all(fixture.root().join("docs")).expect("created");
    std::os::unix::fs::symlink(probe.root(), fixture.root().join("docs/adr")).expect("linked");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("docs/adr: resolves outside the repository"),
        "{}",
        stderr(&output)
    );
}

/// Verifies: ADR-0013
#[test]
fn the_inherited_adrs_and_the_identity_are_checked_once_the_setup_file_is_gone() {
    let config =
        "minimum = \"0.0.0\"\nrules = [\"docs/inherited-adrs\", \"docs/template-identity\"]\n";
    let readme = format!(
        "# NUC — an Agentic Fixture\n\n{}",
        &README["# Fixture\n\n".len()..]
    );
    let fixture = Fixture::project("identity", config, &readme);
    fixture.write(
        "docs/adr/0001-a.md",
        "# A\n\n- **Status:** 🟢 accepted\n- **Deciders:** NUC maintainer\n- **Applies to:** x\n",
    );
    fixture.write("TEMPLATE-SETUP.md", "# Template setup\n");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(0), "{}", stdout(&output));
    std::fs::remove_file(fixture.root().join("TEMPLATE-SETUP.md")).expect("removed");
    let output = fixture.nuc(&["check"]);
    assert_eq!(
        stdout(&output).lines().collect::<Vec<_>>(),
        [
            "docs/adr/0001-a.md: docs/inherited-adrs: inherited from the template and not decided by this project — add your name to Deciders and accept it, or supersede it (docs/adr/README.md)",
            "README.md: docs/template-identity: the title is still the template's; give the project its name",
        ],
        "{}",
        stderr(&output)
    );
}

#[test]
fn an_adr_holding_the_unit_separator_is_not_decided() {
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/inherited-adrs\"]\n";
    let fixture = Fixture::project("unit-separator", config, README);
    fixture.write(
        "docs/adr/0001-a.md",
        "# A\n\n- **Status:** 🟢 accepted\n- **Deciders:** NUC maintainer\u{1f}\n- **Applies to:** x\n",
    );
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(stderr(&output).contains("0x1F"), "{}", stderr(&output));
}

#[test]
fn while_the_setup_file_exists_the_identity_files_are_not_read() {
    let fixture = only("identity-setup", "docs/template-identity", README);
    fixture.write("TEMPLATE-SETUP.md", "# Template setup\n");
    std::fs::write(fixture.root().join("LICENSE"), b"Copyright\0 Maintainer\n").expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
}

#[test]
fn a_markdown_file_git_marks_generated_is_not_read_for_its_width() {
    let long = format!("{}\n", "word ".repeat(24));
    for (name, attribute, findings) in [
        ("width-generated", "CHANGELOG.md linguist-generated\n", 0),
        (
            "width-generated-unset",
            "CHANGELOG.md -linguist-generated\n",
            1,
        ),
        (
            "width-generated-true",
            "CHANGELOG.md linguist-generated=true\n",
            0,
        ),
        ("width-generated-pattern", "*.md linguist-generated\n", 0),
        (
            "width-generated-false",
            "CHANGELOG.md linguist-generated=false\n",
            1,
        ),
    ] {
        let fixture = only(name, "docs/prose-width", README);
        fixture.write("CHANGELOG.md", &long);
        fixture.write(".gitattributes", attribute);
        let output = fixture.nuc(&["check"]);
        let out = stdout(&output);
        assert_eq!(
            out.lines().count(),
            findings,
            "{attribute}{out}{}",
            stderr(&output)
        );
        if findings == 1 {
            assert!(
                out.starts_with("CHANGELOG.md:1: docs/prose-width: 120 columns"),
                "{out}"
            );
        }
    }
}

#[test]
fn a_wide_line_the_bash_counts_otherwise_is_not_decided() {
    let fixture = only("width-bytes", "docs/prose-width", README);
    let mut line = b"[a\xff b](https://example.invalid/".to_vec();
    line.extend(b"a".repeat(100));
    line.extend(b")\n");
    std::fs::write(fixture.root().join("odd.md"), &line).expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("odd.md:1: a line over 100 bytes"),
        "{}",
        stderr(&output)
    );
}

/// Verifies: ADR-draft-rules-return-an-outcome
#[test]
fn a_rule_s_refusal_ends_the_run_before_the_findings_of_another_are_printed() {
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/template-release\", \"docs/prose-width\"]\n";
    let fixture = Fixture::project("outcome-refusal", config, "# X\n");
    let mut line = b"[a\xff b](https://example.invalid/".to_vec();
    line.extend(b"a".repeat(100));
    line.extend(b")\n");
    std::fs::write(fixture.root().join("odd.md"), &line).expect("written");
    // Without the line the template release rule reports the README; with it the run refuses.
    std::fs::remove_file(fixture.root().join("odd.md")).expect("removed");
    assert_eq!(code(&fixture.nuc(&["check"])), Some(1));
    std::fs::write(fixture.root().join("odd.md"), &line).expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "",
        "a finding was printed beside the refusal"
    );
    assert!(
        stderr(&output).contains("odd.md:1: a line over 100 bytes"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_draft_fails_where_ci_says_a_ready_change_lands_on_main() {
    let fixture = only("no-draft", "docs/no-draft-on-main", README);
    fixture.write("docs/adr/draft-next.md", "# Next\n");
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        "draft-next.md",
        fixture.root().join("docs/adr/draft-link.md"),
    )
    .expect("linked");
    let ready = [("GITHUB_BASE_REF", "main"), ("PULL_REQUEST_DRAFT", "false")];
    let output = fixture.nuc_with(&ready, &["check"]);
    assert_eq!(
        stdout(&output).trim_end(),
        "docs/adr/draft-next.md: docs/no-draft-on-main: a draft lands on main — give it its number first with scripts/number-adrs.sh (ADR-0002)",
        "{}",
        stderr(&output)
    );
    for vars in [
        &[("GITHUB_BASE_REF", "main"), ("PULL_REQUEST_DRAFT", "true")][..],
        &[("GITHUB_BASE_REF", "feat/topic")][..],
        &[][..],
    ] {
        let output = fixture.nuc_with(vars, &["check"]);
        assert_eq!(code(&output), Some(0), "{vars:?}: {}", stdout(&output));
    }
}

#[cfg(unix)]
#[test]
fn a_draft_whose_name_holds_a_newline_is_not_decided() {
    let fixture = only("no-draft-newline", "docs/no-draft-on-main", README);
    fixture.write("docs/adr/draft-a\nb.md", "# A\n");
    let ready = [("GITHUB_BASE_REF", "main"), ("PULL_REQUEST_DRAFT", "false")];
    let output = fixture.nuc_with(&ready, &["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("holds a newline"),
        "{}",
        stderr(&output)
    );
}

const TRACEABILITY: &str = "minimum = \"0.0.0\"\nrules = [\"traceability/criteria\"]\n";

#[test]
fn the_criteria_of_the_specification_are_checked_where_the_bash_finds_them() {
    let fixture = Fixture::project("criteria", TRACEABILITY, README);
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(1), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "docs/SPECIFICATION.md: traceability/criteria: not found — the criteria cannot be read\n"
    );
    // A directory in its place is no file, as `-f` finds it.
    std::fs::create_dir_all(fixture.root().join("docs/SPECIFICATION.md")).unwrap();
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(1), "{}", stderr(&output));
    std::fs::remove_dir(fixture.root().join("docs/SPECIFICATION.md")).unwrap();
    fixture.write(
        "docs/SPECIFICATION.md",
        "## Goals / Success Criteria\n\n- **G-1** — it works.\n- **G-1** — again.\n",
    );
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(1), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "docs/SPECIFICATION.md:4: traceability/criteria: identifier 'G-1' appears twice — \
         identifiers are never reused\n"
    );
    fixture.write(
        "docs/SPECIFICATION.md",
        "## Goals / Success Criteria\n\n- **G-1** — it works.\n",
    );
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
}

#[test]
fn a_specification_the_bash_reads_as_binary_is_not_decided() {
    let fixture = Fixture::project("criteria-nul", TRACEABILITY, README);
    fixture.write("docs/SPECIFICATION.md", "## Quality Goals\n- **Q-1** \0\n");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2));
    assert!(stderr(&output).contains("NUL byte"), "{}", stderr(&output));
}

#[test]
fn the_criteria_rule_is_explained() {
    let fixture = Fixture::project("criteria-explain", TRACEABILITY, README);
    let output = fixture.nuc(&["explain", "traceability/criteria"]);
    assert_eq!(code(&output), Some(0));
    assert!(
        stdout(&output).contains("**G-n** under Goals"),
        "{}",
        stdout(&output)
    );
}

const CITATIONS: &str = "minimum = \"0.0.0\"\nrules = [\"traceability/citations\"]\n";

/// A marker citing the identifier, assembled so that this file carries no literal one.
fn mark(id: &str) -> String {
    format!("{}: {id}\n", "Verifies")
}

#[test]
fn citations_are_read_from_the_files_the_bash_reads_them_from() {
    let fixture = Fixture::project("citations", CITATIONS, README);
    fixture.write(
        "docs/SPECIFICATION.md",
        "## Goals / Success Criteria\n\n- **G-1** — it works.\n",
    );
    fixture.write(".gitignore", "ignored.txt\n");
    fixture.write("notes.md", &mark("G-7"));
    fixture.write("ignored.txt", &mark("G-7"));
    let mut binary = b"\0".to_vec();
    binary.extend_from_slice(mark("G-7").as_bytes());
    std::fs::write(fixture.root().join("binary.bin"), binary).unwrap();
    fixture.write("tests/ok.txt", &mark("G-1"));
    fixture.commit("fixture");
    // A new file git does not ignore is read, and so is a path git would quote.
    fixture.write("new.txt", &format!("x\n{}", mark("G-8")));
    fixture.write("tests/é.txt", &mark("Q-3"));
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(1), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "new.txt:2: traceability/citations: cites 'G-8', which is not a criterion of \
         docs/SPECIFICATION.md\n\
         tests/é.txt:1: traceability/citations: cites 'Q-3', which is not a criterion of \
         docs/SPECIFICATION.md\n"
    );
}

#[test]
fn a_marker_beside_a_nul_byte_past_the_first_buffer_is_not_decided() {
    let fixture = Fixture::project("citations-nul", CITATIONS, README);
    let mut bytes = mark("G-1").into_bytes();
    bytes.extend(std::iter::repeat_n(b'x', 40 * 1024));
    bytes.push(0);
    std::fs::write(fixture.root().join("late.bin"), bytes).unwrap();
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(stderr(&output).contains("late.bin"), "{}", stderr(&output));
}

#[cfg(unix)]
#[test]
fn a_path_that_is_no_utf8_is_read_and_refused_only_where_it_holds_a_marker() {
    use std::os::unix::ffi::OsStrExt;
    let fixture = Fixture::project("citations-bytes", CITATIONS, README);
    fixture.write(
        "docs/SPECIFICATION.md",
        "## Goals / Success Criteria\n\n- **G-1** — it works.\n",
    );
    let name = fixture
        .root()
        .join(std::ffi::OsStr::from_bytes(b"n\xffo.txt"));
    // A file system that holds only UTF-8 names, as APFS on macOS, cannot have the path at all.
    if std::fs::write(&name, "nothing here\n").is_err() {
        return;
    }
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    std::fs::write(&name, mark("G-1")).unwrap();
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("n\u{FFFD}o.txt"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn the_criteria_without_a_test_are_named_in_the_summary_and_never_fail_the_run() {
    let config = "minimum = \"0.0.0\"\nrules = [\"traceability/coverage\"]\n";
    let fixture = Fixture::project("coverage", config, README);
    fixture.write(
        "docs/SPECIFICATION.md",
        "## Goals / Success Criteria\n\n- **G-1** — a.\n- **G-2** — b.\n\n## Quality Goals\n\n\
         - **Q-1** — c.\n",
    );
    fixture.write("tests/t.txt", &mark("G-2"));
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(0), "{}", stdout(&output));
    assert_eq!(
        stderr(&output),
        "nuc check passed (1 rules); 2 criteria without a test: G-1 Q-1.\n"
    );
    fixture.write(
        "docs/adr/0001-decision.md",
        "# ADR-0001: decision\n\n- **Status:** 🟢 accepted\n",
    );
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(1), "{}", stderr(&output));
    assert!(
        stdout(&output).starts_with("docs/adr: traceability/coverage: ADR-0001 is accepted"),
        "{}",
        stdout(&output)
    );
    assert_eq!(
        stderr(&output),
        "nuc check: 1 finding(s); 2 criteria without a test: G-1 Q-1.\n"
    );
}
