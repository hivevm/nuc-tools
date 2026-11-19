//! The rule `hooks/toml` of the template's `scripts/check-hooks.sh` from v0.12.0 on: `nuc.toml`
//! exists, its minimum is no older than the release the blob list of the hooks names, and its
//! rules name every `hook/` rule the list names (NUC-hooks-run-nuc §3). The file is read as `nuc`
//! reads it for every command, by `src/config.rs`. Each of these ends the run before any rule:
//! no `nuc.toml`, one `nuc` cannot read, one without a minimum, a minimum above the running `nuc`,
//! and a rule ID no `nuc` knows. So this rule decides only a file `nuc` understands. A tree of an
//! earlier release has no such check.

use super::{HOOK_PINS, Input, Inputs, Outcome, Rule};
use crate::config::Version;
use crate::finding::Finding;

pub const RULE_ID: &str = "hooks/toml";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["absent", "minimum", "rule"],
    explain: "nuc.toml exists at the root, its minimum is no older than the release of \
              hivevm/nuc-tools the blob list .githooks/nuc.blobs names, and its rules name every \
              hook/ rule the list names: a nuc older than its shims decides what no release \
              tested, and with a nuc.toml its list governs which rules run, so no project removes \
              a hook/ rule. A rule only in a comment is not named. From template release v0.12.0 \
              on.",
    reads: &[Input::HookList, Input::Toml, Input::Release],
    check,
};

const TOML: &str = "nuc.toml";

fn finding(reason: &'static str, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        reason,
        file: TOML.to_owned(),
        line: None,
        message,
    }
}

fn check(inputs: &Inputs) -> Outcome {
    if !inputs.carries_at_least(HOOK_PINS) {
        return Outcome::default();
    }
    let list = inputs
        .hooks
        .list
        .as_deref()
        .map(crate::pin::read_list)
        .unwrap_or_default();
    let Some(config) = &inputs.toml else {
        return vec![finding(
            "absent",
            "not found — it names the minimum nuc and the hook/ rules the hooks run \
             (NUC-hooks-run-nuc §3)"
                .to_owned(),
        )]
        .into();
    };
    let mut findings = Vec::new();
    if let Some(release) = &list.release {
        // The bash compares each part in base 10 in its 64-bit arithmetic, which wraps where
        // `nuc` cannot hold the number.
        let Some(version) = Version::parse(release) else {
            return Outcome {
                refusal: Some(format!(
                    "{}: the release v{release} has a part beyond 64 bits, which the bash's \
                     arithmetic wraps — not decided",
                    crate::pin::LIST
                )),
                ..Outcome::default()
            };
        };
        if config.minimum < version {
            findings.push(finding(
                "minimum",
                format!(
                    "its minimum {} is older than nuc-tools v{release}, the release of the hooks \
                     — a nuc older than its shims decides what no release tested \
                     (NUC-hooks-run-nuc §3)",
                    config.minimum_text
                ),
            ));
        }
    }
    for rule in &list.rules {
        if !config.rules.contains(rule) {
            findings.push(finding(
                "rule",
                format!(
                    "its rules do not name '{rule}' — with a nuc.toml its list governs which \
                     rules run, and no project removes a hook/ rule (NUC-hooks-run-nuc §3)"
                ),
            ));
        }
    }
    findings.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    /// The rules of the blob list of the tests.
    const RULES: &str = "\"hook/commit-on-main\", \"hook/push-checks\", \"hook/main-moves\"";

    /// A blob list of the release that names the three rules of [`RULES`].
    fn list(release: &str) -> Vec<u8> {
        format!(
            "release v{release}\nrule hook/commit-on-main\nrule hook/push-checks\n\
             rule hook/main-moves\n"
        )
        .into_bytes()
    }

    /// The findings on `nuc.toml` as `nuc` reads `toml`, or none, beside the list of the release.
    fn keys(release: &str, toml: Option<&str>) -> Vec<String> {
        let toml = toml.map(|text| {
            config::read(text, Version(u64::MAX, 0, 0)).expect("nuc reads the nuc.toml")
        });
        let inputs = Inputs {
            hooks: crate::pin::Hooks {
                list: Some(list(release)),
                ..Default::default()
            },
            toml,
            ..Inputs::default()
        };
        check(&inputs)
            .findings
            .into_iter()
            .map(|f| format!("{}: {} [{}]", f.file, f.message, f.reason))
            .collect()
    }

    fn toml(minimum: &str, rules: &str) -> String {
        format!("minimum = \"{minimum}\"\n{rules}\n")
    }

    #[test]
    fn a_nuc_toml_of_the_release_with_every_hook_rule_passes() {
        assert!(keys("0.2.2", Some(&toml("0.2.2", &format!("rules = [{RULES}]")))).is_empty());
        let inputs = Inputs {
            release: Some(Version(0, 11, 1)),
            ..Inputs::default()
        };
        assert_eq!(check(&inputs), Outcome::default());
    }

    /// The rule's answer where it is given no `nuc.toml`. `nuc check` never gives it none: it
    /// refuses a tree without one first, `config/toml:missing`, as tests/check.rs asserts.
    #[test]
    fn no_nuc_toml_is_absent() {
        assert_eq!(
            keys("0.2.2", None),
            [
                "nuc.toml: not found — it names the minimum nuc and the hook/ rules the hooks run \
              (NUC-hooks-run-nuc §3) [absent]"
            ]
        );
    }

    #[test]
    fn a_minimum_older_than_the_release_fails_as_written() {
        // bash: a minimum older than the release of the hooks
        assert_eq!(
            keys(
                "0.2.2",
                Some(&toml("0.01.0", &format!("rules = [{RULES}]")))
            ),
            [
                "nuc.toml: its minimum 0.01.0 is older than nuc-tools v0.2.2, the release of the \
              hooks — a nuc older than its shims decides what no release tested \
              (NUC-hooks-run-nuc §3) [minimum]"
            ]
        );
        assert_eq!(
            keys("10.0.0", Some(&toml("9.99.99", "rules = []"))).len(),
            4
        );
    }

    #[test]
    fn a_minimum_newer_than_the_release_and_further_rules_pass() {
        // bash: a minimum newer than the release, on one line of rules
        assert!(keys("0.2.2", Some(&toml("0.2.3", &format!("rules = [{RULES}]")))).is_empty());
        // bash: further rules besides the hook/ rules
        let further = format!("rules = [{RULES}, \"docs/prose-width\"]");
        assert!(keys("0.2.2", Some(&toml("0.3.0", &further))).is_empty());
        // bash: a minimum of a newer major
        assert!(keys("0.2.2", Some(&toml("1.0.0", &format!("rules = [{RULES}]")))).is_empty());
        assert!(keys("0.2.2", Some(&toml("0.2.10", "rules = [\n  \"hook/commit-on-main\",\n  \"hook/push-checks\",\n  \"hook/main-moves\",\n]"))).is_empty());
    }

    #[test]
    fn a_rules_list_that_lacks_a_hook_rule_names_it() {
        // bash: a rules list that lacks a hook/ rule
        let lacking = "rules = [\"hook/commit-on-main\", \"hook/main-moves\"]";
        assert_eq!(
            keys("0.2.2", Some(&toml("0.2.2", lacking))),
            [
                "nuc.toml: its rules do not name 'hook/push-checks' — with a nuc.toml its list \
              governs which rules run, and no project removes a hook/ rule \
              (NUC-hooks-run-nuc §3) [rule]"
            ]
        );
    }

    #[test]
    fn a_hook_rule_only_in_a_comment_is_not_named_and_a_hash_in_a_string_is_no_comment() {
        // bash: a hook/ rule only in a comment after a rule
        let after =
            "rules = [\n\"hook/commit-on-main\", \"hook/main-moves\", # \"hook/push-checks\",\n]";
        assert_eq!(keys("0.2.2", Some(&toml("0.2.2", after))).len(), 1);
        // bash: a hook/ rule only in a comment of the rules list
        let commented =
            "rules = [\n  # \"hook/push-checks\",\n\"hook/commit-on-main\", \"hook/main-moves\"\n]";
        assert_eq!(keys("0.2.2", Some(&toml("0.2.2", commented))).len(), 1);
        // test-check-hooks.sh: a '#' inside a string starts no comment
        // The bash passes it; nuc reads the '#' as part of the string, and refuses the rule ID no
        // nuc knows before any rule runs.
        let hash = format!("rules = [{RULES}, \"docs/x#y\"] # a comment");
        assert_eq!(
            config::read(&toml("0.2.2", &hash), Version(9, 0, 0)).err(),
            Some(config::Refusal::UnknownRule("docs/x#y".to_owned()))
        );
    }

    #[test]
    fn a_release_beyond_64_bits_is_not_decided() {
        let inputs = Inputs {
            hooks: crate::pin::Hooks {
                list: Some(list("18446744073709551616.0.0")),
                ..Default::default()
            },
            toml: Some(config::read(&toml("0.2.2", "rules = []"), Version(1, 0, 0)).unwrap()),
            ..Inputs::default()
        };
        assert!(check(&inputs).refusal.is_some());
    }
}
