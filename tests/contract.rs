//! The command-line contract of `nuc`, clause by clause, on fixture repositories, and `nuc sense`:
//! the names it knows and refuses, `nuc.toml` judged first, `--repo` in place of the bash's
//! `--root`, the release a sensor reports as, and the exits and keys of its refusals. The reports
//! themselves are compared with the bash sensors by the parity harness.
//! Verifies: API-command-line-with-sense, API-git-only-sensors-under-sense, G-9

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

/// Verifies: API-config-refusals-keyed, G-5
#[test]
fn every_refusal_of_nuc_toml_ends_in_its_key() {
    let missing = Fixture::new("config-key-missing");
    missing.write("README.md", README);
    for (fixture, reason) in [
        (missing, "missing"),
        (
            Fixture::project(
                "config-key-unreadable",
                "minimum = \"0.0.0\"\n[rules]\n",
                README,
            ),
            "unreadable",
        ),
        (
            Fixture::project(
                "config-key-old",
                "minimum = \"99.0.0\"\nrules = []\n",
                README,
            ),
            "old",
        ),
        (
            Fixture::project(
                "config-key-unknown",
                "minimum = \"0.0.0\"\nrules = [\"docs/no-such-rule\"]\n",
                README,
            ),
            "unknown",
        ),
    ] {
        let output = fixture.nuc(&["check"]);
        assert_eq!(code(&output), Some(2), "{reason}");
        let err = stderr(&output);
        let line = err
            .lines()
            .find(|line| line.contains("nuc.toml"))
            .unwrap_or("");
        assert!(
            line.ends_with(&format!(" [config/toml:{reason}]"))
                && line.matches("[config/toml:").count() == 1,
            "{reason}: the refusal ends in its key, once: {err}"
        );
        assert_eq!(
            stdout(&output),
            "",
            "{reason}: a refusal writes nothing on the standard output"
        );
    }
}

/// Verifies: API-config-refusals-keyed
#[test]
fn a_nuc_toml_that_cannot_be_read_at_all_refuses_as_unreadable_in_its_key() {
    let text = Fixture::project("config-key-bytes", CONFIG, README);
    std::fs::write(text.root().join("nuc.toml"), [0xff, 0xfe, b'\n']).expect("bytes are written");
    let dir = Fixture::project("config-key-dir", CONFIG, README);
    std::fs::remove_file(dir.root().join("nuc.toml")).expect("the file is removed");
    std::fs::create_dir(dir.root().join("nuc.toml")).expect("a directory takes its place");
    for (name, fixture) in [("no UTF-8", text), ("a directory", dir)] {
        let output = fixture.nuc(&["check"]);
        assert_eq!(code(&output), Some(2), "{name}");
        assert!(
            stderr(&output)
                .trim_end()
                .ends_with(" [config/toml:unreadable]"),
            "{name}: {}",
            stderr(&output)
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

/// The keys the shims under `shims/` write (API-hook-contract-keyed-shims):
/// `[shim/<hook>:<reason>]` for a refusal of a shim's own, its hook the shim's own name, and
/// `[hook/<rule>:red]` for a red check, its rule one of `hook::ALL`.
fn shim_keys() -> Vec<(String, String)> {
    let dir = support::sources::root().join("shims");
    let mut keys = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("shims/ is listed") {
        let path = entry.expect("an entry").path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a name")
            .to_owned();
        let text = std::fs::read_to_string(&path).expect("a shim is read");
        let shim = text.split("[shim/").skip(1).map(|part| ("shim", part));
        let hook = text.split("[hook/").skip(1).map(|part| ("hook", part));
        for (area, part) in shim.chain(hook) {
            let key = part.split(']').next().expect("a key");
            let (id, reason) = key.split_once(':').expect("a key has a reason");
            if area == "shim" {
                assert_eq!(
                    id, name,
                    "shims/{name} writes the key of another shim, shim/{id}"
                );
            } else {
                assert!(
                    nuc::hook::ALL
                        .iter()
                        .any(|rule| rule.id == format!("hook/{id}")),
                    "shims/{name} writes hook/{id}, which is no hook rule"
                );
            }
            keys.push((format!("{area}/{id}"), reason.to_owned()));
        }
    }
    keys.sort();
    keys.dedup();
    keys
}

/// Verifies: API-command-line-with-sense, API-hook-contract-keyed-shims, G-6
/// Verifies: API-config-refusals-keyed, API-git-only-sensors-under-sense
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
        .map(|(id, reason)| (id.to_owned(), reason.to_owned()))
        .chain(shim_keys())
        .chain(nuc::sense::ALL.iter().flat_map(|sensor| {
            sensor
                .reasons
                .iter()
                .map(move |reason| (format!("sense/{}", sensor.name), (*reason).to_owned()))
        }))
        .chain(std::iter::once((
            "sense/unknown".to_owned(),
            nuc::sense::UNKNOWN
                .rsplit(':')
                .next()
                .expect("a reason")
                .to_owned(),
        )))
        .chain(
            nuc::config::Refusal::REASONS
                .iter()
                .map(|reason| (nuc::config::KEY.to_owned(), (*reason).to_owned())),
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

const LINE: &str = "**Last design revision:** none yet, due after 2 changes.";

/// A project whose overview carries the line, with one commit.
fn project(name: &str, readme: &str, overview: &str) -> Fixture {
    let fixture = Fixture::project(name, CONFIG, readme);
    fixture.write("docs/ARCHITECTURE.md", &format!("# A\n\n{overview}\n"));
    fixture.commit("start");
    fixture
}

#[test]
fn the_sensors_left_to_the_bash_and_an_unknown_name_exit_two_with_their_key() {
    let fixture = project("sense-unknown", README, LINE);
    for name in [
        "red-commits",
        "release-notes",
        "repo-settings",
        "work-measures",
        "no-such-sensor",
    ] {
        let output = fixture.nuc(&["sense", name]);
        assert_eq!(code(&output), Some(2), "{name}");
        assert_eq!(stdout(&output), "", "{name}");
        assert!(
            stderr(&output).trim_end().ends_with("[sense/unknown:name]"),
            "{name}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn a_nuc_toml_above_this_nuc_refuses_before_the_name_is_judged() {
    let fixture = Fixture::project("sense-newer", "minimum = \"99.0.0\"\nrules = []\n", README);
    for name in ["revision-due", "no-such-sensor"] {
        let output = fixture.nuc(&["sense", name]);
        assert_eq!(code(&output), Some(2), "{name}");
        assert!(
            stderr(&output).contains("requires nuc 99.0.0 or later"),
            "{name}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn repo_names_the_repository_and_root_is_refused() {
    let fixture = project("sense-repo", README, LINE);
    let root = fixture.root().to_str().expect("UTF-8");
    let output = fixture.nuc_in("docs", &["sense", "revision-due", "--repo", root]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "Design revision sensor: 1 of 2 changes outside docs/ since the first commit — not yet due.\n"
    );
    let output = fixture.nuc(&["sense", "revision-due", "--root", root]);
    assert_eq!(code(&output), Some(2));
    assert!(stderr(&output).contains("usage:"), "{}", stderr(&output));
}

#[test]
fn a_tree_whose_release_has_no_such_sensor_is_not_decided() {
    let fixture = project(
        "sense-release",
        "# Fixture\n\n- **Template release:** v0.0.9\n",
        LINE,
    );
    let output = fixture.nuc(&["sense", "revision-due"]);
    assert_eq!(code(&output), Some(2));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output)
            .trim_end()
            .ends_with("[sense/revision-due:release]"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_sensor_that_cannot_read_its_input_exits_two_with_its_key() {
    for (name, overview, reason) in [
        ("sense-no-line", "No line.", "line"),
        (
            "sense-unreadable",
            "**Last design revision:** soon, due after some changes.",
            "unreadable",
        ),
    ] {
        let fixture = project(name, README, overview);
        let output = fixture.nuc(&["sense", "revision-due"]);
        assert_eq!(code(&output), Some(2), "{name}");
        assert_eq!(stdout(&output), "", "{name}");
        assert!(
            stderr(&output)
                .trim_end()
                .ends_with(&format!("[sense/revision-due:{reason}]")),
            "{name}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn a_due_revision_names_the_entries_its_changes_touched() {
    let fixture = project(
        "sense-due",
        "# Fixture\n\n- **Template release:** v0.12.0\n",
        LINE,
    );
    fixture.write("src/a.rs", "a\n");
    fixture.commit("a");
    fixture.write("src/b.rs", "b\n");
    fixture.commit("b");
    let output = fixture.nuc(&["sense", "revision-due"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(
        text.starts_with(
            "Design revision sensor: 3 changes outside docs/ since the first commit, due after 2 — DUE:"
        ),
        "{text}"
    );
    // The ties stand in the order `sort -rn` gives them in the C locale: by the line, descending.
    assert!(
        text.ends_with(
            "(NUC-design-is-revised §5).\n  - src/ (2 changes)\n  - nuc.toml (1 change)\n  - README.md (1 change)\n"
        ),
        "{text}"
    );
}

/// The diffs `git log -G` reads to find the commit that set the date run no textconv driver the
/// repository names (Q-7): the driver here would write a file. A shell script is the driver, so
/// the test runs where one can run.
#[cfg(unix)]
#[test]
fn the_anchor_runs_no_textconv_driver_of_the_repository() {
    let fixture = Fixture::project("sense-textconv", CONFIG, README);
    fixture.write(".gitattributes", "*.md diff=conv\n");
    let marker = fixture.root().join(".git").join("textconv-ran");
    let script = fixture.root().join(".git").join("conv.sh");
    std::fs::write(
        &script,
        format!("#!/bin/sh\ntouch '{}'\ncat \"$1\"\n", marker.display()),
    )
    .expect("written");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).expect("mode");
    fixture.git(&[
        "config",
        "diff.conv.textconv",
        script.to_str().expect("UTF-8"),
    ]);
    fixture.write(
        "docs/ARCHITECTURE.md",
        "# A\n\n**Last design revision:** 2026-01-01, due after 9 changes.\n",
    );
    fixture.commit("start");
    let output = fixture.nuc(&["sense", "revision-due"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(!marker.exists(), "nuc sense ran the textconv driver");
}

/// A `--require` that fails exits 1, its references on the standard error where the bash writes
/// them and nothing on the standard output; one that holds exits 0 with its line.
#[test]
fn a_failed_require_exits_one_and_one_that_holds_exits_zero() {
    let fixture = Fixture::project("sense-require", CONFIG, README);
    fixture.write(
        ".github/workflows/ci.yml",
        "jobs:\n  x:\n    steps:\n      - uses: actions/checkout@v4\n",
    );
    fixture.commit("workflow");
    let output = fixture.nuc(&["sense", "action-refs", "--require", "sha"]);
    assert_eq!(code(&output), Some(1), "{}", stderr(&output));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).contains("is on a major tag, not on a commit SHA"),
        "{}",
        stderr(&output)
    );
    fixture.write(
        ".github/dependabot.yml",
        "updates:\n  - package-ecosystem: github-actions\n",
    );
    let output = fixture.nuc(&["sense", "action-refs", "--require", "major-tag"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).ends_with("every reference is in the decided form.\n"),
        "{}",
        stdout(&output)
    );
    let output = fixture.nuc(&["sense", "action-refs", "--require", "tag"]);
    assert_eq!(code(&output), Some(2));
    assert!(
        stderr(&output)
            .trim_end()
            .ends_with("[sense/action-refs:require]"),
        "{}",
        stderr(&output)
    );
}

/// A sensor's argument that reads as an option is refused, as the bash of v0.10.0 on refuses it.
#[test]
fn a_sensor_argument_that_reads_as_an_option_is_refused() {
    let fixture = Fixture::project(
        "sense-option",
        CONFIG,
        "# Fixture\n\n- **Template release:** v0.12.0\n",
    );
    for args in [
        &["sense", "tests-kept", "", "-x"][..],
        &["sense", "tests-kept", "a", "b", "c"],
        &["sense", "backlog", "x"],
    ] {
        let output = fixture.nuc(args);
        assert_eq!(code(&output), Some(2), "{args:?}");
        assert!(stderr(&output).contains("usage:"), "{args:?}");
    }
}

/// A script that writes the marker where it runs, for the drivers and commands a repository's
/// configuration names.
#[cfg(unix)]
fn bait(fixture: &Fixture, name: &str, marker: &std::path::Path) -> String {
    use std::os::unix::fs::PermissionsExt;
    let script = fixture.root().join(".git").join(name);
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\ntouch '{}'\ncat \"$1\" 2>/dev/null\nexit 0\n",
            marker.display()
        ),
    )
    .expect("written");
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).expect("mode");
    script.to_str().expect("UTF-8").to_owned()
}

/// The diffs of `tests-kept` run no textconv driver and no external diff the repository names
/// (Q-7).
#[cfg(unix)]
#[test]
fn tests_kept_runs_no_driver_of_the_repository() {
    let fixture = Fixture::project("sense-tests-kept-bait", CONFIG, README);
    fixture.write(".gitattributes", "*.rs diff=bait\n");
    fixture.write("tests/a_test.rs", "fn a() {}\n");
    fixture.commit("tests");
    fixture.git(&["switch", "-q", "-c", "feature"]);
    fixture.write("tests/a_test.rs", "#[ignore]\nfn a() {}\n");
    fixture.commit("ignore");
    let marker = fixture.root().join(".git").join("driver-ran");
    let textconv = bait(&fixture, "textconv.sh", &marker);
    let command = bait(&fixture, "command.sh", &marker);
    fixture.git(&["config", "diff.bait.textconv", &textconv]);
    fixture.git(&["config", "diff.bait.command", &command]);
    fixture.git(&["config", "diff.external", &command]);
    let output = fixture.nuc(&["sense", "tests-kept"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("skip or focus marker added: tests/a_test.rs:1: #[ignore]"),
        "{}",
        stdout(&output)
    );
    assert!(!marker.exists(), "nuc sense tests-kept ran a driver");

    // The diff of a change that moves a submodule starts no diff in the submodule.
    let (main, marker) = moved_submodule("sense-tests-kept-bait", "`sm`");
    let output = main.nuc(&["sense", "tests-kept", "main", "feature"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        !marker.exists(),
        "a diff in the submodule ran its external diff"
    );
}

/// The file's modification time a minute ahead, its content as it was: git must read it again to
/// tell whether it changed, as it does for a file the working tree touched.
#[cfg(unix)]
fn stat_changed(path: &std::path::Path) {
    let later = std::time::SystemTime::now() + std::time::Duration::from_secs(60);
    std::fs::File::options()
        .write(true)
        .open(path)
        .and_then(|f| f.set_modified(later))
        .expect("the mtime is set");
}

/// The diffs of `plain-rules` run no textconv driver and no external diff the repository names
/// (Q-7).
#[cfg(unix)]
#[test]
fn plain_rules_runs_no_driver_of_the_repository() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let fixture = Fixture::project("sense-plain-rules-bait", CONFIG, readme);
    fixture.write(".gitattributes", "*.md diff=bait\n");
    fixture.write("AGENTS.md", "# Agent Guide\n\n- The human decides.\n");
    fixture.commit("rules");
    fixture.git(&["switch", "-q", "-c", "feature"]);
    fixture.write(
        "AGENTS.md",
        "# Agent Guide\n\n- The human decides.\n- The session should read it.\n",
    );
    fixture.commit("modal");
    let marker = fixture.root().join(".git").join("driver-ran");
    let textconv = bait(&fixture, "textconv.sh", &marker);
    let command = bait(&fixture, "command.sh", &marker);
    fixture.git(&["config", "diff.bait.textconv", &textconv]);
    fixture.git(&["config", "diff.bait.command", &command]);
    fixture.git(&["config", "diff.external", &command]);
    let output = fixture.nuc(&["sense", "plain-rules", "main", "feature"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output)
            .contains("  - AGENTS.md:4: the soft modal verb \"should\" [rules/soft-modal:verb]\n"),
        "{}",
        stdout(&output)
    );
    assert!(!marker.exists(), "nuc sense plain-rules ran a driver");
}

/// A change that ends in the working tree runs no filter the repository names (Q-7): a file of
/// rules under a filter is not decided, since the bash's git reads it through the filter; and the
/// diffs leave out a submodule, whose own filter git would run to tell whether it is dirty.
#[cfg(unix)]
#[test]
fn plain_rules_in_the_working_tree_runs_no_filter() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let fixture = Fixture::project("sense-plain-rules-filter", CONFIG, readme);
    fixture.write(".gitattributes", "*.md filter=bait\n");
    fixture.write("AGENTS.md", "# Agent Guide\n\n- The human decides.\n");
    fixture.commit("rules");
    let marker = fixture.root().join(".git").join("filter-ran");
    let clean = bait(&fixture, "clean.sh", &marker);
    fixture.git(&["config", "filter.bait.clean", &clean]);
    fixture.write(
        "AGENTS.md",
        "# Agent Guide\n\n- The human decides.\n- The session should read it.\n",
    );
    let output = fixture.nuc(&["sense", "plain-rules", "HEAD"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output)
            .trim_end()
            .ends_with("[sense/plain-rules:filter]"),
        "{}",
        stderr(&output)
    );
    assert!(
        !marker.exists(),
        "nuc sense plain-rules ran the clean filter"
    );

    // A submodule whose own configuration names a filter for a file whose stat changed.
    let sub = Fixture::project("sense-plain-rules-sub", CONFIG, readme);
    sub.write(".gitattributes", "*.txt filter=bait\n");
    sub.write("f.txt", "a\n");
    sub.commit("sub");
    let main = Fixture::project("sense-plain-rules-super", CONFIG, readme);
    let url = sub.root().to_str().expect("UTF-8");
    let added = main.git(&[
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "-q",
        "add",
        url,
        "sm",
    ]);
    assert!(added.status.success(), "{}", stderr(&added));
    main.write("AGENTS.md", "# Agent Guide\n\n- The human decides.\n");
    main.commit("submodule");
    let marker = main.root().join(".git").join("sub-filter-ran");
    let clean = bait(&main, "clean.sh", &marker);
    support::git_in(
        &main.root().join("sm"),
        &["config", "filter.bait.clean", &clean],
    );
    stat_changed(&main.root().join("sm/f.txt"));
    main.write(
        "AGENTS.md",
        "# Agent Guide\n\n- The human decides.\n- It might run.\n",
    );
    let output = main.nuc(&["sense", "plain-rules", "HEAD"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("AGENTS.md:4: the soft modal verb \"might\""),
        "{}",
        stdout(&output)
    );
    assert!(
        !marker.exists(),
        "nuc sense plain-rules ran a submodule's filter"
    );
}

/// A filter on a file outside the rules, as Git LFS names one, whose file's stat changed, runs
/// neither its clean nor its process command, even where it is required, and the sensor reports on
/// the rules (Q-7). The parity scenario `plain_filtered` holds the report to the bash's.
#[cfg(unix)]
#[test]
fn plain_rules_neutralizes_a_filter_outside_the_rules() {
    // A filter on a file outside the rules, as Git LFS names one, whose file's stat changed.
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let other = Fixture::project("sense-plain-rules-lfs", CONFIG, readme);
    other.write(".gitattributes", "*.png filter=bait\n");
    other.write("a.png", "image\n");
    other.write("AGENTS.md", "# Agent Guide\n\n- The human decides.\n");
    other.commit("rules");
    let marker = other.root().join(".git").join("lfs-filter-ran");
    let clean = bait(&other, "clean.sh", &marker);
    other.git(&["config", "filter.bait.clean", &clean]);
    other.git(&["config", "filter.bait.process", &clean]);
    other.git(&["config", "filter.bait.required", "true"]);
    stat_changed(&other.root().join("a.png"));
    other.write(
        "AGENTS.md",
        "# Agent Guide\n\n- The human decides.\n- The session should read it.\n",
    );
    let output = other.nuc(&["sense", "plain-rules", "HEAD"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).ends_with(
            "..working tree): 1 to read on the lines the change adds to the rules (AGENTS.md section 4):\n  - AGENTS.md:4: the soft modal verb \"should\" [rules/soft-modal:verb]\n"
        ),
        "{}",
        stdout(&output)
    );
    assert!(
        !marker.exists(),
        "nuc sense plain-rules ran the filter of a.png"
    );
}

/// The value of the filter attribute read as git reads it: a driver whose name holds `=`, which
/// `-c` would split there and so neutralize another key, is not decided and does not run; a bare
/// `filter` on a file of rules names no driver, and the sensor reads the file's bytes as git does.
#[cfg(unix)]
#[test]
fn plain_rules_reads_the_filter_attribute_as_git_does() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    // A driver whose name holds `=`, which `-c` would split there: not decided, and not run.
    let odd = Fixture::project("sense-plain-rules-odd-driver", CONFIG, readme);
    odd.write(".gitattributes", "*.png filter=a=b\n");
    odd.write("a.png", "image\n");
    odd.write("AGENTS.md", "# Agent Guide\n\n- The human decides.\n");
    odd.commit("rules");
    let marker = odd.root().join(".git").join("odd-filter-ran");
    let clean = bait(&odd, "clean.sh", &marker);
    odd.git(&["config", "filter.a=b.clean", &clean]);
    stat_changed(&odd.root().join("a.png"));
    let output = odd.nuc(&["sense", "plain-rules", "HEAD"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output)
            .trim_end()
            .ends_with("[sense/plain-rules:filter]"),
        "{}",
        stderr(&output)
    );
    assert!(!marker.exists(), "nuc sense plain-rules ran the driver a=b");

    // A bare `filter` on a file of rules names no driver: git reads its bytes, and so does nuc.
    let bare = Fixture::project("sense-plain-rules-bare-filter", CONFIG, readme);
    bare.write(".gitattributes", "*.md filter\n");
    bare.write("AGENTS.md", "# Agent Guide\n\n- The human decides.\n");
    bare.commit("rules");
    bare.write(
        "AGENTS.md",
        "# Agent Guide\n\n- The human decides.\n- It might run.\n",
    );
    let output = bare.nuc(&["sense", "plain-rules", "HEAD"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("AGENTS.md:4: the soft modal verb \"might\""),
        "{}",
        stdout(&output)
    );
}

/// A path the working tree adds under a filter, beside a renamed file of rules, is not decided:
/// git would find renames through its raw bytes where the bash's git reads the bytes the filter
/// cleans, and pair the file of rules otherwise (Q-1, Q-7). Here the bash's git pairs the old ADR
/// with `z.bin`, so the renamed ADR reads with every line added and `--require` fails on its
/// "should"; read raw, the ADR pairs with its old self and the gate would pass.
#[cfg(unix)]
#[test]
fn plain_rules_refuses_a_rename_a_filter_could_pair_otherwise() {
    // A name git quotes in its listing, as it quotes one with a `"`, is read unquoted.
    for (n, name) in ["z.bin", "z\"q.bin"].into_iter().enumerate() {
        let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
        let fixture = Fixture::project(
            &format!("sense-plain-rules-rename-filter-{n}"),
            CONFIG,
            readme,
        );
        let adr = "# FIX-rule-x: X\n\n## Decision\n\nIt should run.\nLine two of the rule.\n\
                   Line three of the rule.\nLine four of the rule.\nLine five of the rule.\n\
                   Line six of the rule.\n";
        fixture.write(".gitattributes", "*.bin filter=strip\n");
        fixture.write("docs/adr/FIX-rule-x.md", adr);
        fixture.commit("rule");
        let strip = fixture.root().join(".git").join("strip.sh");
        std::fs::write(&strip, "#!/bin/sh\ngrep -v '^JUNK'\n").expect("written");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&strip, std::fs::Permissions::from_mode(0o755)).expect("mode");
        fixture.git(&[
            "config",
            "filter.strip.clean",
            strip.to_str().expect("UTF-8"),
        ]);
        fixture.git(&["mv", "docs/adr/FIX-rule-x.md", "docs/adr/FIX-rule-y.md"]);
        fixture.write(
            "docs/adr/FIX-rule-y.md",
            &adr.replace("Line six of the rule.", "Line six, edited."),
        );
        fixture.write(name, &format!("{}{adr}", "JUNK line\n".repeat(30)));
        fixture.git(&["add", name]);
        stat_changed(&fixture.root().join(name));
        let output = fixture.nuc(&["sense", "plain-rules", "--require", "HEAD"]);
        assert_eq!(code(&output), Some(2), "{name}: {}", stdout(&output));
        assert_eq!(stdout(&output), "", "{name}");
        assert!(
            stderr(&output)
                .trim_end()
                .ends_with("[sense/plain-rules:filter]"),
            "{name}: {}",
            stderr(&output)
        );
    }
}

/// An ADR the change adds beside a file under a filter, as a new ADR comes with its diagram under
/// Git LFS, is no refusal where nothing is deleted, so that no rename can pair otherwise: on a
/// clean checkout, as CI runs the gate, with the driver configured and without one. The parity
/// scenario `plain_filtered_added` holds the report to the bash's.
#[cfg(unix)]
#[test]
fn plain_rules_runs_on_an_adr_added_beside_a_filtered_file() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    for configured in [true, false] {
        let fixture = Fixture::project(
            &format!("sense-plain-rules-adr-image-{configured}"),
            CONFIG,
            readme,
        );
        fixture.write(".gitattributes", "*.png filter=bait\n");
        fixture.commit("attributes");
        fixture.write(
            "docs/adr/FIX-rule-x.md",
            "# FIX-rule-x: X\n\n## Decision\n\nIt should run.\n",
        );
        fixture.write("img/d.png", "image\n");
        fixture.commit("adr and diagram");
        let marker = fixture.root().join(".git").join("added-filter-ran");
        if configured {
            let clean = bait(&fixture, "clean.sh", &marker);
            fixture.git(&["config", "filter.bait.clean", &clean]);
        }
        let output = fixture.nuc(&["sense", "plain-rules", "--require", "HEAD~1"]);
        assert_eq!(code(&output), Some(1), "{configured}: {}", stderr(&output));
        assert!(
            stdout(&output).contains(
                "  - docs/adr/FIX-rule-x.md:5: the soft modal verb \"should\" (fails) [rules/soft-modal:verb]\n"
            ),
            "{configured}: {}",
            stdout(&output)
        );
        assert!(!marker.exists(), "nuc sense plain-rules ran the filter");
    }
}

/// `check-attr` prints `set` for a bare `filter` and for a driver named `set` alike: where a driver
/// of that name is configured, a file of rules under the value is not decided, and the driver does
/// not run (Q-1, Q-7).
#[cfg(unix)]
#[test]
fn plain_rules_refuses_a_rule_file_under_a_driver_named_set() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let fixture = Fixture::project("sense-plain-rules-driver-set", CONFIG, readme);
    fixture.write(".gitattributes", "docs/adr/*.md filter=set\n");
    fixture.write(
        "docs/adr/FIX-rule-x.md",
        "# FIX-rule-x: X\n\n## Decision\n\nIt runs.\n",
    );
    fixture.commit("rule");
    let marker = fixture.root().join(".git").join("set-filter-ran");
    let clean = bait(&fixture, "clean.sh", &marker);
    fixture.git(&["config", "filter.set.clean", &clean]);
    stat_changed(&fixture.root().join("docs/adr/FIX-rule-x.md"));
    let output = fixture.nuc(&["sense", "plain-rules", "--require", "HEAD"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output)
            .trim_end()
            .ends_with("[sense/plain-rules:filter]"),
        "{}",
        stderr(&output)
    );
    assert!(!marker.exists(), "nuc sense plain-rules ran the driver set");
}

/// A driver that no entry gives a command runs nothing and is left as git reads it: one that is
/// only `required`, on a file whose stat changed, makes nuc's git die as the bash's does, and the
/// run is not decided, where the bash exits 2 as well.
#[cfg(unix)]
#[test]
fn plain_rules_leaves_a_driver_without_a_command_to_git() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let fixture = Fixture::project("sense-plain-rules-required-only", CONFIG, readme);
    fixture.write(".gitattributes", "*.png filter=req\n");
    fixture.write("a.png", "image\n");
    fixture.write("AGENTS.md", "# Agent Guide\n\n- The human decides.\n");
    fixture.commit("rules");
    fixture.git(&["config", "filter.req.required", "true"]);
    stat_changed(&fixture.root().join("a.png"));
    fixture.write(
        "AGENTS.md",
        "# Agent Guide\n\n- The human decides.\n- It might run.\n",
    );
    let output = fixture.nuc(&["sense", "plain-rules", "HEAD"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output)
            .trim_end()
            .ends_with("[sense/plain-rules:git]"),
        "{}",
        stderr(&output)
    );
}

/// A file of rules the change only deletes from is not read, as the bash does not read it: one
/// that holds a NUL byte is no refusal there, with a head and in the working tree.
#[test]
fn plain_rules_reads_no_file_the_change_adds_no_line_to() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let fixture = Fixture::project("sense-plain-rules-nul", CONFIG, readme);
    let text = "# FIX-rule-n: N\n\n## Decision\n\nA \0 byte.\nThe human decides.\n";
    fixture.write("docs/adr/FIX-rule-n.md", text);
    fixture.commit("nul");
    fixture.git(&["switch", "-q", "-c", "work"]);
    fixture.write(
        "docs/adr/FIX-rule-n.md",
        &text.replace("The human decides.\n", ""),
    );
    fixture.commit("deleted");
    for args in [
        &["sense", "plain-rules", "main", "work"][..],
        &["sense", "plain-rules", "main"],
    ] {
        let output = fixture.nuc(args);
        assert_eq!(code(&output), Some(0), "{args:?}: {}", stderr(&output));
        assert!(
            stdout(&output).contains("no long sentence, no soft modal verb"),
            "{args:?}: {}",
            stdout(&output)
        );
    }
}

/// `plain-rules --require` fails on a soft modal verb in a project ADR, its report on the standard
/// output as the bash writes it, and exits 1; without the flag it reports and exits 0. Without a
/// head, the change ends in the working tree, its untracked files included. A release before the
/// flag reads `--require` as a base, which names no commit; a release before the sensor has none.
#[test]
fn plain_rules_require_fails_on_a_project_adr_and_reads_the_working_tree() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let fixture = Fixture::project("sense-plain-rules", CONFIG, readme);
    fixture.write(
        "docs/adr/FIX-rule-x.md",
        "# FIX-rule-x: X\n\n## Decision\n\nThe human decides.\n",
    );
    fixture.commit("rules");
    fixture.git(&["switch", "-q", "-c", "work"]);
    fixture.write(
        "docs/adr/FIX-rule-y.md",
        "# FIX-rule-y: Y\n\n## Decision\n\nWe might do it.\n",
    );
    let output = fixture.nuc(&["sense", "plain-rules", "--require"]);
    assert_eq!(code(&output), Some(1), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("..working tree): 1 to read"), "{text}");
    assert!(
        text.contains(
            "  - docs/adr/FIX-rule-y.md:5: the soft modal verb \"might\" (fails) [rules/soft-modal:verb]\n"
        ),
        "{text}"
    );
    assert!(
        text.ends_with("1 of them fail: a soft modal verb or an avoided word in a project ADR (NUC-documents-are-fixed §5).\n"),
        "{text}"
    );
    let output = fixture.nuc(&["sense", "plain-rules", "main"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("\"might\" [rules/soft-modal:verb]\n"),
        "{}",
        stdout(&output)
    );
    for (release, reason) in [("v0.8.1", "ref"), ("v0.7.0", "release")] {
        fixture.write(
            "README.md",
            &format!("# Fixture\n\n- **Template release:** {release}\n"),
        );
        let output = fixture.nuc(&["sense", "plain-rules", "--require", "main"]);
        assert_eq!(code(&output), Some(2), "{release}");
        assert_eq!(stdout(&output), "", "{release}");
        assert!(
            stderr(&output)
                .trim_end()
                .ends_with(&format!("[sense/plain-rules:{reason}]")),
            "{release}: {}",
            stderr(&output)
        );
    }
}

/// A partial clone's lazy fetch of an object it lacks starts no transport, so that no command the
/// repository's configuration names for one runs, not even where it turns the transport back on
/// (Q-7, Q-11); and an entry whose object the clone lacks is not decided, where reading it as an
/// absent file would name the entry by its file name (Q-1).
/// Verifies: SEC-git-cleared-without-lazy-fetch
#[cfg(unix)]
#[test]
fn a_lazy_fetch_of_a_partial_clone_runs_no_command_of_the_repository() {
    let source = Fixture::project("sense-promisor-source", CONFIG, README);
    source.write("docs/backlog/one.md", "# One\n");
    source.commit("entry");
    source.git(&["config", "uploadpack.allowFilter", "true"]);
    let holder = Fixture::new("sense-promisor-clone");
    let url = format!("file://{}", source.root().display());
    let cloned = holder.git(&[
        "clone",
        "-q",
        "--filter=blob:none",
        "--no-checkout",
        &url,
        "pc",
    ]);
    assert!(
        cloned.status.success(),
        "{}",
        String::from_utf8_lossy(&cloned.stderr)
    );
    let clone = holder.root().join("pc");
    let git = |args: &[&str]| support::git_in(&clone, args);
    git(&["config", "remote.origin.url", "ssh://example.invalid/x"]);
    let marker = holder.root().join(".git").join("transport-ran");
    let ssh = bait(&holder, "ssh.sh", &marker);
    git(&["config", "core.sshCommand", &ssh]);
    git(&["config", "protocol.ssh.allow", "always"]);
    // The files nuc reads before the sensor, in the work tree that the clone left unchecked out.
    std::fs::write(clone.join("nuc.toml"), CONFIG).expect("written");
    std::fs::write(
        clone.join("README.md"),
        "# Fixture\n\n- **Template release:** v0.12.0\n",
    )
    .expect("written");
    let repo = clone.to_str().expect("UTF-8");
    let output = holder.nuc(&["sense", "backlog", "--repo", repo]);
    assert!(!marker.exists(), "nuc sense backlog started a transport");
    assert_eq!(code(&output), Some(2), "{}", stderr(&output));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).contains("as in a partial clone"),
        "{}",
        stderr(&output)
    );
}

/// `nuc check` reads the README at the merge base with main for `docs/template-release`: where a
/// partial clone lacks its object, the run is not decided, rather than read as a README the base
/// does not have (Q-1).
/// Verifies: SEC-git-cleared-without-lazy-fetch
#[test]
fn the_readme_at_the_base_that_a_partial_clone_lacks_is_not_decided() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let config = "minimum = \"0.0.0\"\nrules = [\"docs/template-release\"]\n";
    let source = Fixture::project("base-promisor-source", config, readme);
    let base = support::stdout(&source.git(&["rev-parse", "HEAD"]));
    source.write("README.md", &format!("{readme}\nMore.\n"));
    source.commit("more");
    source.git(&["config", "uploadpack.allowFilter", "true"]);
    let holder = Fixture::new("base-promisor-clone");
    let url = format!("file://{}", source.root().display());
    let cloned = holder.git(&["clone", "-q", "--filter=blob:none", &url, "pc"]);
    assert!(
        cloned.status.success(),
        "{}",
        String::from_utf8_lossy(&cloned.stderr)
    );
    let clone = holder.root().join("pc");
    support::git_in(
        &clone,
        &["update-ref", "refs/remotes/origin/main", base.trim()],
    );
    let repo = clone.to_str().expect("UTF-8");
    let output = holder.nuc(&["check", "--repo", repo]);
    assert_eq!(code(&output), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("as in a partial clone"),
        "{}",
        stderr(&output)
    );
}

/// An ADR of `binding-adrs` that names the rule of a check of `c.sh`, and `c.sh` with two checks.
#[cfg(unix)]
fn binding_checks(fixture: &Fixture, b: &str) {
    for (name, key) in [("FIX-a-rule", "t/a"), ("FIX-b-rule", "t/b")] {
        fixture.write(
            &format!("docs/adr/{name}.md"),
            &format!(
                "# {name}: X\n\n- **Status:** 🟢 accepted\n- **Applies to:** the rule `{key}` of \
                 `c.sh`\n"
            ),
        );
    }
    fixture.write(
        "c.sh",
        &format!("set -u\ncheck_a() {{  # t/a\n  :\n}}\n\ncheck_b() {{  # t/b\n  {b}\n}}\n"),
    );
}

/// The diff `binding-adrs` reads the hunks of a file named with checks from runs no textconv
/// driver, no external diff, and no colour the repository names (Q-7): each would hide the hunks,
/// and the change inside one check would bind the ADR of the other.
#[cfg(unix)]
#[test]
fn binding_adrs_runs_no_driver_of_the_repository() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let fixture = Fixture::project("sense-binding-bait", CONFIG, readme);
    fixture.write(".gitattributes", "*.sh diff=bait\n");
    binding_checks(&fixture, ":");
    fixture.commit("checks");
    fixture.git(&["switch", "-q", "-c", "feature"]);
    binding_checks(&fixture, "true");
    fixture.commit("inside b");
    let marker = fixture.root().join(".git").join("driver-ran");
    let textconv = bait(&fixture, "textconv.sh", &marker);
    let command = bait(&fixture, "command.sh", &marker);
    fixture.git(&["config", "diff.bait.textconv", &textconv]);
    fixture.git(&["config", "diff.bait.command", &command]);
    fixture.git(&["config", "diff.external", &command]);
    fixture.git(&["config", "color.diff", "always"]);
    let output = fixture.nuc(&["sense", "binding-adrs", "main", "feature"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).ends_with(
            "1 path(s) changed; 1 ADR(s) in force name one of them. Read these, and the index for \
             the ADRs whose Applies to is prose:\n  - FIX-b-rule (docs/adr/FIX-b-rule.md) — c.sh\n"
        ),
        "{}",
        stdout(&output)
    );
    assert!(!marker.exists(), "nuc sense binding-adrs ran a driver");

    // A submodule named with checks of its own, whose commit the change moves.
    let (main, marker) = moved_submodule("sense-binding-bait", "the rule `t/a` of `sm`");
    let output = main.nuc(&["sense", "binding-adrs", "main", "feature"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).ends_with("  - FIX-s-rule (docs/adr/FIX-s-rule.md) — sm\n"),
        "{}",
        stdout(&output)
    );
    assert!(
        !marker.exists(),
        "a diff in the submodule ran its external diff"
    );
}

/// A repository whose branch `feature` moves the commit of the submodule `sm`, an ADR naming what
/// the *Applies to* says, `diff.submodule=diff` set, and an external diff in the submodule's
/// configuration that touches the marker returned: git would start a diff in the submodule that
/// reads none of nuc's flags and runs it, but where `diff.submodule=short` is neutralized.
#[cfg(unix)]
fn moved_submodule(name: &str, applies: &str) -> (Fixture, std::path::PathBuf) {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let sub = Fixture::project(&format!("{name}-sub"), CONFIG, readme);
    let main = Fixture::project(&format!("{name}-super"), CONFIG, readme);
    let url = sub.root().to_str().expect("UTF-8");
    main.git(&[
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "-q",
        "add",
        url,
        "sm",
    ]);
    main.write(
        "docs/adr/FIX-s-rule.md",
        &format!("# FIX-s-rule: X\n\n- **Status:** 🟢 accepted\n- **Applies to:** {applies}\n"),
    );
    main.commit("submodule");
    main.git(&["switch", "-q", "-c", "feature"]);
    let sm = main.root().join("sm");
    std::fs::write(sm.join("more.txt"), "more\n").expect("written");
    support::git_in(&sm, &["add", "-A"]);
    support::git_in(&sm, &["commit", "-q", "-m", "more"]);
    main.commit("move the submodule");
    let marker = main.root().join(".git").join("submodule-diff-ran");
    let command = bait(&main, "command.sh", &marker);
    main.git(&["config", "diff.submodule", "diff"]);
    support::git_in(&sm, &["config", "diff.external", &command]);
    (main, marker)
}

/// A change that ends in the working tree runs no filter the repository names (Q-7). A path under
/// a filter whose stat changed while its bytes did not reads alike to the bash, which runs the
/// filter, and to nuc, which runs none; one whose bytes changed is not decided, since the bash's
/// git tells it through the filter. A submodule checked out is not decided either, whatever its
/// `ignore` setting, since the bash's git may ask its own status, which runs its filters.
#[cfg(unix)]
#[test]
fn binding_adrs_in_the_working_tree_runs_no_filter() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let fixture = Fixture::project("sense-binding-filter", CONFIG, readme);
    fixture.write(".gitattributes", "*.png filter=bait\n");
    fixture.write("a.png", "image\n");
    binding_checks(&fixture, ":");
    fixture.commit("image");
    let marker = fixture.root().join(".git").join("filter-ran");
    let clean = bait(&fixture, "clean.sh", &marker);
    fixture.git(&["config", "filter.bait.clean", &clean]);
    fixture.git(&["config", "filter.bait.process", &clean]);
    fixture.git(&["config", "filter.bait.required", "true"]);
    stat_changed(&fixture.root().join("a.png"));
    binding_checks(&fixture, "true");
    let output = fixture.nuc(&["sense", "binding-adrs", "HEAD"]);
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).ends_with("  - FIX-b-rule (docs/adr/FIX-b-rule.md) — c.sh\n"),
        "{}",
        stdout(&output)
    );
    assert!(
        !marker.exists(),
        "nuc sense binding-adrs ran the filter of a.png"
    );
    fixture.write("a.png", "another image\n");
    let output = fixture.nuc(&["sense", "binding-adrs", "HEAD"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output)
            .trim_end()
            .ends_with("[sense/binding-adrs:filter]"),
        "{}",
        stderr(&output)
    );
    assert!(
        !marker.exists(),
        "nuc sense binding-adrs ran the filter of a.png"
    );

    // A submodule whose own configuration names a filter for a file whose stat changed.
    let sub = Fixture::project("sense-binding-sub", CONFIG, readme);
    sub.write(".gitattributes", "*.txt filter=bait\n");
    sub.write("f.txt", "a\n");
    sub.commit("sub");
    let main = Fixture::project("sense-binding-super", CONFIG, readme);
    let url = sub.root().to_str().expect("UTF-8");
    let added = main.git(&[
        "-c",
        "protocol.file.allow=always",
        "submodule",
        "-q",
        "add",
        url,
        "sm",
    ]);
    assert!(added.status.success(), "{}", stderr(&added));
    main.commit("submodule");
    let marker = main.root().join(".git").join("sub-filter-ran");
    let clean = bait(&main, "clean.sh", &marker);
    support::git_in(
        &main.root().join("sm"),
        &["config", "filter.bait.clean", &clean],
    );
    stat_changed(&main.root().join("sm/f.txt"));
    main.write("README.md", &format!("{readme}\nMore.\n"));
    let output = main.nuc(&["sense", "binding-adrs", "HEAD"]);
    assert!(
        !marker.exists(),
        "nuc sense binding-adrs ran a submodule's filter"
    );
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output)
            .trim_end()
            .ends_with("[sense/binding-adrs:submodule]"),
        "{}",
        stderr(&output)
    );
    // Under the repository's `ignore = all` the bash's git never looks into the submodule, but nuc
    // refuses a submodule checked out alike, so that no flag of its own overrides that setting.
    main.git(&["config", "-f", ".gitmodules", "submodule.sm.ignore", "all"]);
    let output = main.nuc(&["sense", "binding-adrs", "HEAD"]);
    assert!(
        !marker.exists(),
        "nuc sense binding-adrs ran a submodule's filter"
    );
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("a submodule checked out, whose state the bash's git may read"),
        "{}",
        stderr(&output)
    );
}

/// `binding-adrs` refuses what its bash reads otherwise or not at all (Q-1): `--path` without a
/// path and `--root`, as the value of `--path` too, exit 2 as a usage error, an ADR with a NUL
/// byte, which the bash's command substitution drops, is not decided, and so is a path that is no
/// UTF-8 where the sensor has to hand it to git.
#[test]
fn binding_adrs_refuses_what_it_cannot_read_as_the_bash_does() {
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let fixture = Fixture::project("sense-binding-refusals", CONFIG, readme);
    for args in [
        &["sense", "binding-adrs", "--path"][..],
        &["sense", "binding-adrs", "--sections", "--path", ""],
        &["sense", "binding-adrs", "--root", "."],
        &["sense", "binding-adrs", "--path", "--root"],
        &["sense", "binding-adrs", "--path", "x", "HEAD"],
    ] {
        let output = fixture.nuc(args);
        assert_eq!(code(&output), Some(2), "{args:?}: {}", stdout(&output));
        assert_eq!(stdout(&output), "", "{args:?}");
    }
    fixture.write(
        "docs/adr/FIX-a-rule.md",
        "# FIX-a-rule: X\n\n- **Status:** 🟢 accepted\n- **Applies to:** `README.md`\n\0\n",
    );
    let output = fixture.nuc(&["sense", "binding-adrs", "--path", "README.md"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output)
            .trim_end()
            .ends_with("[sense/binding-adrs:file]"),
        "{}",
        stderr(&output)
    );
}

/// A changed path that is no UTF-8 and named with checks of its own is not decided: nuc would have
/// to hand it to git for its hunks.
#[cfg(unix)]
#[test]
fn binding_adrs_refuses_a_path_it_cannot_hand_to_git() {
    use std::os::unix::ffi::OsStrExt;
    let readme = "# Fixture\n\n- **Template release:** v0.12.0\n";
    let fixture = Fixture::project("sense-binding-name", CONFIG, readme);
    let mut adr = b"# FIX-a-rule: X\n\n- **Status:** \xf0\x9f\x9f\xa2 accepted\n- **Applies to:** \
                    the rule `t/a` of `\xff.sh`\n"
        .to_vec();
    adr.push(b'\n');
    std::fs::create_dir_all(fixture.root().join("docs/adr")).expect("a directory");
    std::fs::write(fixture.root().join("docs/adr/FIX-a-rule.md"), adr).expect("written");
    let odd = fixture.root().join(std::ffi::OsStr::from_bytes(b"\xff.sh"));
    // A file system that holds only UTF-8 names, as APFS on macOS, cannot have the path at all.
    if std::fs::write(&odd, "check_a() {  # t/a\n  :\n}\n").is_err() {
        return;
    }
    let output = fixture.nuc(&["sense", "binding-adrs", "HEAD"]);
    assert_eq!(code(&output), Some(2), "{}", stdout(&output));
    assert!(
        stderr(&output)
            .trim_end()
            .ends_with("[sense/binding-adrs:name]"),
        "{}",
        stderr(&output)
    );
}
