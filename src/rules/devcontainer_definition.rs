//! The rule `devcontainer/definition` of the template's `scripts/check-devcontainer.sh`: a tree the
//! Dev Container binds has its definition `.devcontainer/devcontainer.json`
//! (NUC-container-hides-host-daemon §1). From release v0.9.0 on, a tree without `.devcontainer/`
//! has no Dev Container, and nothing binds it; before, every tree is bound.

use super::{CATEGORY_IDENTIFIERS, Input, Inputs, Outcome, Rule};
use crate::devcontainer::{self, CONFIG};
use crate::finding::Finding;

pub const RULE_ID: &str = "devcontainer/definition";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["absent"],
    explain: "A tree with .devcontainer/ has its definition .devcontainer/devcontainer.json, \
              which the other devcontainer/ rules read: a Dev Container without it is not \
              governed. A tree without .devcontainer/ has no Dev Container, and nothing binds it; \
              before template release v0.9.0, every tree is held to the definition.",
    reads: &[Input::Devcontainer, Input::Release],
    check,
};

fn check(inputs: &Inputs) -> Outcome {
    if !devcontainer::binds(inputs) || inputs.devcontainer.config.is_some() {
        return Outcome::default();
    }
    let message = if inputs.carries_at_least(CATEGORY_IDENTIFIERS) {
        format!(
            "not found — a .devcontainer/ without its definition {}",
            devcontainer::cite(inputs, 1)
        )
    } else {
        // The bash writes the ADR inline here, not as a citation group, so `uncited` of the shell
        // cannot strip it: under a README with no release line this finding differs from the
        // later release's and drops out, though every release reports it. The message must match
        // the bash, so the gap stays.
        format!(
            "not found — the Dev Container {}-0004 governs has nothing to check",
            "ADR"
        )
    };
    vec![Finding {
        rule: RULE_ID,
        reason: "absent",
        file: CONFIG.to_owned(),
        line: None,
        message,
    }]
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Version;
    use crate::devcontainer::fixture::Tree;

    fn messages(tree: &Tree) -> Vec<String> {
        check(&tree.inputs())
            .findings
            .into_iter()
            .map(|f| format!("{}: {} [{}]", f.file, f.message, f.reason))
            .collect()
    }

    #[test]
    fn a_dev_container_without_its_definition_is_absent() {
        let mut tree = Tree::base();
        assert!(messages(&tree).is_empty());
        tree.files.config = None;
        assert_eq!(
            messages(&tree),
            [
                ".devcontainer/devcontainer.json: not found — a .devcontainer/ without its \
              definition (NUC-container-hides-host-daemon \u{a7}1) [absent]"
            ]
        );
    }

    // bash: no Dev Container at all
    #[test]
    fn no_dev_container_binds_nothing_from_v0_9_0_on_and_lacks_its_definition_before() {
        let mut tree = Tree::base();
        tree.files.dir = false;
        tree.files.config = None;
        tree.files.lock = None;
        assert!(messages(&tree).is_empty());
        tree.release = Some(Version(0, 9, 0));
        assert!(messages(&tree).is_empty());
        tree.release = Some(Version(0, 8, 1));
        assert_eq!(
            messages(&tree),
            [format!(
                ".devcontainer/devcontainer.json: not found — the Dev Container {} governs has \
                 nothing to check [absent]",
                crate::testing::adr_id(4)
            )]
        );
    }
}
