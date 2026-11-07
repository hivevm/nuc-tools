//! Check 16 of the template's `scripts/check-docs.sh`: a change that lands on `main` carries no
//! draft ADR. The branch is the one CI names, which the shell reads from the environment as the
//! bash does: the base of a pull request, else the branch a push runs on; a draft pull request
//! passes, and so does a run outside CI.

use super::{Input, Inputs, Outcome, Rule};
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/no-draft-on-main";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "A change that lands on main carries no draft ADR: a draft takes its number on the \
              branch whose pull request brings it there, by scripts/number-adrs.sh. The branch a \
              change lands on is the one CI names: the base of a pull request, else the branch a \
              push runs on. A draft pull request passes, since it tests a feat/ branch still \
              under rework, and so does a run outside CI, where nothing names the branch.",
    reads: &[Input::Landing],
    check: outcome,
};

/// What CI says about the change, as the bash reads it from the environment, and the drafts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Landing {
    /// `GITHUB_BASE_REF`.
    pub base_ref: Option<String>,
    /// Whether `GITHUB_ACTIONS` is `true`.
    pub actions: bool,
    /// `GITHUB_REF_NAME`.
    pub ref_name: Option<String>,
    /// Whether `PULL_REQUEST_DRAFT` is `true`.
    pub draft_pull_request: bool,
    /// The regular files `docs/adr/draft-*.md`, by name, sorted as `find | LC_ALL=C sort` gives
    /// them.
    pub drafts: Vec<String>,
}

impl Landing {
    /// The branch the change lands on, as the bash's `lands_on` prints it.
    pub fn lands_on(&self) -> Option<&str> {
        match self.base_ref.as_deref().filter(|b| !b.is_empty()) {
            Some(base) => Some(base),
            None if self.actions => Some(self.ref_name.as_deref().unwrap_or("")),
            None => None,
        }
    }
}

/// The findings of the rule, or its refusal of a draft whose name holds a newline, which the bash
/// reads from find's list as two (Q-1).
fn outcome(inputs: &Inputs) -> Outcome {
    if let Some(name) = inputs
        .landing
        .drafts
        .iter()
        .find(|name| name.contains('\n'))
    {
        return Outcome {
            refusal: Some(format!(
                "docs/adr/{}: a draft whose name holds a newline, which the bash reads as two — not decided",
                name.escape_debug()
            )),
            ..Outcome::default()
        };
    }
    check(inputs).into()
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let landing = &inputs.landing;
    if landing.lands_on() != Some("main") || landing.draft_pull_request {
        return Vec::new();
    }
    landing
        .drafts
        .iter()
        .map(|name| Finding {
            rule: RULE_ID,
            file: format!("docs/adr/{name}"),
            line: None,
            message: "a draft lands on main — give it its number first with scripts/number-adrs.sh (ADR-0002)"
                .to_owned(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(landing: Landing) -> usize {
        let inputs = Inputs {
            landing: Landing {
                drafts: vec!["draft-next.md".to_owned()],
                ..landing
            },
            ..Inputs::default()
        };
        check(&inputs).len()
    }

    fn pull_request(base: &str, draft: bool) -> Landing {
        Landing {
            base_ref: Some(base.to_owned()),
            draft_pull_request: draft,
            ..Landing::default()
        }
    }

    /// The cases of check 16 in the template's `scripts/test-check-docs.sh`.
    #[test]
    fn a_draft_fails_only_where_a_ready_change_lands_on_main() {
        // bash: a draft, its index row, a link, and a supersession, landing on a feat/ branch
        assert_eq!(found(pull_request("feat/topic", false)), 0);
        // bash: a draft outside CI, where nothing names the branch
        assert_eq!(found(Landing::default()), 0);
        // bash: a draft in a pull request into main ready for review
        assert_eq!(found(pull_request("main", false)), 1);
        // bash: a draft in a draft pull request into main, a feat/ branch under rework
        assert_eq!(found(pull_request("main", true)), 0);
        // bash: a draft in a push to main
        let push = Landing {
            actions: true,
            ref_name: Some("main".to_owned()),
            ..Landing::default()
        };
        assert_eq!(found(push), 1);
    }

    #[test]
    fn an_empty_base_falls_back_to_the_branch_of_the_push() {
        let landing = Landing {
            base_ref: Some(String::new()),
            actions: true,
            ref_name: Some("main".to_owned()),
            ..Landing::default()
        };
        assert_eq!(landing.lands_on(), Some("main"));
        let outside = Landing {
            ref_name: Some("main".to_owned()),
            ..Landing::default()
        };
        assert_eq!(outside.lands_on(), None);
    }

    #[test]
    fn a_draft_whose_name_holds_a_newline_is_its_refusal() {
        let inputs = Inputs {
            landing: Landing {
                drafts: vec!["draft-a\nb.md".to_owned()],
                ..pull_request("main", false)
            },
            ..Inputs::default()
        };
        let outcome = outcome(&inputs);
        assert!(outcome.findings.is_empty());
        assert_eq!(
            outcome.refusal.as_deref(),
            Some(
                "docs/adr/draft-a\\nb.md: a draft whose name holds a newline, which the bash reads as two — not decided"
            )
        );
    }
}
