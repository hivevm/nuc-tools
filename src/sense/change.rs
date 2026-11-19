//! The change a sensor reads, as `resolve_change` of the template's `scripts/lib/git.sh` resolves
//! it from the base and the head it was given (from v0.8.1; the sensors before resolved it alike in
//! their own lines). The shell asks git; what a sensor reports where there is nothing to compare,
//! and what it refuses, stand here.

use super::Refusal;

/// The base given, none where it is empty or zeros, as a push event reports for a new branch.
pub fn given_base(base: Option<&str>) -> Option<&str> {
    base.filter(|base| !base.is_empty() && !base.bytes().all(|b| b == b'0'))
}

/// The head given, `HEAD` where none is.
pub fn given_head(head: Option<&str>) -> &str {
    head.filter(|head| !head.is_empty()).unwrap_or("HEAD")
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
}
