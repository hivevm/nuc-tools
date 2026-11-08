//! `nuc check --format`, end to end on fixture repositories.
//! Verifies: ADR-0027 ADR-draft-output-formats-with-a-concise-short-description G-7
//!
//! The JSON document and the SARIF log carry the findings of the text output, field for field and
//! in order; the exit code and the standard error do not depend on the format; a run that cannot
//! decide writes nothing on the standard output; `--format text` and no option write the lines of
//! ADR-0020.

mod support;

use std::process::Output;

use support::{CONFIG, Fixture, README, id, mark, stderr, stdout};

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
    for format in ["json", "sarif"] {
        let output = fixture.nuc(&["check", "--format", format]);
        assert_eq!(code(&output), Some(2), "{format}");
        assert_eq!(stdout(&output), "", "{format}");
        assert!(
            stderr(&output).contains("nuc.toml:1: unreadable"),
            "{}",
            stderr(&output)
        );
        assert_eq!(stderr(&output), stderr(&text), "{format}");
    }
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

/// The rules the fixtures with findings turn on, in the order `nuc check` runs them.
const RULES: [&str; 3] = [
    "docs/links",
    "docs/template-release",
    "traceability/citations",
];

/// `text` as the tests expect it in a JSON string: the characters the fixtures' messages hold
/// that RFC 8259 escapes.
fn escaped(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\t', "\\t")
}

/// A line of the text output split into its file, line, rule ID, and message, at the first
/// `: <rule ID>: ` of a rule the fixture turns on.
fn split(line: &str) -> (String, Option<usize>, &'static str, String) {
    let (at, rule) = RULES
        .iter()
        .filter_map(|rule| line.find(&format!(": {rule}: ")).map(|at| (at, *rule)))
        .min()
        .expect("a finding names a rule the fixture turns on");
    let place = &line[..at];
    let message = line[at + rule.len() + 4..].to_owned();
    match place.rsplit_once(':') {
        Some((file, number))
            if !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()) =>
        {
            (file.to_owned(), number.parse().ok(), rule, message)
        }
        _ => (place.to_owned(), None, rule, message),
    }
}

/// A project with two awkward paths and the URIs SARIF names them by: a Markdown file with a
/// space, a colon where the platform allows one, `#`, and `%`, and a broken link in it; and a file
/// with a letter beyond ASCII and a space, whose marker cites an ADR that does not exist. The link
/// rule skips a path git quotes, as the bash does; the citations rule lists the files unquoted.
fn with_awkward_paths(name: &str) -> (Fixture, [(&'static str, &'static str); 2]) {
    let (markdown, markdown_uri) = if cfg!(windows) {
        ("docs/a b#%.md", "docs/a%20b%23%25.md")
    } else {
        ("docs/a b:#%.md", "docs/a%20b%3A%23%25.md")
    };
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/links\", \"docs/template-release\", \
                  \"traceability/citations\"]\n";
    let fixture = Fixture::project(name, config, README);
    fixture
        .write(markdown, "[x](missing.md)\n")
        .write("tests/é x.rs", &format!("// {}", mark(&id("9999"))))
        .commit("Awkward paths");
    (
        fixture,
        [
            (markdown, markdown_uri),
            ("tests/é x.rs", "tests/%C3%A9%20x.rs"),
        ],
    )
}

/// The SARIF rule object of `rule` with its reason, as `nuc explain` gives it: the first sentence
/// as its short description, up to the period of the first `. ` a capital letter follows, and the
/// whole reason as its full description and help.
fn sarif_rule(fixture: &Fixture, rule: &str) -> String {
    let out = stdout(&fixture.nuc(&["explain", rule]));
    let whole = out
        .trim_end()
        .strip_prefix(&format!("{rule}: "))
        .expect("explain names the rule");
    let end = whole
        .char_indices()
        .find(|&(at, _)| {
            whole[at..].starts_with(". ")
                && whole[at + 2..]
                    .chars()
                    .next()
                    .is_some_and(char::is_uppercase)
        })
        .map_or(whole.len(), |(at, _)| at + 1);
    let (short, reason) = (escaped(&whole[..end]), escaped(whole));
    assert!(
        short.len() < reason.len(),
        "{rule}: its reason has one sentence"
    );
    format!(
        "{{\"id\":\"{rule}\",\"shortDescription\":{{\"text\":\"{short}\"}},\
         \"fullDescription\":{{\"text\":\"{reason}\"}},\"help\":{{\"text\":\"{reason}\"}}}}"
    )
}

#[test]
fn each_format_carries_the_findings_of_the_text_output_field_for_field_and_in_order() {
    let (awkward, uris) = with_awkward_paths("formats-g7-paths");
    let cases = [
        (
            with_findings("formats-g7-findings"),
            &RULES[..2],
            &[][..],
            1,
        ),
        (awkward, &RULES[..], &uris[..], 1),
        (
            Fixture::project("formats-g7-clean", CONFIG, README),
            &["docs/template-release"][..],
            &[][..],
            0,
        ),
    ];
    for (fixture, turned_on, uris, exit) in &cases {
        let text = fixture.nuc(&["check"]);
        assert_eq!(code(&text), Some(*exit), "{}", stderr(&text));
        let findings: Vec<_> = stdout(&text).lines().map(split).collect();
        assert_eq!(findings.is_empty(), *exit == 0, "{}", stdout(&text));
        for (path, _) in *uris {
            assert!(
                findings.iter().any(|(file, ..)| file == path),
                "no finding names {path}: {}",
                stdout(&text)
            );
        }

        let json = fixture.nuc(&["check", "--format", "json"]);
        let objects: Vec<String> = findings
            .iter()
            .map(|(file, line, rule, message)| {
                format!(
                    "{{\"rule\":\"{rule}\",\"file\":\"{}\",\"line\":{},\"message\":\"{}\"}}",
                    escaped(file),
                    line.map_or("null".to_owned(), |n| n.to_string()),
                    escaped(message)
                )
            })
            .collect();
        assert_eq!(
            stdout(&json),
            format!(
                "{{\"format\":\"nuc-findings\",\"version\":1,\"findings\":[{}],\"notes\":[]}}\n",
                objects.join(",")
            )
        );

        let sarif = fixture.nuc(&["check", "--format", "sarif"]);
        let rules: Vec<String> = turned_on.iter().map(|r| sarif_rule(fixture, r)).collect();
        let results: Vec<String> = findings
            .iter()
            .map(|(file, line, rule, message)| {
                let index = turned_on.iter().position(|r| r == rule).expect("a rule turned on");
                let uri = uris
                    .iter()
                    .find(|(path, _)| path == file)
                    .map_or(file.as_str(), |(_, uri)| uri);
                format!(
                    "{{\"ruleId\":\"{rule}\",\"ruleIndex\":{index},\"level\":\"error\",\
                     \"message\":{{\"text\":\"{}\"}},\"locations\":[{{\"physicalLocation\":\
                     {{\"artifactLocation\":{{\"uri\":\"{uri}\"}},\"region\":{{\"startLine\":{}}}}}}}]}}",
                    escaped(message),
                    line.unwrap_or(1)
                )
            })
            .collect();
        assert_eq!(
            stdout(&sarif),
            format!(
                "{{\"$schema\":\"https://json.schemastore.org/sarif-2.1.0.json\",\"version\":\"2.1.0\",\
                 \"runs\":[{{\"tool\":{{\"driver\":{{\"name\":\"nuc\",\"semanticVersion\":\"{}\",\
                 \"rules\":[{}]}}}},\"results\":[{}]}}]}}\n",
                env!("CARGO_PKG_VERSION"),
                rules.join(","),
                results.join(",")
            )
        );

        for output in [&json, &sarif] {
            assert_eq!(code(output), code(&text));
            assert_eq!(stderr(output), stderr(&text));
        }
    }
}

#[test]
fn the_notes_reach_the_json_in_the_order_of_the_summary_and_stay_out_of_the_sarif() {
    let config = "minimum = \"0.0.0\"\nrules = [\"traceability/coverage\"]\n";
    let fixture = Fixture::project("formats-notes", config, README);
    fixture
        .write(
            "docs/SPECIFICATION.md",
            "## Goals / Success Criteria\n\n- **G-1** — a.\n- **G-2** — b.\n\n## Quality Goals\n\n\
             - **Q-1** — c.\n",
        )
        .write("tests/t.txt", &mark("G-2"))
        .write(
            "docs/adr/0001-decision.md",
            &format!("# {}: decision\n\n- **Status:** 🟢 accepted\n", id("0001")),
        )
        .commit("A criterion and an accepted decision without a test");
    let text = fixture.nuc(&["check"]);
    assert_eq!(code(&text), Some(1), "{}", stderr(&text));
    let note = "2 criteria without a test: G-1 Q-1";
    assert_eq!(stderr(&text), format!("nuc check: 1 finding(s); {note}.\n"));

    let json = fixture.nuc(&["check", "--format", "json"]);
    assert!(
        stdout(&json).ends_with(&format!("],\"notes\":[\"{note}\"]}}\n")),
        "{}",
        stdout(&json)
    );
    let sarif = fixture.nuc(&["check", "--format", "sarif"]);
    assert!(
        stdout(&sarif).contains("\"results\":[{"),
        "{}",
        stdout(&sarif)
    );
    assert!(
        !stdout(&sarif).contains("criteria without a test"),
        "{}",
        stdout(&sarif)
    );
    for output in [&json, &sarif] {
        assert_eq!(code(output), code(&text));
        assert_eq!(stderr(output), stderr(&text));
    }
}
