//! `nuc check --format`, end to end on fixture repositories. Verifies: ADR-draft-output-formats
//!
//! The JSON document carries the findings of the text output, field for field and in order; the
//! exit code and the standard error do not depend on the format; a run that cannot decide writes
//! nothing on the standard output; `--format text` and no option write the lines of ADR-0020.

mod support;

use std::process::Output;

use support::{CONFIG, Fixture, README, stderr, stdout};

fn code(output: &Output) -> Option<i32> {
    output.status.code()
}

/// A project with three findings: two on a line, whose messages carry a non-ASCII letter, a
/// quote, a backslash, and a tab from a link's fragment, and one without a line.
fn with_findings(name: &str) -> Fixture {
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/links\", \"docs/template-release\"]\n";
    let fixture = Fixture::project(name, config, "# X\n");
    fixture
        .write("x.md", "# T\n")
        .write("doc.md", "[a](x.md#é\"\\b\tc) [b](x.md#q\"r)\n")
        .commit("Links without their anchors");
    fixture
}

#[test]
fn json_carries_the_findings_of_the_text_in_order_with_the_same_exit_and_summary() {
    let fixture = with_findings("formats-json");
    let text = fixture.nuc(&["check"]);
    let json = fixture.nuc(&["check", "--format", "json"]);
    assert_eq!(code(&text), Some(1), "{}", stderr(&text));
    assert_eq!(
        stdout(&text),
        "doc.md:1: docs/links: link 'x.md#é\"\\b\tc' has no matching anchor '#é\"\\b\tc' in x.md\n\
         doc.md:1: docs/links: link 'x.md#q\"r' has no matching anchor '#q\"r' in x.md\n\
         README.md: docs/template-release: no '**Template release:**' line — the Template section \
         names the release of the template this repository carries, or 'unreleased' (ADR-0005)\n"
    );
    assert_eq!(code(&json), Some(1), "{}", stderr(&json));
    assert_eq!(
        stdout(&json),
        "{\"format\":\"nuc-findings\",\"version\":1,\"findings\":[\
         {\"rule\":\"docs/links\",\"file\":\"doc.md\",\"line\":1,\
         \"message\":\"link 'x.md#é\\\"\\\\b\\tc' has no matching anchor '#é\\\"\\\\b\\tc' in x.md\"},\
         {\"rule\":\"docs/links\",\"file\":\"doc.md\",\"line\":1,\
         \"message\":\"link 'x.md#q\\\"r' has no matching anchor '#q\\\"r' in x.md\"},\
         {\"rule\":\"docs/template-release\",\"file\":\"README.md\",\"line\":null,\
         \"message\":\"no '**Template release:**' line — the Template section names the release \
         of the template this repository carries, or 'unreleased' (ADR-0005)\"}],\
         \"notes\":[]}\n"
    );
    assert_eq!(stderr(&json), stderr(&text));
}

#[test]
fn json_without_findings_holds_an_empty_list_and_passes() {
    let fixture = Fixture::project("formats-json-clean", CONFIG, README);
    let output = fixture.nuc(&["check", "--format", "json"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "{\"format\":\"nuc-findings\",\"version\":1,\"findings\":[],\"notes\":[]}\n"
    );
    assert!(stderr(&output).contains("nuc check passed (1 rules)"));
}

#[test]
fn a_run_that_cannot_decide_writes_no_document() {
    let fixture = Fixture::project("formats-refusal", "nonsense\n", README);
    let text = fixture.nuc(&["check"]);
    let json = fixture.nuc(&["check", "--format", "json"]);
    assert_eq!(code(&json), Some(2));
    assert_eq!(stdout(&json), "");
    assert!(
        stderr(&json).contains("nuc.toml:1: unreadable"),
        "{}",
        stderr(&json)
    );
    assert_eq!(stderr(&json), stderr(&text));
}

#[test]
fn text_is_the_default_and_the_option_stands_before_or_after_the_repository() {
    let fixture = with_findings("formats-text");
    let repo = fixture.root().to_str().expect("a UTF-8 path");
    let plain = fixture.nuc(&["check"]);
    for args in [
        &["check", "--format", "text"][..],
        &["check", "--repo", repo, "--format", "text"],
        &["check", "--format", "text", "--repo", repo],
    ] {
        let output = fixture.nuc(args);
        assert_eq!(code(&output), code(&plain), "{args:?}");
        assert_eq!(stdout(&output), stdout(&plain), "{args:?}");
    }
    let json = fixture.nuc(&["check", "--format", "json"]);
    let after = fixture.nuc(&["check", "--repo", repo, "--format", "json"]);
    let before = fixture.nuc(&["check", "--format", "json", "--repo", repo]);
    assert_eq!(stdout(&after), stdout(&json));
    assert_eq!(stdout(&before), stdout(&json));
}

#[test]
fn an_unknown_or_repeated_option_is_a_usage_error() {
    let fixture = Fixture::project("formats-usage", CONFIG, README);
    for args in [
        &["check", "--format", "yaml"][..],
        &["check", "--format", "json", "--format", "json"],
        &["check", "--format"],
        &["check", "--repo", ".", "--repo", "."],
    ] {
        let output = fixture.nuc(args);
        assert_eq!(code(&output), Some(2), "{args:?}");
        assert_eq!(stdout(&output), "", "{args:?}");
        assert!(stderr(&output).contains("usage: nuc check"), "{args:?}");
    }
}
