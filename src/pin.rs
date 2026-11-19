//! The blob list of the git hooks, `.githooks/nuc.blobs`, as the template reads it from release
//! v0.12.0 on (NUC-hooks-run-nuc §2): `pinned_files` of its `scripts/lib/pin.sh`, which every check
//! that skips the pinned files calls, and the reading of `scripts/check-hooks.sh`, which the rules
//! `hooks/pin` and `hooks/toml` share here, beside them, so that no rule module calls another.

use std::collections::BTreeMap;

/// The path of the blob list, from the root of the repository.
pub const LIST: &str = ".githooks/nuc.blobs";

/// The hooks a blob list may pin, as the template's `scripts/lib/pin.sh` names them.
pub const PINNED_HOOKS: [&str; 5] = [
    "merge-adr-index",
    "pre-commit",
    "pre-merge-commit",
    "pre-push",
    "reference-transaction",
];

/// What the shell reads of the hooks for the rules: the bytes of the blob list where it is a
/// file, as `-f` finds it; and, for each hook of the five that is a file under `.githooks/`, by
/// its name, the blob git would store for it, or none where git cannot hash it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hooks {
    pub list: Option<Vec<u8>>,
    pub blobs: BTreeMap<String, Option<String>>,
}

/// The path of each file the blob list pins, as pinned_files of the template's
/// `scripts/lib/pin.sh` reads the bytes of `.githooks/nuc.blobs`: each line, the last one without
/// a newline included, split as `read -r blob hook rest` splits it, names `.githooks/<hook>` where
/// the blob is a git object name of SHA-1 or SHA-256 in lower-case hex, the hook, without one
/// trailing carriage return, is one of the five, and nothing follows; any other line names none.
pub fn pinned_files(list: &[u8]) -> std::collections::BTreeSet<String> {
    crate::bash::awk_lines(list)
        .into_iter()
        .filter_map(|line| {
            let (blob, hook, rest) = crate::bash::read_fields(line);
            let hook = hook.strip_suffix(b"\r").unwrap_or(&hook);
            pinned_blob(&blob)
                .then_some(())
                .filter(|()| rest.is_empty())
                .and_then(|()| pinned_hook(hook))
                .map(|name| format!(".githooks/{name}"))
        })
        .collect()
}

/// Whether the word is the blob of a line `<blob> <hook>`, as pin.sh's `PINNED_BLOB_RE` matches
/// it: a git object name of SHA-1 or SHA-256, in lower-case hex.
fn pinned_blob(word: &[u8]) -> bool {
    matches!(word.len(), 40 | 64) && word.iter().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// The hook of the five the word names, as pin.sh's `pinned_hook` finds it.
fn pinned_hook(word: &[u8]) -> Option<&'static str> {
    PINNED_HOOKS
        .iter()
        .copied()
        .find(|name| name.as_bytes() == word)
}

/// A line of the blob list that `scripts/check-hooks.sh` fails on as it reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wrong {
    /// A line `<blob> <name>` whose name is none of the five hooks.
    Foreign(String),
    /// Any other line that is no entry, as the line reads after its carriage return.
    Entry(String),
}

/// The blob list as `scripts/check-hooks.sh` reads it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct List {
    /// The release of the last line `release v<X.Y.Z>`, without its `v`.
    pub release: Option<String>,
    /// The rule ID of each line `rule hook/<name>`, in order, twice where it stands twice.
    pub rules: Vec<String>,
    /// The blob of each hook a line `<blob> <hook>` pins, the last line of a hook winning.
    pub listed: BTreeMap<&'static str, String>,
    /// The lines that are no entry, in order.
    pub wrong: Vec<Wrong>,
}

/// The blob list in `bytes` as the loop of `scripts/check-hooks.sh` reads it: each line, the last
/// one without a newline included, without its NUL bytes as `read` drops them and without one
/// trailing carriage return; an empty line and one that opens with `#` are no entry. The rest
/// split as `read -r first second rest` splits it is, in this order of tests: a release where the
/// first word is `release`, the second `v` and three dot-separated runs of digits, and no rest; a
/// rule where the first is `rule`, the second opens with `hook/`, and no rest; a pin where the
/// first is a blob, the second one of the five hooks, and no rest; a foreign pin where the first
/// is a blob, the second is not empty, and no rest; and otherwise no entry.
pub fn read_list(bytes: &[u8]) -> List {
    let mut list = List::default();
    for line in crate::bash::awk_lines(bytes) {
        let line: Vec<u8> = line.iter().copied().filter(|&b| b != 0).collect();
        let line = line.strip_suffix(b"\r").unwrap_or(&line);
        if line.is_empty() || line.starts_with(b"#") {
            continue;
        }
        let (first, second, rest) = crate::bash::read_fields(line);
        let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();
        if first == b"release" && release_word(&second) && rest.is_empty() {
            list.release = Some(text(&second[1..]));
        } else if first == b"rule" && second.starts_with(b"hook/") && rest.is_empty() {
            list.rules.push(text(&second));
        } else if let Some(hook) = (pinned_blob(&first) && rest.is_empty())
            .then(|| pinned_hook(&second))
            .flatten()
        {
            list.listed.insert(hook, text(&first));
        } else if pinned_blob(&first) && !second.is_empty() && rest.is_empty() {
            list.wrong.push(Wrong::Foreign(text(&second)));
        } else {
            list.wrong.push(Wrong::Entry(text(line)));
        }
    }
    list
}

/// Whether the word matches `^v[0-9]+\.[0-9]+\.[0-9]+$`.
fn release_word(word: &[u8]) -> bool {
    let Some(version) = word.strip_prefix(b"v") else {
        return false;
    };
    let parts: Vec<&[u8]> = version.split(|&b| b == b'.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.iter().all(u8::is_ascii_digit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_blob_list_pins_a_hook_of_the_five_by_a_blob_and_nothing_else() {
        let sha1 = "0".repeat(40);
        let sha256 = "a".repeat(64);
        let list = format!(
            "# The pin\nrelease v0.2.2\n{sha1} pre-push\n {sha256}\tpre-commit\r\n\
             {sha1} own\n{sha1} pre-merge-commit extra\n{} merge-adr-index\n\
             {} reference-transaction\nrule hook/main-moves\n{sha1} merge-adr-index",
            "A".repeat(40),
            "0".repeat(41),
        );
        let pinned: Vec<String> = pinned_files(list.as_bytes()).into_iter().collect();
        assert_eq!(
            pinned,
            [
                ".githooks/merge-adr-index",
                ".githooks/pre-commit",
                ".githooks/pre-push"
            ]
        );
        assert!(pinned_files(b"").is_empty());
    }

    #[test]
    fn check_hooks_reads_a_release_rules_pins_and_what_is_no_entry() {
        let sha1 = "0".repeat(40);
        let sha256 = "a".repeat(64);
        let text = format!(
            "# The pin\n\nrelease v0.1.0\nrelease v0.02.2\r\nrelease 0.3.0\nrelease v1.2\n\
             rule hook/a\nrule hook/a\nrule docs/b\nrule hook/c d\n\
             {sha1} pre-push\n{sha256}\tpre-commit \n{sha1} pre-push\n{sha1} own\n\
             {sha1} ../pre-push\n{sha1}\n{sha1} pre-merge-commit extra\n  # indented\n \n\
             r\0ule hook/n\0ul\nrule hook/last"
        );
        let list = read_list(text.as_bytes());
        assert_eq!(list.release.as_deref(), Some("0.02.2"));
        assert_eq!(list.rules, ["hook/a", "hook/a", "hook/nul", "hook/last"]);
        assert_eq!(
            list.listed,
            BTreeMap::from([("pre-commit", sha256), ("pre-push", sha1.clone())])
        );
        let entry = |line: &str| Wrong::Entry(line.to_owned());
        assert_eq!(
            list.wrong,
            [
                entry("release 0.3.0"),
                entry("release v1.2"),
                entry("rule docs/b"),
                entry("rule hook/c d"),
                Wrong::Foreign("own".to_owned()),
                Wrong::Foreign("../pre-push".to_owned()),
                entry(&sha1),
                entry(&format!("{sha1} pre-merge-commit extra")),
                entry("  # indented"),
                entry(" "),
            ]
        );
        assert_eq!(read_list(b""), List::default());
        assert_eq!(read_list(b"\xff\n").wrong, [entry("\u{fffd}")]);
    }
}
