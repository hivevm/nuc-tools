//! The decisions of `nuc hook`: what each git hook of the template decides from the git state,
//! as the bash hook of the release does (ADR-0021). The shell reads the state; these functions
//! decide on it, and the reference shim runs the project's check where they say it runs.

use crate::finding::Finding;

/// A hook rule: its ID and the reason `nuc explain` gives. A hook rule decides in `nuc hook`
/// alone; `nuc check` knows its ID, so that one `nuc.toml` lists both, and runs it never.
pub struct HookRule {
    pub id: &'static str,
    pub explain: &'static str,
}

/// The refusal of a commit on `main`.
pub const COMMIT_ON_MAIN: HookRule = HookRule {
    id: "hook/commit-on-main",
    explain: "a commit lands on a branch, never on main, and changes reach main through a pull \
              request (AGENTS.md, section 6); the pre-commit hook refuses a commit while the \
              checkout is on main",
};

/// The full check, with `--changed`, on the commit that concludes a conflicted merge.
pub const MERGE_CHECKS: HookRule = HookRule {
    id: "hook/merge-checks",
    explain: "the resolution of a conflicted merge is the one part of a merge written by hand, so \
              the pre-commit hook has scripts/check-all.sh --changed run on the commit that \
              concludes it, and refuses it while the checks are red",
};

/// Every hook rule this `nuc` knows, all of them on where the repository has no `nuc.toml`
/// (ADR-0023).
pub const ALL: &[HookRule] = &[COMMIT_ON_MAIN, MERGE_CHECKS];

/// The hook rule with the ID, if this `nuc` knows it.
pub fn find(id: &str) -> Option<&'static HookRule> {
    ALL.iter().find(|rule| rule.id == id)
}

/// What a hook answers the shim: pass, refuse with a finding, or pass so far and the check of the
/// hook runs now, exit 0, 1, and 3 (ADR-0021).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Pass,
    Refuse(Finding),
    RunCheck,
}

/// The git state `pre-commit` decides on.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Commit {
    /// The branch the checkout is on, none on a detached `HEAD`.
    pub branch: Option<String>,
    /// Whether `MERGE_HEAD` names a commit: the commit concludes a merge that stopped.
    pub merging: bool,
    /// Whether the repository has a work tree with `scripts/check-all.sh` as a file in it.
    pub check_script: bool,
}

/// `pre-commit`: refuse a commit on `main`; on any other branch, the commit that concludes a
/// conflicted merge only after the full check, where there is one to run. `on` says which hook
/// rules are turned on.
pub fn pre_commit(state: &Commit, on: &dyn Fn(&HookRule) -> bool) -> Answer {
    if state.branch.as_deref() == Some("main") && on(&COMMIT_ON_MAIN) {
        return Answer::Refuse(Finding {
            rule: COMMIT_ON_MAIN.id,
            file: "refs/heads/main".to_owned(),
            line: None,
            message: "the checkout is on 'main', and changes reach main through a pull request \
                      (AGENTS.md, section 6). Create a branch first: git switch -c <name>"
                .to_owned(),
        });
    }
    if state.merging && state.check_script && on(&MERGE_CHECKS) {
        return Answer::RunCheck;
    }
    Answer::Pass
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(branch: Option<&str>, merging: bool, check_script: bool) -> Commit {
        Commit {
            branch: branch.map(str::to_owned),
            merging,
            check_script,
        }
    }

    fn all(_: &HookRule) -> bool {
        true
    }

    fn none(_: &HookRule) -> bool {
        false
    }

    // The bash case: "a commit on main".
    #[test]
    fn a_commit_on_main_is_refused_at_its_ref() {
        let Answer::Refuse(finding) = pre_commit(&state(Some("main"), false, true), &all) else {
            panic!("a commit on main passed");
        };
        assert_eq!(finding.rule, "hook/commit-on-main");
        assert_eq!(finding.file, "refs/heads/main");
        assert!(finding.message.contains("git switch -c <name>"));
    }

    // The bash cases: "a commit on a branch", "a commit on a detached HEAD is not on main".
    #[test]
    fn a_commit_on_a_branch_or_a_detached_head_passes() {
        assert_eq!(
            pre_commit(&state(Some("feature"), false, true), &all),
            Answer::Pass
        );
        assert_eq!(pre_commit(&state(None, false, true), &all), Answer::Pass);
    }

    // The bash cases: "concluding a conflicted merge …" on a branch and on main.
    #[test]
    fn a_concluded_merge_runs_the_check_on_a_branch_and_is_refused_on_main() {
        assert_eq!(
            pre_commit(&state(Some("feat/x"), true, true), &all),
            Answer::RunCheck
        );
        assert_eq!(pre_commit(&state(None, true, true), &all), Answer::RunCheck);
        assert!(matches!(
            pre_commit(&state(Some("main"), true, true), &all),
            Answer::Refuse(_)
        ));
    }

    #[test]
    fn without_a_check_script_a_concluded_merge_passes() {
        assert_eq!(
            pre_commit(&state(Some("feat/x"), true, false), &all),
            Answer::Pass
        );
    }

    #[test]
    fn rules_turned_off_decide_nothing() {
        assert_eq!(
            pre_commit(&state(Some("main"), true, true), &none),
            Answer::Pass
        );
        let only_main = |rule: &HookRule| rule.id == COMMIT_ON_MAIN.id;
        assert_eq!(
            pre_commit(&state(Some("feat/x"), true, true), &only_main),
            Answer::Pass
        );
    }
}
