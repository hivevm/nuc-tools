//! The change a sensor reads, as `resolve_change` of the template's `scripts/lib/git.sh` resolves
//! it from the base and the head it was given (from v0.8.1; the sensors before resolved it alike in
//! their own lines). The shell asks git; how the arguments read, what a sensor reports where there
//! is nothing to compare, and what it refuses, stand here.

use super::Refusal;

/// The base given, none where it is empty or zeros, as a push event reports for a new branch.
pub fn given_base(base: Option<&str>) -> Option<&str> {
    base.filter(|base| !base.is_empty() && !base.bytes().all(|b| b == b'0'))
}

/// The head given, `HEAD` where none is.
pub fn given_head(head: Option<&str>) -> &str {
    head.filter(|head| !head.is_empty()).unwrap_or("HEAD")
}

/// The change as the arguments give it, before git is asked: the base, none where it is to be
/// main's merge base; the head; and whether the change ends in the working tree, `CHANGE_WORKTREE`,
/// which it does where no head is given, the head then `HEAD`.
#[derive(Debug, PartialEq, Eq)]
pub struct Given<'a> {
    pub base: Option<&'a str>,
    pub head: &'a str,
    pub worktree: bool,
}

/// The change as `resolve_change <name> <base> [head]` reads its arguments.
pub fn given<'a>(base: Option<&'a str>, head: Option<&'a str>) -> Given<'a> {
    Given {
        base: given_base(base),
        head: given_head(head),
        worktree: head.is_none_or(str::is_empty),
    }
}

/// The candidates for main, in the order the bash tries them: the remote's main first, since a
/// pull request merges into it and a local main may lag.
pub const MAINS: [&str; 2] = ["origin/main", "main"];

/// The report where no base was given and there is no main to compare with.
pub fn nothing_to_compare(label: &str) -> Vec<u8> {
    format!("{label}: nothing to compare — no base given and no main to compare with.\n")
        .into_bytes()
}

/// The refusal of a base or head that does not resolve to a commit, where the bash exits 2.
pub fn unresolved(sensor: &'static str, reference: &str) -> Refusal {
    Refusal {
        sensor,
        reason: "ref",
        message: format!("'{reference}' does not resolve to a commit"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeros_and_nothing_are_no_base_and_the_head_defaults_to_head() {
        assert_eq!(given_base(Some("0000000")), None);
        assert_eq!(given_base(Some("")), None);
        assert_eq!(given_base(None), None);
        assert_eq!(given_base(Some("0a")), Some("0a"));
        assert_eq!(given_head(None), "HEAD");
        assert_eq!(given_head(Some("x")), "x");
    }

    #[test]
    fn each_form_of_the_arguments_reads_as_resolve_change_reads_it() {
        let given = |base, head, want_base, want_head, worktree| {
            assert_eq!(
                super::given(base, head),
                Given {
                    base: want_base,
                    head: want_head,
                    worktree
                },
                "{base:?} {head:?}"
            );
        };
        // No argument: main's merge base with HEAD, up to the working tree.
        given(None, None, None, "HEAD", true);
        // An empty base and head count as none, as `[[ -n ]]` reads them.
        given(Some(""), Some(""), None, "HEAD", true);
        // A base alone: from the base up to the working tree.
        given(Some("main"), None, Some("main"), "HEAD", true);
        // A base of zeros, as a push event names a new branch: main's merge base with the head.
        given(Some("000"), Some("feature"), None, "feature", false);
        // A base and a head: the commits, the working tree left out, even where the head is HEAD.
        given(Some("main"), Some("HEAD"), Some("main"), "HEAD", false);
        given(Some("a"), Some("b"), Some("a"), "b", false);
    }
}
