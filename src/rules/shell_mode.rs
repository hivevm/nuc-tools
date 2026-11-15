//! The rule `shell/mode` of the template's `scripts/check-shell.sh` from v0.10.0 on: a tracked
//! script `scripts/*.sh`, or a skill's `.agents/skills/<name>/*.sh`, has the mode 100755 in git's
//! index (NUC-checks-are-bash §9). It is the one rule of `check-shell.sh` that `nuc` ports; the
//! others run the project's scripts (ARCH-rules-need-only-git). A tree of an earlier release has
//! no such check.

use super::{Input, Inputs, Outcome, Rule, TEMPLATE_FINDINGS};
use crate::finding::Finding;

pub const RULE_ID: &str = "shell/mode";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["exec"],
    explain: "A tracked script scripts/*.sh, or a skill's .agents/skills/<name>/*.sh, has the mode \
              100755 in git's index: each is run, and a file without the executable bit refuses a \
              direct call. The mode is read from the index, so the rule holds on a file system \
              that keeps none. A file deeper below scripts/ or a skill's directory, a symbolic \
              link, a file git does not track, and a file in conflict are not held. From template \
              release v0.10.0 on.",
    reads: &[Input::IndexModes, Input::Release],
    check,
};

/// One entry of `git ls-files -s` for the pathspecs the bash names: the path, its mode, and its
/// stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    pub path: String,
    pub mode: String,
    pub stage: String,
}

/// Whether the bash holds the path: a file directly in `scripts/`, or directly in the directory of
/// a skill under `.agents/skills/`, as its two `${rel#…}` cut it.
fn held(path: &str) -> bool {
    if let Some(rest) = path.strip_prefix("scripts/") {
        return !rest.contains('/');
    }
    path.strip_prefix(".agents/skills/")
        .and_then(|rest| rest.split_once('/'))
        .is_some_and(|(_, file)| !file.contains('/'))
}

/// The path as the bash's `printf %q` quotes it, where it takes no quoting: letters, digits, and
/// `%+-./:=@_`; none for any other path, whose quoting this rule does not decide, among them one
/// read with U+FFFD for bytes that are no UTF-8.
fn quoted(path: &str) -> Option<&str> {
    path.bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"%+-./:=@_".contains(&b))
        .then_some(path)
}

fn check(inputs: &Inputs) -> Outcome {
    if !inputs.carries_at_least(TEMPLATE_FINDINGS) {
        return Outcome::default();
    }
    let mut findings = Vec::new();
    for entry in &inputs.index_modes {
        if entry.stage != "0"
            || !held(&entry.path)
            || entry.mode == "100755"
            || entry.mode == "120000"
        {
            continue;
        }
        let Some(quoted) = quoted(&entry.path) else {
            return Outcome {
                refusal: Some(format!(
                    "{}: a script's path that printf %q quotes, which shell/mode does not spell as \
                     the bash does — not decided",
                    entry.path
                )),
                ..Outcome::default()
            };
        };
        findings.push(Finding {
            rule: RULE_ID,
            reason: "exec",
            file: entry.path.clone(),
            line: None,
            message: format!(
                "is tracked with mode {}, not 100755 (NUC-checks-are-bash §9), so a direct call is \
                 refused — a script is run, and git keeps its executable bit: git update-index \
                 --chmod=+x -- {quoted}",
                entry.mode
            ),
        });
    }
    findings.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, mode: &str, stage: &str) -> IndexEntry {
        IndexEntry {
            path: path.to_owned(),
            mode: mode.to_owned(),
            stage: stage.to_owned(),
        }
    }

    fn files(entries: Vec<IndexEntry>, release: Option<crate::config::Version>) -> Vec<String> {
        let inputs = Inputs {
            index_modes: entries,
            release,
            ..Inputs::default()
        };
        check(&inputs)
            .findings
            .into_iter()
            .map(|f| f.file)
            .collect()
    }

    #[test]
    fn a_script_without_the_executable_bit_fails_as_the_bash_reads_the_index() {
        let entries = vec![
            entry("scripts/a.sh", "100644", "0"),
            entry("scripts/b.sh", "100755", "0"),
            entry("scripts/link.sh", "120000", "0"),
            entry("scripts/lib/c.sh", "100644", "0"),
            entry("scripts/fixtures/d.sh", "100644", "0"),
            entry("scripts/e.sh", "100644", "2"),
            entry(".agents/skills/s/f.sh", "100644", "0"),
            entry(".agents/skills/s/deep/g.sh", "100644", "0"),
        ];
        assert_eq!(
            files(entries.clone(), None),
            ["scripts/a.sh", ".agents/skills/s/f.sh"]
        );
        assert!(files(entries, Some(crate::config::Version(0, 9, 1))).is_empty());
    }

    #[test]
    fn the_message_names_the_mode_and_the_command_that_fixes_it() {
        let inputs = Inputs {
            index_modes: vec![entry("scripts/a.sh", "100644", "0")],
            ..Inputs::default()
        };
        assert_eq!(
            check(&inputs).findings[0].message,
            "is tracked with mode 100644, not 100755 (NUC-checks-are-bash §9), so a direct call \
             is refused — a script is run, and git keeps its executable bit: git update-index \
             --chmod=+x -- scripts/a.sh"
        );
        for odd in ["scripts/a b.sh", "scripts/a,b.sh", "scripts/a\u{fffd}.sh"] {
            let inputs = Inputs {
                index_modes: vec![entry(odd, "100644", "0")],
                ..Inputs::default()
            };
            assert!(check(&inputs).refusal.is_some(), "{odd}");
        }
    }
}
