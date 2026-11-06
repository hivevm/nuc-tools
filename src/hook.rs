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

/// The refusal of a merge on `main`.
pub const MERGE_ON_MAIN: HookRule = HookRule {
    id: "hook/merge-on-main",
    explain: "changes reach main through a pull request, and the local main only takes up the \
              remote's by a fast-forward (AGENTS.md, section 6); the pre-merge-commit hook \
              refuses a merge while the checkout is on main",
};

/// The documentation check on the tree of a merge into any other branch.
pub const MERGE_DOCS: HookRule = HookRule {
    id: "hook/merge-docs",
    explain: "two branches green on their own collide at a merge mostly in the record, which \
              scripts/check-docs.sh decides in about a second, so the pre-merge-commit hook has \
              it run on the merged tree and refuses the merge while it is red",
};

/// The refusal of a push to the remote's `main`, its deletion included.
pub const PUSH_TO_MAIN: HookRule = HookRule {
    id: "hook/push-to-main",
    explain: "changes reach main through a pull request, not a push (AGENTS.md, section 6); the \
              pre-push hook refuses any push to the remote's main, its deletion included",
};

/// The full check, with `--changed`, before a push.
pub const PUSH_CHECKS: HookRule = HookRule {
    id: "hook/push-checks",
    explain: "the checks pass before anything is pushed (AGENTS.md, section 5), so the pre-push \
              hook has scripts/check-all.sh --changed run before a push that sends a ref, unless \
              they passed before on the clean tree it pushes, and refuses the push while they \
              are red",
};

/// Every hook rule this `nuc` knows, all of them on where the repository has no `nuc.toml`
/// (ADR-0023).
pub const ALL: &[HookRule] = &[
    COMMIT_ON_MAIN,
    MERGE_CHECKS,
    MERGE_ON_MAIN,
    MERGE_DOCS,
    PUSH_TO_MAIN,
    PUSH_CHECKS,
];

/// How many green trees the record keeps, the last first out.
pub const RECORD_LIMIT: usize = 50;

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

/// The git state `pre-commit` and `pre-merge-commit` decide on.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Commit {
    /// The branch the checkout is on, none on a detached `HEAD`.
    pub branch: Option<String>,
    /// Whether `MERGE_HEAD` names a commit: the commit concludes a merge that stopped. Read by
    /// `pre-commit` alone, since every merge `pre-merge-commit` sees has one.
    pub merging: bool,
    /// Whether the repository has a work tree with the hook's check script as a file in it:
    /// `scripts/check-all.sh` for `pre-commit`, `scripts/check-docs.sh` for `pre-merge-commit`.
    pub check_script: bool,
}

/// `pre-merge-commit`: refuse a merge on `main`; on any other branch, the merge only after the
/// documentation check, where there is one to run.
pub fn pre_merge_commit(state: &Commit, on: &dyn Fn(&HookRule) -> bool) -> Answer {
    if state.branch.as_deref() == Some("main") && on(&MERGE_ON_MAIN) {
        return Answer::Refuse(Finding {
            rule: MERGE_ON_MAIN.id,
            file: "refs/heads/main".to_owned(),
            line: None,
            message: "the checkout is on 'main', and changes reach main through a pull request \
                      (AGENTS.md, section 6). git merge --abort, then merge into a feat/ branch, \
                      or take up the remote's main: git merge --ff-only origin/main"
                .to_owned(),
        });
    }
    if state.check_script && on(&MERGE_DOCS) {
        return Answer::RunCheck;
    }
    Answer::Pass
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

/// One line git hands `pre-push` on its standard input: the remote ref, and the local object, none
/// for a deletion, which sends an object of zeros.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushedRef {
    pub local_object: Option<String>,
    pub remote_ref: String,
}

/// The refs of `pre-push`'s standard input, one line each of four fields, local ref, local object,
/// remote ref, remote object, the objects in hex; a line in any other shape is not decided (Q-1).
pub fn pushed_refs(input: &str) -> Result<Vec<PushedRef>, String> {
    let hex = |field: &str| {
        matches!(field.len(), 40 | 64) && field.bytes().all(|b| b.is_ascii_hexdigit())
    };
    input
        .lines()
        .map(|line| match line.split(' ').collect::<Vec<_>>()[..] {
            [local, object, remote, remote_object]
                if !local.is_empty() && hex(object) && !remote.is_empty() && hex(remote_object) =>
            {
                Ok(PushedRef {
                    local_object: (!object.bytes().all(|b| b == b'0')).then(|| object.to_owned()),
                    remote_ref: remote.to_owned(),
                })
            }
            _ => Err(format!(
                "the standard input of pre-push holds a line that is not '<local ref> <local object> \
                 <remote ref> <remote object>': {line:?} — not decided"
            )),
        })
        .collect()
}

/// The git state `pre-push` decides on.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Push {
    /// Each ref the push sends, with the count of the draft ADRs its object carries, none for a
    /// deletion.
    pub refs: Vec<(String, Option<usize>)>,
    /// Whether the draft notice names `CONTRIBUTING.md`, as from template release v0.7.0, rather
    /// than the README.
    pub contributing: bool,
    /// The tree of `HEAD` where the working tree is clean, none otherwise or without a work tree.
    pub tree: Option<String>,
    /// Whether the record of green trees holds that tree.
    pub recorded: bool,
    /// Whether the repository has a work tree with `scripts/check-all.sh` as a file in it.
    pub check_script: bool,
}

/// What `pre-push` answers, and the notices it writes before, which decide nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushAnswer {
    pub notices: Vec<String>,
    pub answer: Answer,
}

/// `pre-push`: refuse a push to the remote's `main`; pass one that only deletes; tell each ref
/// that carries draft ADRs where they take their numbers; pass a clean tree the checks passed on
/// before; otherwise the full check, where there is one to run.
pub fn pre_push(state: &Push, on: &dyn Fn(&HookRule) -> bool) -> PushAnswer {
    let mut notices = Vec::new();
    let guide = if state.contributing {
        "CONTRIBUTING.md"
    } else {
        "README"
    };
    for (remote_ref, drafts) in &state.refs {
        if remote_ref == "refs/heads/main" && on(&PUSH_TO_MAIN) {
            let answer = Answer::Refuse(Finding {
                rule: PUSH_TO_MAIN.id,
                file: "refs/heads/main".to_owned(),
                line: None,
                message: "changes reach main through a pull request, not a push (AGENTS.md, \
                          section 6). Push a branch of its own and open one: gh pr create --base \
                          main"
                    .to_owned(),
            });
            return PushAnswer { notices, answer };
        }
        if let Some(drafts @ 1..) = drafts {
            let branch = remote_ref.strip_prefix("refs/heads/").unwrap_or(remote_ref);
            notices.push(format!(
                "pre-push: {branch} carries {drafts} draft ADR(s); they take their numbers on the \
                 branch of the pull request into main, by scripts/ready-for-main.sh ({guide}, \
                 Integration branches, step 7)."
            ));
        }
    }
    let answer = if !state.refs.is_empty() && state.refs.iter().all(|(_, d)| d.is_none()) {
        Answer::Pass
    } else if state.tree.is_some() && state.recorded {
        notices.push("pre-push: the checks passed on this tree before; not run again.".to_owned());
        Answer::Pass
    } else if state.check_script && on(&PUSH_CHECKS) {
        Answer::RunCheck
    } else {
        Answer::Pass
    };
    PushAnswer { notices, answer }
}

/// The record of green trees with `tree` added last: the other trees it holds, the last
/// [`RECORD_LIMIT`] - 1 of them, then the tree, one per line.
pub fn record(existing: &str, tree: &str) -> String {
    let kept: Vec<&str> = existing.lines().filter(|line| *line != tree).collect();
    let start = kept.len().saturating_sub(RECORD_LIMIT - 1);
    let mut out: String = kept[start..]
        .iter()
        .map(|line| format!("{line}\n"))
        .collect();
    out.push_str(tree);
    out.push('\n');
    out
}

/// Whether the record of green trees holds `tree` as a line of its own.
pub fn holds(record: &str, tree: &str) -> bool {
    record.lines().any(|line| line == tree)
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

    // The bash cases: "a merge on main, though the checks are green", "a merge into a branch while
    // the documentation check is red".
    #[test]
    fn a_merge_on_main_is_refused_and_one_elsewhere_runs_the_documentation_check() {
        let Answer::Refuse(finding) = pre_merge_commit(&state(Some("main"), false, true), &all)
        else {
            panic!("a merge on main passed");
        };
        assert_eq!(finding.rule, "hook/merge-on-main");
        assert_eq!(finding.file, "refs/heads/main");
        assert_eq!(
            pre_merge_commit(&state(Some("feat/x"), false, true), &all),
            Answer::RunCheck
        );
        assert_eq!(
            pre_merge_commit(&state(None, false, true), &all),
            Answer::RunCheck
        );
        assert_eq!(
            pre_merge_commit(&state(Some("feat/x"), false, false), &all),
            Answer::Pass
        );
        assert_eq!(
            pre_merge_commit(&state(Some("main"), false, true), &none),
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

    const ZERO: &str = "0000000000000000000000000000000000000000";
    const ONE: &str = "1111111111111111111111111111111111111111";

    fn push(refs: &[(&str, Option<usize>)]) -> Push {
        Push {
            refs: refs.iter().map(|(r, d)| ((*r).to_owned(), *d)).collect(),
            contributing: true,
            tree: Some("t".to_owned()),
            recorded: false,
            check_script: true,
        }
    }

    #[test]
    fn the_standard_input_reads_four_fields_and_refuses_any_other_shape() {
        let input =
            format!("refs/heads/a {ONE} refs/heads/a {ZERO}\n(delete) {ZERO} refs/heads/b {ONE}\n");
        let refs = pushed_refs(&input).unwrap();
        assert_eq!(refs[0].local_object.as_deref(), Some(ONE));
        assert_eq!(refs[1].local_object, None);
        assert_eq!(refs[1].remote_ref, "refs/heads/b");
        assert_eq!(pushed_refs("").unwrap(), []);
        for bad in [
            "x",
            "a b c d",
            &format!("a {ONE} refs/heads/a"),
            &format!("a {ONE}  {ZERO}"),
        ] {
            assert!(pushed_refs(bad).is_err(), "{bad:?}");
        }
    }

    // The bash cases: "a push to the remote's main", "a deletion of the remote's main while the
    // checks are red".
    #[test]
    fn a_push_to_main_is_refused_a_deletion_included() {
        for drafts in [Some(0), None] {
            let out = pre_push(&push(&[("refs/heads/main", drafts)]), &all);
            let Answer::Refuse(finding) = out.answer else {
                panic!("a push to main passed");
            };
            assert_eq!(finding.rule, "hook/push-to-main");
            assert_eq!(finding.file, "refs/heads/main");
        }
    }

    // The bash cases: "a deletion of a branch while the checks are red", "a deletion together with
    // a push of a branch is checked".
    #[test]
    fn a_push_that_only_deletes_passes_and_one_that_sends_runs_the_checks() {
        assert_eq!(
            pre_push(&push(&[("refs/heads/a", None)]), &all).answer,
            Answer::Pass
        );
        assert_eq!(
            pre_push(
                &push(&[("refs/heads/a", None), ("refs/heads/b", Some(0))]),
                &all
            )
            .answer,
            Answer::RunCheck
        );
        assert_eq!(pre_push(&push(&[]), &all).answer, Answer::RunCheck);
    }

    #[test]
    fn the_draft_notice_counts_per_ref_in_the_wording_of_the_release() {
        let mut state = push(&[
            ("refs/heads/feature", Some(2)),
            ("refs/heads/main", Some(1)),
        ]);
        let out = pre_push(&state, &all);
        assert_eq!(
            out.notices.len(),
            1,
            "a notice after the refusal: {:?}",
            out.notices
        );
        assert!(out.notices[0].starts_with("pre-push: feature carries 2 draft ADR(s);"));
        assert!(out.notices[0].ends_with("(CONTRIBUTING.md, Integration branches, step 7)."));
        state.contributing = false;
        state.refs.pop();
        let out = pre_push(&state, &all);
        assert!(out.notices[0].ends_with("(README, Integration branches, step 7)."));
    }

    // The bash cases: "a reworked branch on a tree that passed before …", "the same tree with an
    // untracked file is checked again", "a new tree is checked again".
    #[test]
    fn a_recorded_clean_tree_skips_the_checks() {
        let mut state = push(&[("refs/heads/a", Some(0))]);
        state.recorded = true;
        let out = pre_push(&state, &none);
        assert_eq!(out.answer, Answer::Pass);
        assert_eq!(
            out.notices,
            ["pre-push: the checks passed on this tree before; not run again."]
        );
        state.tree = None;
        assert_eq!(pre_push(&state, &all).answer, Answer::RunCheck);
        state.check_script = false;
        assert_eq!(pre_push(&state, &all).answer, Answer::Pass);
    }

    #[test]
    fn the_record_keeps_the_last_fifty_and_the_tree_once_last() {
        assert_eq!(record("", "t"), "t\n");
        assert_eq!(record("a\nt\nb\n", "t"), "a\nb\nt\n");
        let many: String = (0..60).map(|n| format!("{n}\n")).collect();
        let out = record(&many, "t");
        assert_eq!(out.lines().count(), 50);
        assert_eq!(out.lines().next(), Some("11"));
        assert!(holds(&out, "t") && !holds(&out, "1") && !holds("tt\n", "t"));
    }
}
