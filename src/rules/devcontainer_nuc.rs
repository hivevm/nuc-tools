//! The rule `devcontainer/nuc` of the template's `scripts/check-devcontainer.sh` from release
//! v0.12.0 on: `.devcontainer/devcontainer.json` adds the Feature `ghcr.io/hivevm/features/nuc`,
//! on any tag, which installs the `nuc` the git hooks hand their work to (NUC-hooks-run-nuc §4).
//! `devcontainer/lock` holds its tag and its lock as any Feature's. A tree of an earlier release
//! has no such check.

use super::{HOOK_PINS, Input, Inputs, Outcome, Rule};
use crate::devcontainer::{self, CONFIG};
use crate::finding::Finding;

pub const RULE_ID: &str = "devcontainer/nuc";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["absent"],
    explain: "The Dev Container adds the Feature ghcr.io/hivevm/features/nuc, on any tag: the git \
              hooks hand their work to nuc, and a hook without it refuses. \
              devcontainer/lock holds its tag and its lock. From template release v0.12.0 on.",
    reads: &[Input::Devcontainer, Input::Release],
    check,
};

/// The Feature that installs `nuc`, without its tag.
const FEATURE: &str = "ghcr.io/hivevm/features/nuc";

fn check(inputs: &Inputs) -> Outcome {
    let Some(config) = inputs.devcontainer.config.as_deref() else {
        return Outcome::default();
    };
    if !devcontainer::binds(inputs) || !inputs.carries_at_least(HOOK_PINS) {
        return Outcome::default();
    }
    let tagged = format!("{FEATURE}:");
    let added = devcontainer::feature_ids(config)
        .iter()
        .any(|id| id == FEATURE.as_bytes() || id.starts_with(tagged.as_bytes()));
    if added {
        return Outcome::default();
    }
    vec![Finding {
        rule: RULE_ID,
        reason: "absent",
        file: CONFIG.to_owned(),
        line: None,
        message: format!(
            "adds no Feature '{FEATURE}' — the git hooks hand their work to nuc, and a hook \
             without it refuses (NUC-hooks-run-nuc \u{a7}4)"
        ),
    }]
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Version;
    use crate::devcontainer::fixture::Tree;

    fn keys(tree: &Tree) -> Vec<String> {
        check(&tree.inputs())
            .findings
            .into_iter()
            .map(|f| format!("{}: {} [{}]", f.file, f.message, f.reason))
            .collect()
    }

    #[test]
    fn a_dev_container_without_the_nuc_feature_is_absent_from_v0_12_0_on() {
        // check-devcontainer.txt: a Dev Container without the nuc Feature the git hooks need
        let mut tree = Tree::base();
        assert!(keys(&tree).is_empty());
        tree.nuc = None;
        tree.config();
        assert_eq!(
            keys(&tree),
            [
                ".devcontainer/devcontainer.json: adds no Feature 'ghcr.io/hivevm/features/nuc' — \
              the git hooks hand their work to nuc, and a hook without it refuses \
              (NUC-hooks-run-nuc \u{a7}4) [absent]"
            ]
        );
        tree.release = Some(Version(0, 11, 1));
        assert!(keys(&tree).is_empty());
    }

    #[test]
    fn the_nuc_feature_on_any_tag_or_none_counts_as_added() {
        // check-devcontainer.txt: the nuc Feature on another major tag
        // check-devcontainer.txt: the nuc Feature on a full version still counts as added
        for id in [
            "ghcr.io/hivevm/features/nuc:1",
            "ghcr.io/hivevm/features/nuc:0.2.2",
            "ghcr.io/hivevm/features/nuc",
        ] {
            let mut tree = Tree::base();
            tree.nuc = Some(id.to_owned());
            tree.config();
            assert!(keys(&tree).is_empty(), "{id}");
        }
        let mut tree = Tree::base();
        tree.nuc = Some("ghcr.io/hivevm/features/nucleus:1".to_owned());
        tree.config();
        assert_eq!(keys(&tree).len(), 1);
    }
}
