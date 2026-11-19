//! The rule `devcontainer/git-settings` of the template's `scripts/check-devcontainer.sh`: the
//! postStartCommand of `.devcontainer/devcontainer.json` runs `git config` for the git hooks and
//! the merge settings they rely on (NUC-container-hides-host-daemon §6). The value is the text from
//! its top-level key to the next one, whatever its form, its tabs and newlines made spaces and its
//! runs of spaces squeezed.

use super::{CATEGORY_IDENTIFIERS, CONTRIBUTING_GUIDE, Input, Inputs, Outcome, Rule};
use crate::devcontainer::{self, CONFIG};
use crate::finding::Finding;

pub const RULE_ID: &str = "devcontainer/git-settings";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["absent"],
    explain: "The postStartCommand of .devcontainer/devcontainer.json runs 'git config \
              core.hooksPath .githooks', 'git config merge.ff false', 'git config pull.ff only', \
              and 'git config merge.adr-index.driver' with the driver's command as \
              CONTRIBUTING.md writes it, as a string, an array, or an object: the git hooks and \
              the merge settings they rely on are enabled on every start.",
    reads: &[Input::Devcontainer, Input::Release],
    check,
};

/// The settings, each as `git config` takes it.
const SETTINGS: [&str; 4] = [
    "core.hooksPath .githooks",
    "merge.ff false",
    "pull.ff only",
    "merge.adr-index.driver '.githooks/merge-adr-index %O %A %B %S %X %Y'",
];

/// The value as `tr '\t\n' '  ' | tr -s ' '` writes it.
fn collapsed(value: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(value.len());
    for &b in value {
        let b = if matches!(b, b'\t' | b'\n') { b' ' } else { b };
        if !(b == b' ' && out.last() == Some(&b' ')) {
            out.push(b);
        }
    }
    out
}

fn check(inputs: &Inputs) -> Outcome {
    let Some(config) = inputs.devcontainer.config.as_deref() else {
        return Outcome::default();
    };
    if !devcontainer::binds(inputs) {
        return Outcome::default();
    }
    let cited = if inputs.carries_at_least(CATEGORY_IDENTIFIERS) {
        devcontainer::cite(inputs, 6)
    } else if inputs.carries_at_least(CONTRIBUTING_GUIDE) {
        "(CONTRIBUTING.md, Git hooks)".to_owned()
    } else {
        "(README, Git hooks)".to_owned()
    };
    let post = collapsed(&devcontainer::top_level_value(config, b"postStartCommand"));
    SETTINGS
        .iter()
        .filter(|setting| !crate::bash::contains(&post, format!("git config {setting}").as_bytes()))
        .map(|setting| Finding {
            rule: RULE_ID,
            reason: "absent",
            file: CONFIG.to_owned(),
            line: None,
            message: format!(
                "postStartCommand does not run 'git config {setting}' — the git hooks and the \
                 merge settings they rely on are enabled on every start {cited}"
            ),
        })
        .collect::<Vec<_>>()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Version;
    use crate::devcontainer::fixture::{POST, Tree};

    fn keys(tree: &Tree) -> Vec<String> {
        check(&tree.inputs())
            .findings
            .into_iter()
            .map(|f| format!("{}: {} [{}]", f.file, f.message, f.reason))
            .collect()
    }

    fn missing(tree: &Tree) -> Vec<&'static str> {
        let found = keys(tree);
        SETTINGS
            .iter()
            .copied()
            .filter(|s| {
                found
                    .iter()
                    .any(|f| f.contains(&format!("'git config {s}'")))
            })
            .collect()
    }

    fn with_post(post: Option<&str>) -> Tree {
        let mut tree = Tree::base();
        tree.post = post.map(str::to_owned);
        tree.config();
        tree
    }

    // bash: no postStartCommand at all
    #[test]
    fn no_post_start_command_lacks_every_setting() {
        // check-devcontainer.txt: no postStartCommand at all
        assert!(keys(&Tree::base()).is_empty());
        let tree = with_post(None);
        assert_eq!(missing(&tree), SETTINGS);
        assert_eq!(
            keys(&tree)[0],
            ".devcontainer/devcontainer.json: postStartCommand does not run 'git config \
             core.hooksPath .githooks' — the git hooks and the merge settings they rely on are \
             enabled on every start (NUC-container-hides-host-daemon \u{a7}6) [absent]"
        );
    }

    // bash: the git settings in a postStartCommand string pass
    // bash: a brace or bracket in an entry before the git settings
    #[test]
    fn the_settings_pass_as_a_string_and_with_braces_before_them() {
        // check-devcontainer.txt: the git settings in a postStartCommand string pass
        assert!(
            keys(&with_post(Some(
                "\"git config core.hooksPath .githooks && git config merge.ff false && git \
                 config pull.ff only && git config merge.adr-index.driver \
                 '.githooks/merge-adr-index %O %A %B %S %X %Y'\""
            )))
            .is_empty()
        );
        // check-devcontainer.txt: a brace or bracket in an entry before the git settings
        let post = POST.replacen(
            "{ ",
            "{ \"cd\": \"cd ${containerWorkspaceFolder}\", \"list\": [\"a]b\"], ",
            1,
        );
        assert!(keys(&with_post(Some(&post))).is_empty());
        // Written over lines, its tabs and runs of spaces collapse.
        let post = POST.replace("git config pull.ff only", "git\tconfig\n   pull.ff only");
        assert!(keys(&with_post(Some(&post))).is_empty());
    }

    // bash: a postStartCommand without 'git config $setting'
    // bash: the merge driver without the arguments git passes it
    #[test]
    fn each_setting_left_out_is_reported_alone() {
        // check-devcontainer.txt: a postStartCommand without 'git config core.hooksPath .githooks'
        // check-devcontainer.txt: a postStartCommand without 'git config merge.ff false'
        // check-devcontainer.txt: a postStartCommand without 'git config pull.ff only'
        // check-devcontainer.txt: a postStartCommand without 'git config
        // merge.adr-index.driver ...'
        // check-devcontainer.txt: the merge driver without the arguments git passes it
        let cases = [
            ("git config core.hooksPath .githooks", "", SETTINGS[0]),
            ("git config merge.ff false && ", " && ", SETTINGS[1]),
            (" && git config pull.ff only", " && ", SETTINGS[2]),
            ("git config merge.adr-index.driver '", "'", SETTINGS[3]),
            (" %O %A %B %S %X %Y", "", SETTINGS[3]),
        ];
        for (cut, put, setting) in cases {
            let post = POST.replacen(cut, put, 1);
            assert_eq!(missing(&with_post(Some(&post))), [setting], "{post}");
        }
    }

    // bash: the git settings in another command than postStartCommand
    #[test]
    fn the_settings_in_another_command_than_post_start_command_are_absent() {
        // check-devcontainer.txt: the git settings in another command than postStartCommand
        let mut tree = with_post(Some("\"echo started\""));
        tree.extra(
            "\"postCreateCommand\": \"git config core.hooksPath .githooks && git config merge.ff \
             false && git config pull.ff only\",",
        );
        assert_eq!(missing(&tree), SETTINGS);
    }

    #[test]
    fn a_release_before_v0_9_0_cites_the_guide_of_its_git_hooks() {
        let mut tree = with_post(None);
        tree.release = Some(Version(0, 8, 1));
        assert!(keys(&tree)[0].ends_with("on every start (CONTRIBUTING.md, Git hooks) [absent]"));
        tree.release = Some(Version(0, 6, 2));
        assert!(keys(&tree)[0].ends_with("on every start (README, Git hooks) [absent]"));
        assert_eq!(collapsed(b"a\t\n  b  "), b"a b ");
    }
}
