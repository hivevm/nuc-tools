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
}

/// A partial clone's lazy fetch of an object it lacks starts no transport, so that no command the
/// repository's configuration names for one runs (Q-7, Q-11): `backlog` shows its entries, and
/// falls back to their file names where the object is missing.
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
    assert_eq!(code(&output), Some(0), "{}", stderr(&output));
}
