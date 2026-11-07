//! `nuc.toml`: the minimum version of `nuc` a project requires and the rule IDs it turns on
//! (ADR-0023).
//!
//! The file is a subset of TOML, read without a TOML crate: comments, blank lines, and the two
//! keys `minimum`, a string `"X.Y.Z"`, and `rules`, an array of strings that may span lines.
//! Anything else is unreadable, so that a file `nuc` does not understand never passes (Q-1).

use std::fmt;

use crate::hook::{self, HookRule};
use crate::rules::{self, Rule};

/// How to install or update `nuc`, named in every refusal that a newer `nuc` would resolve.
pub const INSTALL: &str = "install or update nuc from https://github.com/hivevm/nuc-tools/releases";

/// A version `X.Y.Z`, ordered by its numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u64, pub u64, pub u64);

impl Version {
    /// The version in `text`, three dot-separated numbers and nothing else.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split('.').map(|part| {
            (!part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
                .then(|| part.parse().ok())
                .flatten()
        });
        let version = Self(parts.next()??, parts.next()??, parts.next()??);
        parts.next().is_none().then_some(version)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// What `nuc.toml` says.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Config {
    minimum: Version,
    rules: Vec<String>,
}

/// Why `nuc` refuses to check with a `nuc.toml`; each is a run that could not decide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The file is not there.
    Missing,
    /// The file cannot be read as the subset this `nuc` knows, with the line and the reason.
    Unreadable(usize, &'static str),
    /// The project requires a newer `nuc` than the one running.
    TooOld { minimum: Version, running: Version },
    /// The file turns on a rule this `nuc` does not know.
    UnknownRule(String),
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(
                f,
                "nuc.toml: not found at the root of the repository — the template release ships it; take \
                 it up, or {INSTALL} if this one predates it"
            ),
            Self::Unreadable(line, why) => write!(
                f,
                "nuc.toml:{line}: unreadable — {why}; a newer nuc may read it: {INSTALL}"
            ),
            Self::TooOld { minimum, running } => write!(
                f,
                "nuc.toml: requires nuc {minimum} or later, this is {running} — {INSTALL}"
            ),
            Self::UnknownRule(id) => write!(
                f,
                "nuc.toml: turns on the rule '{id}', which this nuc does not know — {INSTALL}"
            ),
        }
    }
}

/// The rules of `all` that the configuration in `text` turns on, in the order of `all`, for a
/// `nuc` of version `running`; a rule not listed stays off. It refuses as [`load`] and then
/// [`known`] refuse, so no caller turns on less than the file names (ADR-0023).
pub fn turned_on<'a, R: Named>(
    text: &str,
    running: Version,
    all: &'a [R],
) -> Result<Vec<&'a R>, Refusal> {
    let config = load(text, running)?;
    known(&config)?;
    Ok(all
        .iter()
        .filter(|rule| config.rules.iter().any(|id| id == rule.id()))
        .collect())
}

/// A rule `nuc.toml` can turn on by its ID: a rule of `nuc check` or a hook rule.
pub trait Named {
    fn id(&self) -> &'static str;
}

impl Named for Rule {
    fn id(&self) -> &'static str {
        self.id
    }
}

impl Named for HookRule {
    fn id(&self) -> &'static str {
        self.id
    }
}

/// The configuration in `text` for a `nuc` of version `running`. The minimum is read before
/// anything else, so that a newer file, with a key this `nuc` does not know, refuses it as too
/// old rather than as unreadable (ADR-0020).
fn load(text: &str, running: Version) -> Result<Config, Refusal> {
    if let Some(minimum) = minimum_first(text).filter(|&minimum| minimum > running) {
        return Err(Refusal::TooOld { minimum, running });
    }
    parse(text)
}

/// The version of the first line that sets `minimum` to a readable `"X.Y.Z"`, if one does.
fn minimum_first(text: &str) -> Option<Version> {
    text.split('\n').find_map(|line| {
        let (key, value) = strip_comment(line).split_once('=')?;
        (key.trim() == "minimum")
            .then(|| string(value.trim()).and_then(Version::parse))
            .flatten()
    })
}

/// The configuration in `text`, or why it cannot be read.
fn parse(text: &str) -> Result<Config, Refusal> {
    let mut minimum = None;
    let mut rules = None;
    let mut lines = text.split('\n').enumerate().map(|(i, line)| (i + 1, line));
    while let Some((n, line)) = lines.next() {
        let line = strip_comment(line).trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or(Refusal::Unreadable(
            n,
            "a line is neither a comment nor 'key = value'",
        ))?;
        match key.trim() {
            "minimum" if minimum.is_none() => minimum = Some(parse_minimum(n, value.trim())?),
            "rules" if rules.is_none() => rules = Some(parse_rules(n, value.trim(), &mut lines)?),
            "minimum" | "rules" => return Err(Refusal::Unreadable(n, "a key is set twice")),
            _ => {
                return Err(Refusal::Unreadable(
                    n,
                    "a key other than 'minimum' and 'rules'",
                ));
            }
        }
    }
    let end = text.split('\n').count();
    Ok(Config {
        minimum: minimum.ok_or(Refusal::Unreadable(end, "no 'minimum' version"))?,
        rules: rules.ok_or(Refusal::Unreadable(end, "no 'rules' list"))?,
    })
}

/// The line without a comment that starts outside a string.
fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..i],
            _ => {}
        }
    }
    line
}

/// The string literal that is all of `value`: plain, without escapes.
fn string(value: &str) -> Option<&str> {
    let inner = value.strip_prefix('"')?.strip_suffix('"')?;
    (!inner.contains(['"', '\\'])).then_some(inner)
}

fn parse_minimum(n: usize, value: &str) -> Result<Version, Refusal> {
    string(value)
        .and_then(Version::parse)
        .ok_or(Refusal::Unreadable(
            n,
            "'minimum' is not a string \"X.Y.Z\"",
        ))
}

/// The strings of an array that opens on line `n` and may close on a later line of `lines`.
fn parse_rules<'a>(
    n: usize,
    value: &'a str,
    lines: &mut impl Iterator<Item = (usize, &'a str)>,
) -> Result<Vec<String>, Refusal> {
    let mut text = value
        .strip_prefix('[')
        .ok_or(Refusal::Unreadable(n, "'rules' is not an array"))?
        .to_owned();
    let mut last = n;
    while !text.contains(']') {
        let (m, line) = lines
            .next()
            .ok_or(Refusal::Unreadable(last, "'rules' is not closed by ']'"))?;
        text.push(' ');
        text.push_str(strip_comment(line));
        last = m;
    }
    let (items, rest) = text.split_once(']').unwrap_or((&text, ""));
    if !rest.trim().is_empty() {
        return Err(Refusal::Unreadable(
            last,
            "text after the closing ']' of 'rules'",
        ));
    }
    let mut entries: Vec<&str> = items.split(',').map(str::trim).collect();
    // An empty last entry is a trailing comma, or the whole of an empty array.
    if entries.last() == Some(&"") {
        entries.pop();
    }
    entries
        .into_iter()
        .map(|entry| {
            string(entry).map(str::to_owned).ok_or(Refusal::Unreadable(
                last,
                "an entry of 'rules' is not a string",
            ))
        })
        .collect()
}

/// The rule ID and the reason `nuc explain` gives for `id`, where this `nuc` knows it: a rule
/// `nuc check` runs, or a hook rule `nuc hook` decides. The one answer to which IDs `nuc.toml` may
/// name.
pub fn explanation(id: &str) -> Option<(&'static str, &'static str)> {
    rules::find(id)
        .map(|rule| (rule.id, rule.explain))
        .or_else(|| hook::find(id).map(|rule| (rule.id, rule.explain)))
}

/// The first rule ID the configuration names that this `nuc` does not know, which `nuc check`
/// and `nuc hook` refuse alike (ADR-0023).
fn known(config: &Config) -> Result<(), Refusal> {
    match config.rules.iter().find(|id| explanation(id).is_none()) {
        Some(unknown) => Err(Refusal::UnknownRule(unknown.clone())),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V: Version = Version(0, 1, 0);

    #[test]
    fn a_file_with_both_keys_reads() {
        let text = "# Shipped by the template.\nminimum = \"0.1.0\" # floor\nrules = [\n  \"docs/a\", # first\n  \"docs/b\",\n]\n";
        let config = parse(text).unwrap();
        assert_eq!(config.minimum, V);
        assert_eq!(config.rules, ["docs/a", "docs/b"]);
    }

    #[test]
    fn an_empty_list_and_a_list_on_one_line_read() {
        assert_eq!(
            parse("minimum = \"1.2.3\"\nrules = []\n").unwrap().rules,
            Vec::<String>::new()
        );
        assert_eq!(
            parse("rules = [\"x/y\"]\nminimum = \"1.2.3\"")
                .unwrap()
                .rules,
            ["x/y"]
        );
    }

    #[test]
    fn what_this_nuc_does_not_know_is_unreadable() {
        for (text, line) in [
            ("minimum = \"0.1.0\"\n", 2),
            ("rules = []\n", 2),
            ("minimum = \"0.1\"\nrules = []", 1),
            ("minimum = 1\nrules = []", 1),
            ("minimum = \"0.1.0\"\nrules = [\"a\", 1]", 2),
            ("minimum = \"0.1.0\"\nrules = [\"a\",,]", 2),
            ("minimum = \"0.1.0\"\nrules = [\n\"a\"", 3),
            ("minimum = \"0.1.0\"\nrules = [] x", 2),
            ("minimum = \"0.1.0\"\nminimum = \"0.1.0\"\nrules = []", 2),
            ("minimum = \"0.1.0\"\nrules = []\nextra = \"x\"", 3),
            ("minimum = \"0.1.0\"\n[table]\nrules = []", 2),
            ("minimum = \"0.1.0\"\nrules = [\"a\\\"b\"]", 2),
        ] {
            match parse(text) {
                Err(Refusal::Unreadable(n, _)) => assert_eq!(n, line, "in: {text:?}"),
                other => panic!("{text:?} read as {other:?}"),
            }
        }
    }

    #[test]
    fn versions_parse_and_order_by_their_numbers() {
        assert_eq!(Version::parse("10.0.2"), Some(Version(10, 0, 2)));
        assert!(Version(0, 10, 0) > Version(0, 9, 9));
        for text in ["1.2", "1.2.3.4", "v1.2.3", "1.-2.3", "1.2.x", "", "1..2"] {
            assert_eq!(Version::parse(text), None, "{text}");
        }
    }

    #[test]
    fn a_newer_minimum_refuses_before_a_key_this_nuc_does_not_know() {
        let text = "level = \"strict\"\nminimum = \"0.2.0\" # floor\nrules = []\n";
        assert_eq!(
            load(text, V),
            Err(Refusal::TooOld {
                minimum: Version(0, 2, 0),
                running: V
            })
        );
        assert!(matches!(
            load(text, Version(0, 2, 0)),
            Err(Refusal::Unreadable(1, _))
        ));
        assert!(matches!(
            load("minimum = \"x\"\nrules = []", V),
            Err(Refusal::Unreadable(1, _))
        ));
    }

    #[test]
    fn one_call_turns_on_the_listed_rules_and_hook_rules_and_refuses_as_before() {
        let rule = |id| Rule {
            id,
            explain: "",
            reads: &[],
            check: |_| Vec::new().into(),
        };
        let all = [rule("docs/template-release"), rule("docs/adr-index")];
        let text = "minimum = \"0.1.0\"\nrules = [\"hook/commit-on-main\", \"docs/adr-index\"]\n";
        let rules = turned_on(text, V, &all).ok().unwrap();
        assert_eq!(
            rules.iter().map(|r| r.id).collect::<Vec<_>>(),
            ["docs/adr-index"]
        );
        let hooks = turned_on(text, V, hook::ALL).ok().unwrap();
        assert_eq!(
            hooks.iter().map(|r| r.id).collect::<Vec<_>>(),
            ["hook/commit-on-main"]
        );
        let unknown = "minimum = \"0.1.0\"\nrules = [\"hook/commit-on-main\", \"docs/c\"]\n";
        assert_eq!(
            turned_on(unknown, V, hook::ALL).err(),
            Some(Refusal::UnknownRule("docs/c".to_owned()))
        );
        let newer = "minimum = \"0.2.0\"\nrules = [\"docs/c\"]\n";
        assert!(matches!(
            turned_on(newer, V, &all).err(),
            Some(Refusal::TooOld { .. })
        ));
    }

    #[test]
    fn a_rule_and_a_hook_rule_are_known_and_any_other_id_is_refused() {
        let config = |rules: &[&str]| Config {
            minimum: V,
            rules: rules.iter().map(|r| (*r).to_owned()).collect(),
        };
        assert_eq!(
            known(&config(&["docs/template-release", "hook/commit-on-main"])),
            Ok(())
        );
        assert_eq!(
            known(&config(&["hook/commit-on-main", "docs/c"])),
            Err(Refusal::UnknownRule("docs/c".to_owned()))
        );
    }
}
