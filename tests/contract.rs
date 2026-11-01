//! The command-line contract of `nuc`, clause by clause, on fixture repositories.
//! Verifies: ADR-0019

mod support;

use support::{CONFIG, Fixture, README, stderr, stdout};

fn code(output: &std::process::Output) -> Option<i32> {
    output.status.code()
}

#[test]
fn a_newer_nuc_toml_with_an_unknown_key_refuses_as_too_old() {
    let config = "minimum = \"99.0.0\"\nrules = [\"docs/template-release\"]\nlevel = \"strict\"\n";
    let fixture = Fixture::project("contract-newer", config, README);
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(2));
    assert!(
        stderr(&output).contains("requires nuc 99.0.0 or later"),
        "{}",
        stderr(&output)
    );
    assert!(
        !stderr(&output).contains("unreadable"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn what_this_nuc_cannot_read_refuses_with_the_install_hint() {
    for (name, config) in [
        (
            "contract-key",
            "minimum = \"0.0.0\"\nrules = []\nlevel = \"strict\"\n",
        ),
        (
            "contract-twice",
            "minimum = \"0.0.0\"\nrules = []\nrules = []\n",
        ),
        ("contract-table", "minimum = \"0.0.0\"\n[rules]\n"),
        ("contract-number", "minimum = 1\nrules = []\n"),
    ] {
        let fixture = Fixture::project(name, config, README);
        let output = fixture.nuc(&["check"]);
        assert_eq!(code(&output), Some(2), "{name}");
        assert!(
            stderr(&output).contains("unreadable"),
            "{name}: {}",
            stderr(&output)
        );
        assert!(stderr(&output).contains("install or update nuc"), "{name}");
        assert_eq!(
            stdout(&output),
            "",
            "{name}: a refusal writes nothing on the standard output"
        );
    }
}

/// Verifies: G-6
#[test]
fn a_finding_is_one_line_on_the_standard_output_and_the_summary_is_on_the_standard_error() {
    let fixture = Fixture::project(
        "contract-line",
        CONFIG,
        "# X\n\n- **Template release:** latest\n",
    );
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(1));
    let out = stdout(&output);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 1, "{out}");
    let (location, rest) = lines[0].split_once(": ").expect("<file>[:<line>]: …");
    assert_eq!(location, "README.md:3");
    assert!(rest.starts_with("docs/template-release: "), "{rest}");
    assert!(
        stderr(&output).contains("1 finding(s)"),
        "{}",
        stderr(&output)
    );
}

/// Verifies: G-6
#[test]
fn a_finding_without_a_line_omits_it() {
    let fixture = Fixture::project("contract-no-line", CONFIG, "# X\n");
    let output = fixture.nuc(&["check"]);
    assert_eq!(code(&output), Some(1));
    assert!(
        stdout(&output).starts_with("README.md: docs/template-release: "),
        "{}",
        stdout(&output)
    );
}

/// Verifies: Q-1
#[test]
fn the_exit_codes_are_zero_one_and_two() {
    let clean = Fixture::project("contract-clean", CONFIG, README);
    let finding = Fixture::project("contract-finding", CONFIG, "# X\n");
    let refusal = Fixture::project("contract-refusal", "nonsense\n", README);
    assert_eq!(code(&clean.nuc(&["check"])), Some(0));
    assert_eq!(code(&finding.nuc(&["check"])), Some(1));
    assert_eq!(code(&refusal.nuc(&["check"])), Some(2));
    assert_eq!(code(&clean.nuc(&["no-such-command"])), Some(2));
}

#[test]
fn a_command_a_newer_nuc_may_know_refuses_with_the_install_hint() {
    let fixture = Fixture::project("contract-command", CONFIG, README);
    for args in [&["hook", "pre-commit"][..], &["check", "--format", "json"]] {
        let output = fixture.nuc(args);
        assert_eq!(code(&output), Some(2), "{args:?}");
        assert!(stderr(&output).contains("usage: nuc check"), "{args:?}");
        assert!(
            stderr(&output).contains("install or update nuc"),
            "{args:?}: {}",
            stderr(&output)
        );
    }
}

/// The rule IDs of every release, kept beside this test.
const RULE_IDS: &str = include_str!("contract.rule-ids");

/// Verifies: G-6
#[test]
fn every_rule_keeps_its_id_and_is_explained() {
    let listed: Vec<&str> = RULE_IDS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    let known: Vec<&str> = nuc::rules::ALL.iter().map(|rule| rule.id).collect();
    for id in &listed {
        assert!(
            known.contains(id),
            "the rule ID '{id}' is gone: an ID is stable once listed, since nuc.toml files name it"
        );
    }
    let fixture = Fixture::new("contract-explain");
    for id in &known {
        assert!(
            listed.contains(id),
            "the rule '{id}' is not in contract.rule-ids: list it, and it stays from then on"
        );
        let output = fixture.nuc(&["explain", id]);
        assert_eq!(code(&output), Some(0), "{id}");
        let out = stdout(&output);
        let reason = out.strip_prefix(&format!("{id}: ")).unwrap_or_default();
        assert!(
            !reason.trim().is_empty(),
            "nuc explain {id} gives no reason: {out}"
        );
    }
}
