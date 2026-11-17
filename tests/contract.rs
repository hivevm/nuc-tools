//! The command-line contract of `nuc`, clause by clause, on fixture repositories.
//! Verifies: API-command-line-with-measure

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
        rest.ends_with(" [docs/template-release:version]"),
        "the line ends in its key: {rest}"
    );
    assert!(
        stderr(&output).contains("1 finding(s)"),
        "{}",
        stderr(&output)
    );
}

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
    assert!(
        stdout(&output).ends_with(" [docs/template-release:line]\n"),
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

/// Only `nuc hook` answers 3 or 4, where a shim runs a check; no other command exits 3 or 4, on a
/// clean tree, a finding, a refusal, an unknown command, or an explanation.
#[test]
fn no_command_but_hook_exits_three_or_four() {
    let clean = Fixture::project("contract-no-three-clean", CONFIG, README);
    let finding = Fixture::project("contract-no-three-finding", CONFIG, "# X\n");
    let refusal = Fixture::project("contract-no-three-refusal", "nonsense\n", README);
    let runs: [(&Fixture, &[&str]); 11] = [
        (&clean, &["measure", "--transcripts", "no-such-directory"]),
        (&clean, &["measure", "--no-such-option", "x"]),
        (&clean, &["check"]),
        (&finding, &["check"]),
        (&refusal, &["check"]),
        (&clean, &["check", "--format", "json"]),
        (&finding, &["check", "--format", "sarif"]),
        (&clean, &["explain", "docs/links"]),
        (&clean, &["explain", "docs/no-such-rule"]),
        (&clean, &["no-such-command"]),
        (&clean, &[]),
    ];
    for (fixture, args) in runs {
        let exit = code(&fixture.nuc(args));
        assert!(
            exit != Some(3) && exit != Some(4),
            "nuc {args:?} exits {exit:?}"
        );
    }
}

#[test]
fn a_command_a_newer_nuc_may_know_refuses_with_the_install_hint() {
    let fixture = Fixture::project("contract-command", CONFIG, README);
    for (args, why) in [
        (&["hook", "post-checkout"][..], "no hook 'post-checkout'"),
        (&["check", "--format", "yaml"], "usage: nuc check"),
    ] {
        let output = fixture.nuc(args);
        assert_eq!(code(&output), Some(2), "{args:?}");
        assert!(stderr(&output).contains(why), "{args:?}");
        assert!(
            stderr(&output).contains("install or update nuc"),
            "{args:?}: {}",
            stderr(&output)
        );
    }
}

/// `nuc measure` writes its measures on the standard output, ending in `tokens <n>`, and exits 0;
/// where it cannot read a transcript it exits 2, with nothing on the standard output.
#[test]
fn measure_ends_in_its_tokens_and_exits_zero_or_two() {
    let fixture = Fixture::project("contract-measure", CONFIG, README);
    let dir = fixture
        .root()
        .join(".git")
        .join("contract-measure-transcripts");
    std::fs::create_dir_all(&dir).expect("the directory is made");
    let line = r#"{"type":"assistant","requestId":"r","gitBranch":"main","message":{"usage":{"input_tokens":1,"cache_read_input_tokens":2,"cache_creation_input_tokens":3,"output_tokens":4}}}"#;
    std::fs::write(dir.join("s.jsonl"), format!("{line}\n")).expect("written");
    let path = dir.to_str().expect("UTF-8");
    let output = fixture.nuc(&["measure", "--transcripts", path]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).ends_with("tokens 10\n"),
        "{}",
        stdout(&output)
    );

    let output = fixture.nuc(&["measure", "--transcripts", path, "--breakdown"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.starts_with("session s: input 1 cache read 2 cache written 3 output 4\n"),
        "{text}"
    );
    assert!(
        text.ends_with("tokens 10\n"),
        "the breakdown stands before the line: {text}"
    );
    let output = fixture.nuc(&["measure", "--transcripts", path, "--finding"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.starts_with("## Template release\n"), "{text}");
    assert!(
        text.ends_with("tokens 10\n"),
        "the draft stands before the line: {text}"
    );

    std::fs::write(dir.join("s.jsonl"), "{\n").expect("written");
    let output = fixture.nuc(&["measure", "--transcripts", path]);
    assert_eq!(code(&output), Some(2));
    assert_eq!(stdout(&output), "");
}

/// Every rule ID `nuc` has had, kept beside this test.
const RULE_IDS: &str = include_str!("contract.rule-ids");

/// Verifies: G-6
#[test]
fn every_rule_keeps_its_id_and_is_explained() {
    let listed: Vec<&str> = RULE_IDS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    let known: Vec<&str> = nuc::rules::ALL
        .iter()
        .map(|rule| rule.id)
        .chain(nuc::hook::ALL.iter().map(|rule| rule.id))
        .collect();
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

/// Every key `nuc` has printed, kept beside this test.
const RULE_KEYS: &str = include_str!("contract.rule-keys");

/// Verifies: API-command-line-with-measure, G-6
#[test]
fn every_key_stays_once_listed_and_each_reason_is_one_lower_case_word() {
    let listed: Vec<&str> = RULE_KEYS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    let declared: Vec<String> = nuc::rules::ALL
        .iter()
        .flat_map(|rule| rule.reasons.iter().map(move |reason| (rule.id, *reason)))
        .chain(
            nuc::hook::ALL
                .iter()
                .flat_map(|rule| rule.reasons.iter().map(move |reason| (rule.id, *reason))),
        )
        .map(|(id, reason)| {
            assert!(
                !reason.is_empty() && reason.bytes().all(|b| b.is_ascii_lowercase()),
                "{id}: the reason '{reason}' is not one word of lower-case letters"
            );
            format!("{id}:{reason}")
        })
        .collect();
    for key in &listed {
        assert!(
            declared.iter().any(|d| d == key),
            "the key '{key}' is gone: a key is stable once listed, since readers branch on it"
        );
    }
    for key in &declared {
        assert!(
            listed.contains(&key.as_str()),
            "the key '{key}' is not in contract.rule-keys: list it, and it stays from then on"
        );
    }
    let mut unique = declared.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        declared.len(),
        "a rule declares a reason twice"
    );
}

/// Verifies: ARCH-rules-outcome-hooks-answer, G-6
#[test]
fn no_rule_id_is_held_twice_by_the_rules_and_the_hooks() {
    let mut seen = std::collections::BTreeSet::new();
    for id in nuc::rules::ALL
        .iter()
        .map(|rule| rule.id)
        .chain(nuc::hook::ALL.iter().map(|rule| rule.id))
    {
        assert!(seen.insert(id), "the rule ID '{id}' is held twice");
    }
}

/// Verifies: ARCH-rules-need-only-git
#[test]
fn a_rule_left_to_the_bash_is_refused_as_one_this_nuc_does_not_know() {
    for rule in ["shell/bash-floor", "shell/root"] {
        let config = format!("minimum = \"0.0.0\"\nrules = [\"{rule}\"]\n");
        let fixture =
            Fixture::project(&format!("left-{}", rule.replace('/', "-")), &config, README);
        let output = fixture.nuc(&["check"]);
        assert_eq!(code(&output), Some(2), "{rule}: {}", stdout(&output));
        assert!(
            stderr(&output).contains(&format!(
                "turns on the rule '{rule}', which this nuc does not know"
            )),
            "{rule}: {}",
            stderr(&output)
        );
    }
}
