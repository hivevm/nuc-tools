//! The rule `hooks/pin` of the template's `scripts/check-hooks.sh` from v0.12.0 on: the blob list
//! `.githooks/nuc.blobs` exists and names a release, it pins each of the five hooks and no other
//! file, each line is an entry, and each hook it pins is a file whose blob, as git would store it,
//! is the one listed (NUC-hooks-run-nuc §2). A tree of an earlier release has no such check.

use super::{HOOK_PINS, Input, Inputs, Outcome, Rule};
use crate::finding::Finding;
use crate::pin::{LIST, PINNED_HOOKS, Wrong};

pub const RULE_ID: &str = "hooks/pin";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &[
        "absent", "entry", "foreign", "release", "unlisted", "missing", "differs",
    ],
    explain: "The git hooks under .githooks/ are the shims of a release of hivevm/nuc-tools, \
              unchanged, and the blob list .githooks/nuc.blobs pins them: it names the release, \
              the blob of each of the five hooks, and no other file, and each line is 'release \
              v<X.Y.Z>', '<blob> <hook>', 'rule <rule ID>', a comment, or empty. Each hook it \
              pins is a file whose blob, as git would store it with its line endings converted, \
              is the one listed. A hook edited by hand would decide unseen. From template release \
              v0.12.0 on.",
    reads: &[Input::HookList, Input::HookBlobs, Input::Release],
    check,
};

fn finding(reason: &'static str, file: &str, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        reason,
        file: file.to_owned(),
        line: None,
        message,
    }
}

fn check(inputs: &Inputs) -> Outcome {
    if !inputs.carries_at_least(HOOK_PINS) {
        return Outcome::default();
    }
    let Some(bytes) = &inputs.hooks.list else {
        return vec![finding(
            "absent",
            LIST,
            "not found — the hooks are pinned by their blobs beside them (NUC-hooks-run-nuc §2)"
                .to_owned(),
        )]
        .into();
    };
    let list = crate::pin::read_list(bytes);
    let mut findings: Vec<Finding> = list
        .wrong
        .iter()
        .map(|wrong| match wrong {
            Wrong::Foreign(name) => finding(
                "foreign",
                LIST,
                format!(
                    "pins '{name}', which is none of the five hooks — a file of the project's own \
                     is held by the checks, not by a pin (NUC-hooks-run-nuc §2)"
                ),
            ),
            Wrong::Entry(line) => finding(
                "entry",
                LIST,
                format!(
                    "'{line}' is no entry — write 'release v<X.Y.Z>', '<blob> <hook>', or 'rule \
                     <rule ID>' (NUC-hooks-run-nuc §2)"
                ),
            ),
        })
        .collect();
    if list.release.is_none() {
        findings.push(finding(
            "release",
            LIST,
            "names no release — the hooks come from one release of hivevm/nuc-tools \
             (NUC-hooks-run-nuc §2)"
                .to_owned(),
        ));
    }
    for hook in PINNED_HOOKS {
        if !list.listed.contains_key(hook) {
            findings.push(finding(
                "unlisted",
                LIST,
                format!(
                    "names no blob for the hook '{hook}' — every hook is pinned \
                     (NUC-hooks-run-nuc §2)"
                ),
            ));
        }
    }
    let release = list.release.as_deref().unwrap_or("");
    for (hook, listed) in &list.listed {
        let file = format!(".githooks/{hook}");
        match inputs.hooks.blobs.get(*hook) {
            None => findings.push(finding(
                "missing",
                &file,
                format!("not found, though {LIST} pins it (NUC-hooks-run-nuc §2)"),
            )),
            Some(None) => findings.push(finding(
                "missing",
                &file,
                "git cannot hash it (NUC-hooks-run-nuc §2)".to_owned(),
            )),
            Some(Some(blob)) if blob != listed => findings.push(finding(
                "differs",
                &file,
                format!(
                    "differs from the shim of nuc-tools v{release} that {LIST} pins — a hook is \
                     the release's file unchanged; take it again from shims/ of that release \
                     (NUC-hooks-run-nuc §1, §2)"
                ),
            )),
            Some(Some(_)) => {}
        }
    }
    findings.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Version;
    use std::collections::BTreeMap;

    const BLOB: &str = "2b2030214e355020c34a4a7ceb0ea76ffb273e99";

    /// A blob list of the release that pins each hook by [`BLOB`], with the lines given after it.
    fn list(more: &str) -> String {
        let mut text = String::from("# The pin\nrelease v0.2.2\n");
        for hook in PINNED_HOOKS {
            text.push_str(&format!("{BLOB} {hook}\n"));
        }
        text.push_str("rule hook/push-checks\n");
        text + more
    }

    /// Each hook a file of the blob [`BLOB`].
    fn blobs() -> BTreeMap<String, Option<String>> {
        PINNED_HOOKS
            .iter()
            .map(|hook| ((*hook).to_owned(), Some(BLOB.to_owned())))
            .collect()
    }

    fn keys(list: Option<&str>, blobs: BTreeMap<String, Option<String>>) -> Vec<String> {
        let inputs = Inputs {
            hooks: crate::pin::Hooks {
                list: list.map(|text| text.as_bytes().to_vec()),
                blobs,
            },
            ..Inputs::default()
        };
        check(&inputs)
            .findings
            .into_iter()
            .map(|f| format!("{}: {} [{}]", f.file, f.message, f.reason))
            .collect()
    }

    #[test]
    fn the_hooks_their_blob_list_and_a_tree_before_v0_12_0_pass() {
        // test-check-hooks.sh: the repository's hooks, blob list, and nuc.toml pass
        assert!(keys(Some(&list("")), blobs()).is_empty());
        let inputs = Inputs {
            release: Some(Version(0, 11, 1)),
            ..Inputs::default()
        };
        assert_eq!(check(&inputs), Outcome::default());
    }

    #[test]
    fn a_hook_edited_by_hand_differs_and_names_the_hook() {
        // bash: a hook edited by hand
        let mut edited = blobs();
        edited.insert("pre-push".to_owned(), Some("1".repeat(40)));
        assert_eq!(
            keys(Some(&list("")), edited),
            [
                ".githooks/pre-push: differs from the shim of nuc-tools v0.2.2 that \
              .githooks/nuc.blobs pins — a hook is the release's file unchanged; take it again \
              from shims/ of that release (NUC-hooks-run-nuc §1, §2) [differs]"
            ]
        );
    }

    #[test]
    fn a_hook_the_list_does_not_pin_is_unlisted() {
        // bash: a hook the list does not pin
        let text = list("").replace(&format!("{BLOB} pre-commit\n"), "");
        assert_eq!(
            keys(Some(&text), blobs()),
            [
                ".githooks/nuc.blobs: names no blob for the hook 'pre-commit' — every hook is \
              pinned (NUC-hooks-run-nuc §2) [unlisted]"
            ]
        );
    }

    #[test]
    fn a_hook_the_list_pins_and_the_tree_lacks_or_git_cannot_hash_is_missing() {
        // bash: a hook the list pins and the tree lacks
        let mut lacking = blobs();
        lacking.remove("reference-transaction");
        lacking.insert("pre-commit".to_owned(), None);
        assert_eq!(
            keys(Some(&list("")), lacking),
            [
                ".githooks/pre-commit: git cannot hash it (NUC-hooks-run-nuc §2) [missing]",
                ".githooks/reference-transaction: not found, though .githooks/nuc.blobs pins it \
                 (NUC-hooks-run-nuc §2) [missing]",
            ]
        );
    }

    #[test]
    fn a_list_without_a_release_names_none_in_the_message_of_a_hook_that_differs() {
        // bash: a list without a release
        let text = list("").replace("release v0.2.2\n", "");
        let mut edited = blobs();
        edited.insert("pre-push".to_owned(), Some("1".repeat(40)));
        let found = keys(Some(&text), edited);
        assert_eq!(
            found[0],
            ".githooks/nuc.blobs: names no release — the hooks come from one release of \
             hivevm/nuc-tools (NUC-hooks-run-nuc §2) [release]"
        );
        assert!(found[1].contains("of nuc-tools v that"), "{found:?}");
    }

    #[test]
    fn a_line_the_list_cannot_read_is_no_entry() {
        // bash: a line the list cannot read
        assert_eq!(
            keys(Some(&list("pre-push 1234\n")), blobs()),
            [
                ".githooks/nuc.blobs: 'pre-push 1234' is no entry — write 'release v<X.Y.Z>', \
              '<blob> <hook>', or 'rule <rule ID>' (NUC-hooks-run-nuc §2) [entry]"
            ]
        );
    }

    #[test]
    fn a_list_that_pins_another_file_than_the_five_hooks_is_foreign() {
        // bash: a list that pins a file of the project's own, though its blob holds
        // bash: a list that pins a path outside the hooks
        for name in ["own", "../pre-push"] {
            assert_eq!(
                keys(Some(&list(&format!("{BLOB} {name}\n"))), blobs()),
                [format!(
                    ".githooks/nuc.blobs: pins '{name}', which is none of the five hooks — a \
                     file of the project's own is held by the checks, not by a pin \
                     (NUC-hooks-run-nuc §2) [foreign]"
                )]
            );
        }
    }

    #[test]
    fn no_blob_list_is_absent_and_nothing_else() {
        // bash: no blob list
        assert_eq!(
            keys(None, BTreeMap::new()),
            [
                ".githooks/nuc.blobs: not found — the hooks are pinned by their blobs beside them \
              (NUC-hooks-run-nuc §2) [absent]"
            ]
        );
    }

    #[test]
    fn a_list_with_crlf_line_endings_and_no_last_newline_reads_as_with_lf() {
        let crlf = list("").replace('\n', "\r\n");
        let crlf = crlf.strip_suffix("\r\n").expect("a last line");
        assert!(keys(Some(crlf), blobs()).is_empty());
    }
}
