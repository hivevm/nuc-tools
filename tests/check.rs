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

#[cfg(unix)]
#[test]
fn a_link_path_that_is_no_utf8_is_not_decided() {
    use std::os::unix::ffi::OsStrExt;
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/links\"]\n";
    let fixture = Fixture::project("links-bytes", config, README);
    let name = std::ffi::OsStr::from_bytes(b"caf\xe9.txt");
    std::fs::write(fixture.root().join(name), "a\n").expect("written");
    std::fs::write(fixture.root().join("x.md"), b"[c](caf\xe9.txt)\n").expect("written");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("x.md:1: a link names a path that is no UTF-8"),
        "{}",
        stderr(&output)
    );
}
