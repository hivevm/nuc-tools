//! The rule `devcontainer/lock` of the template's `scripts/check-devcontainer.sh`: every Feature of
//! `.devcontainer/devcontainer.json` is referenced by its major version tag alone, and the lock
//! `.devcontainer/devcontainer-lock.json` exists and names exactly those Features
//! (NUC-container-hides-host-daemon §3). A Feature that stands twice is reported twice, as the bash
//! reports it.

use super::{Input, Inputs, Outcome, Rule};
use crate::devcontainer::{self, CONFIG, LOCK};
use crate::finding::Finding;

pub const RULE_ID: &str = "devcontainer/lock";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["tag", "absent", "missing", "stale"],
    explain: "Every Feature of .devcontainer/devcontainer.json is referenced by its major version \
              tag alone ('<ref>:N'), never untagged, ':latest', a full version, or a digest; and \
              .devcontainer/devcontainer-lock.json is committed and names exactly those Features, \
              so that the same commit yields the same Features. Regenerate the lock by \
              rebuilding the container.",
    reads: &[Input::Devcontainer, Input::DevcontainerLock, Input::Release],
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

/// Whether the id ends in a colon and one digit or more, as `[[ $id =~ :[0-9]+$ ]]` matches it.
fn major_tag(id: &[u8]) -> bool {
    let digits = id.iter().rev().take_while(|b| b.is_ascii_digit()).count();
    digits > 0 && id.len() > digits && id[id.len() - digits - 1] == b':'
}

fn check(inputs: &Inputs) -> Outcome {
    let Some(config) = inputs.devcontainer.config.as_deref() else {
        return Outcome::default();
    };
    if !devcontainer::binds(inputs) {
        return Outcome::default();
    }
    let cited = devcontainer::cite(inputs, 3);
    let features = devcontainer::feature_ids(config);
    let mut findings = Vec::new();
    for id in features.iter().filter(|id| !major_tag(id)) {
        findings.push(finding(
            "tag",
            CONFIG,
            format!(
                "Feature '{}' is not pinned by its major version tag alone ('<ref>:N') — an \
                 untagged, mutable, or fully versioned reference is not what the lock resolves \
                 {cited}",
                devcontainer::spelled(id)
            ),
        ));
    }
    let Some(lock) = inputs.devcontainer.lock.as_deref() else {
        findings.push(finding(
            "absent",
            LOCK,
            format!(
                "not found — the lock is committed so that the same commit yields the same \
                 Features {cited}"
            ),
        ));
        return findings.into();
    };
    let locked = devcontainer::feature_ids(lock);
    for id in features.iter().filter(|id| !locked.contains(id)) {
        findings.push(finding(
            "missing",
            LOCK,
            format!(
                "has no entry for Feature '{}' — regenerate the lock (rebuild the container) so \
                 the Feature builds from a resolved digest {cited}",
                devcontainer::spelled(id)
            ),
        ));
    }
    for id in locked.iter().filter(|id| !features.contains(id)) {
        findings.push(finding(
            "stale",
            LOCK,
            format!(
                "lists Feature '{}', which {CONFIG} no longer adds — the lock was not regenerated \
                 {cited}",
                devcontainer::spelled(id)
            ),
        ));
    }
    findings.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Version;
    use crate::devcontainer::fixture::{FEATURE, NUC, Tree};

    fn keys(tree: &Tree) -> Vec<String> {
        check(&tree.inputs())
            .findings
            .into_iter()
            .map(|f| format!("{}: {} [{}]", f.file, f.message, f.reason))
            .collect()
    }

    /// The base with the first Feature replaced by `id`, and the lock resolving `locked`.
    fn with(id: &str, locked: &[&str]) -> Tree {
        let mut tree = Tree::base();
        tree.feature = id.to_owned();
        tree.config().lock(locked);
        tree
    }

    // bash: a container without a socket, pinned and locked, passes
    #[test]
    fn a_feature_on_its_major_tag_alone_is_pinned() {
        // check-devcontainer.txt: a container without a socket, pinned and locked, passes
        assert!(keys(&Tree::base()).is_empty());
        for id in ["a:1", "x:y:20", "a@sha256:00"] {
            assert!(major_tag(id.as_bytes()), "{id}");
        }
        for id in ["a", "a:", "a:1.2.0", "a:latest", "a@sha256:0a", "1", ":1x"] {
            assert!(!major_tag(id.as_bytes()), "{id}");
        }
    }

    // bash: a Feature on a mutable tag
    // bash: a Feature without a tag
    // bash: a Feature on a full version instead of the major tag
    #[test]
    fn a_feature_on_a_mutable_tag_untagged_or_on_a_full_version_is_reported() {
        // check-devcontainer.txt: a Feature on a mutable tag
        // check-devcontainer.txt: a Feature without a tag
        // check-devcontainer.txt: a Feature on a full version instead of the major tag
        for id in [
            "ghcr.io/devcontainers/features/github-cli:latest",
            "ghcr.io/devcontainers/features/github-cli",
            "ghcr.io/devcontainers/features/github-cli:1.2.0",
        ] {
            assert_eq!(
                keys(&with(id, &[id, NUC])),
                [format!(
                    ".devcontainer/devcontainer.json: Feature '{id}' is not pinned by its major \
                     version tag alone ('<ref>:N') — an untagged, mutable, or fully versioned \
                     reference is not what the lock resolves (NUC-container-hides-host-daemon \
                     \u{a7}3) [tag]"
                )]
            );
        }
        // check-devcontainer.txt: the nuc Feature on a full version instead of the major tag
        let mut tree = Tree::base();
        tree.nuc = Some("ghcr.io/hivevm/features/nuc:0.2.2".to_owned());
        tree.config()
            .lock(&[FEATURE, "ghcr.io/hivevm/features/nuc:0.2.2"]);
        assert_eq!(keys(&tree).len(), 1);
        assert!(keys(&tree)[0].contains("'ghcr.io/hivevm/features/nuc:0.2.2'"));
    }

    // bash: no lock committed
    // bash: a Feature the lock does not resolve
    // bash: a lock entry for a Feature no longer added
    #[test]
    fn the_lock_is_absent_or_lacks_a_feature_or_lists_one_no_longer_added() {
        // check-devcontainer.txt: no lock committed
        let mut tree = Tree::base();
        tree.files.lock = None;
        assert_eq!(
            keys(&tree),
            [
                ".devcontainer/devcontainer-lock.json: not found — the lock is committed so that \
              the same commit yields the same Features (NUC-container-hides-host-daemon \u{a7}3) \
              [absent]"
            ]
        );
        // check-devcontainer.txt: a Feature the lock does not resolve
        let node = "ghcr.io/devcontainers/features/node:1";
        assert_eq!(
            keys(&with(node, &[FEATURE, NUC])),
            [
                format!(
                    ".devcontainer/devcontainer-lock.json: has no entry for Feature '{node}' — \
                     regenerate the lock (rebuild the container) so the Feature builds from a \
                     resolved digest (NUC-container-hides-host-daemon \u{a7}3) [missing]"
                ),
                format!(
                    ".devcontainer/devcontainer-lock.json: lists Feature '{FEATURE}', which \
                     .devcontainer/devcontainer.json no longer adds — the lock was not \
                     regenerated (NUC-container-hides-host-daemon \u{a7}3) [stale]"
                ),
            ]
        );
        // check-devcontainer.txt: a lock entry for a Feature no longer added
        let mut tree = Tree::base();
        tree.lock(&[FEATURE, node, NUC]);
        assert_eq!(keys(&tree).len(), 1);
        assert!(keys(&tree)[0].ends_with("[stale]"));
    }

    #[test]
    fn a_release_before_v0_9_0_cites_adr_0004_and_a_feature_twice_is_reported_twice() {
        let mut tree = with("a", &[NUC]);
        tree.release = Some(Version(0, 5, 0));
        tree.extras.clear();
        tree.nuc = Some("a".to_owned());
        tree.config();
        let found = keys(&tree);
        let adr = crate::testing::adr_id(4);
        assert_eq!(found.len(), 5, "{found:?}");
        assert!(found[0].ends_with(&format!("({adr}) [tag]")), "{found:?}");
        assert_eq!(found[0], found[1]);
        assert_eq!(found[2], found[3]);
        assert!(found[4].ends_with("[stale]"));
    }
}
