//! Check 11 of the template's `scripts/check-docs.sh`: every skill under `.agents/skills/` has a
//! symlink of its name under `.claude/skills/` that resolves to it, and every entry there is such
//! a symlink (NUC-procedures-are-skills of the template).

use super::{Input, Inputs, Rule};
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/skill-pointers";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &[
        "copy",
        "missing",
        "stray",
        "dangles",
        "target",
        "frontmatter",
    ],
    explain: "Every skill under .agents/skills/, a directory with a SKILL.md, has a symlink of \
              the same name under .claude/skills/ that resolves to it, and every entry there is \
              such a symlink. Claude Code reads only the pointer directory, so a skill without \
              its pointer is invisible to it; a copy instead of a symlink is a second source \
              that drifts, and a skill that lives only in the pointer directory is one no other \
              agent finds. Each message names the command that fixes it. From template release \
              v0.9.0 on, each SKILL.md opens with a front matter an agent reads: '---', a 'name' \
              that is its directory, a 'description', unindented lines only keys, then '---'.",
    reads: &[Input::Skills, Input::Release],
    check: |inputs| check(inputs).into(),
};

/// What check 11 tests on `.agents/skills/` and `.claude/skills/`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Skills {
    /// Each skill, a directory of `.agents/skills/` with a file `SKILL.md`, sorted by name, with
    /// what stands at its pointer.
    pub skills: Vec<(String, Pointer)>,
    /// Each entry of `.claude/skills/`, sorted by name.
    pub pointers: Vec<(String, Entry)>,
    /// The bytes of each skill's `SKILL.md`, by name, read from template release v0.9.0 on.
    pub texts: std::collections::BTreeMap<String, Vec<u8>>,
}

/// Whether `SKILL.md` opens with a front matter an agent reads, as the awk of v0.9.1 reads it: a
/// first line `---`; then, up to a closing `---`, lines that are indented, comments, list items,
/// or blank, and keys `<key>:`, among them `name:` naming the directory, quotes and trailing space
/// aside, and `description:`.
pub fn front_matter(text: &[u8], name: &str) -> bool {
    let fence = |line: &[u8]| {
        line.strip_prefix(b"---")
            .is_some_and(|rest| rest.iter().all(|&b| b == b' ' || b == b'\t'))
    };
    let lines = crate::bash::awk_lines(text);
    let Some((first, rest)) = lines.split_first() else {
        return false;
    };
    if !fence(first) {
        return false;
    }
    let (mut named, mut described) = (false, false);
    for line in rest {
        if fence(line) {
            return named && described;
        }
        let skipped = match line.first() {
            None | Some(b' ' | b'\t' | b'#') => true,
            Some(b'-') => matches!(line.get(1), None | Some(b' ' | b'\t')),
            _ => false,
        };
        if skipped {
            continue;
        }
        let key = line
            .iter()
            .take_while(|b| b.is_ascii_alphanumeric() || **b == b'_' || **b == b'-')
            .count();
        let is_key = key > 0
            && line.get(key) == Some(&b':')
            && matches!(line.get(key + 1), None | Some(b' ' | b'\t'));
        if !is_key {
            return false;
        }
        if let Some(value) = line.strip_prefix(b"name:") {
            let value = &value[value
                .iter()
                .take_while(|&&b| b == b' ' || b == b'\t')
                .count()..];
            let value = value
                .strip_prefix(b"\"")
                .or_else(|| value.strip_prefix(b"'"))
                .unwrap_or(value);
            let end = value.len()
                - value
                    .iter()
                    .rev()
                    .take_while(|&&b| b == b' ' || b == b'\t')
                    .count();
            let mut value = &value[..end];
            if let Some(v) = value
                .strip_suffix(b"\"")
                .or_else(|| value.strip_suffix(b"'"))
            {
                value = v;
            }
            named = value == name.as_bytes();
        }
        if line.starts_with(b"description:") {
            described = true;
        }
    }
    false
}

/// What stands at a skill's pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pointer {
    Symlink,
    Other,
    Missing,
}

/// An entry of `.claude/skills/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// No symlink; `skill` when `.agents/skills/<name>/SKILL.md` is a file.
    NotSymlink { skill: bool },
    /// A symlink that resolves to no directory.
    Dangles,
    /// A symlink to the directory `target`, shown relative to the root when it is inside;
    /// `at_skill` when that is the skill of its name.
    Points { target: String, at_skill: bool },
}

fn finding(reason: &'static str, name: &str, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        reason,
        file: format!(".claude/skills/{name}"),
        line: None,
        message,
    }
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let named = inputs.carries_at_least(super::CATEGORY_IDENTIFIERS);
    let mut found = Vec::new();
    for (name, pointer) in &inputs.skills.skills {
        let text = inputs.skills.texts.get(name).map_or(&[][..], Vec::as_slice);
        if named && !front_matter(text, name) {
            found.push(Finding {
                rule: RULE_ID,
                reason: "frontmatter",
                file: format!(".agents/skills/{name}/SKILL.md"),
                line: None,
                message: format!(
                    "no front matter an agent reads — '---' first, a 'name: {name}' and a \
                     'description:' line, unindented lines only keys, then '---' \
                     (NUC-procedures-are-skills §7)"
                ),
            });
        }
        let (reason, message) = match pointer {
            Pointer::Symlink => continue,
            Pointer::Other => (
                "copy",
                format!(
                    "is a copy of the skill, not a symlink — a second source drifts; replace it: rm -r .claude/skills/{name} && ln -s ../../.agents/skills/{name} .claude/skills/{name}"
                ),
            ),
            Pointer::Missing => (
                "missing",
                format!(
                    "no pointer — the skill .agents/skills/{name} is invisible to Claude Code; add it: ln -s ../../.agents/skills/{name} .claude/skills/{name}"
                ),
            ),
        };
        found.push(finding(reason, name, message));
    }
    for (name, entry) in &inputs.skills.pointers {
        let (reason, message) = match entry {
            Entry::NotSymlink { skill: true } | Entry::Points { at_skill: true, .. } => continue,
            Entry::NotSymlink { skill: false } => (
                "stray",
                format!(
                    "is not a symlink into .agents/skills/ — a skill lives there, where every agent \
                     reads it, and is pointed at from here {}",
                    if named {
                        "(NUC-procedures-are-skills §6)"
                    } else {
                        "(ADR-0003)"
                    }
                ),
            ),
            Entry::Dangles => (
                "dangles",
                format!(
                    "dangles — its target does not exist; point it at .agents/skills/{name} or delete it"
                ),
            ),
            Entry::Points { target, .. } => (
                "target",
                format!("points at '{target}', not at the skill .agents/skills/{name}"),
            ),
        };
        found.push(finding(reason, name, message));
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_kind_of_pointer_reports_as_the_bash_does() {
        let inputs = Inputs {
            skills: Skills {
                skills: vec![
                    ("a".to_owned(), Pointer::Symlink),
                    ("b".to_owned(), Pointer::Other),
                    ("c".to_owned(), Pointer::Missing),
                ],
                pointers: vec![
                    (
                        "a".to_owned(),
                        Entry::Points {
                            target: ".agents/skills/a".to_owned(),
                            at_skill: true,
                        },
                    ),
                    ("b".to_owned(), Entry::NotSymlink { skill: true }),
                    ("d".to_owned(), Entry::NotSymlink { skill: false }),
                    ("e".to_owned(), Entry::Dangles),
                    (
                        "f".to_owned(),
                        Entry::Points {
                            target: ".agents/skills/a".to_owned(),
                            at_skill: false,
                        },
                    ),
                ],
                ..Skills::default()
            },
            release: Some(crate::config::Version(0, 8, 1)),
            ..Inputs::default()
        };
        let found: Vec<String> = check(&inputs)
            .into_iter()
            .map(|f| format!("{}: {}", f.file, f.message))
            .collect();
        // bash: a skill without its pointer
        // bash: a pointer whose skill is gone
        // bash: a copy where a symlink belongs
        // bash: a pointer at a different skill
        // bash: a skill that lives only in the pointer directory
        assert_eq!(
            found,
            [
                ".claude/skills/b: is a copy of the skill, not a symlink — a second source drifts; replace it: rm -r .claude/skills/b && ln -s ../../.agents/skills/b .claude/skills/b",
                ".claude/skills/c: no pointer — the skill .agents/skills/c is invisible to Claude Code; add it: ln -s ../../.agents/skills/c .claude/skills/c",
                ".claude/skills/d: is not a symlink into .agents/skills/ — a skill lives there, where every agent reads it, and is pointed at from here (ADR-0003)",
                ".claude/skills/e: dangles — its target does not exist; point it at .agents/skills/e or delete it",
                ".claude/skills/f: points at '.agents/skills/a', not at the skill .agents/skills/f",
            ]
        );
    }

    // bash: skills only in the pointer directory, none for the other agents
    #[test]
    fn a_skill_only_in_the_pointer_directory_with_no_skills_beside_it_fails() {
        let inputs = Inputs {
            skills: Skills {
                skills: Vec::new(),
                pointers: vec![("review".to_owned(), Entry::NotSymlink { skill: false })],
                ..Skills::default()
            },
            release: Some(crate::config::Version(0, 8, 1)),
            ..Inputs::default()
        };
        let found: Vec<String> = check(&inputs).into_iter().map(|f| f.file).collect();
        assert_eq!(found, [".claude/skills/review"]);
        assert!(
            check(&inputs)[0]
                .message
                .starts_with("is not a symlink into .agents/skills/")
        );
    }

    // bash: a repository without skills has nothing to check
    #[test]
    fn a_tree_without_skills_or_pointers_passes() {
        assert_eq!(check(&Inputs::default()), []);
    }

    #[test]
    fn from_v0_9_0_a_skill_without_front_matter_fails_and_a_stray_cites_the_named_adr() {
        let inputs = Inputs {
            skills: Skills {
                skills: vec![("a".to_owned(), Pointer::Symlink)],
                pointers: vec![("d".to_owned(), Entry::NotSymlink { skill: false })],
                texts: [("a".to_owned(), b"# A\n".to_vec())].into(),
            },
            ..Inputs::default()
        };
        let found: Vec<String> = check(&inputs)
            .into_iter()
            .map(|f| format!("{} {}: {}", f.reason, f.file, f.message))
            .collect();
        assert_eq!(
            found,
            [
                "frontmatter .agents/skills/a/SKILL.md: no front matter an agent reads — '---' \
                 first, a 'name: a' and a 'description:' line, unindented lines only keys, then \
                 '---' (NUC-procedures-are-skills §7)",
                "stray .claude/skills/d: is not a symlink into .agents/skills/ — a skill lives \
                 there, where every agent reads it, and is pointed at from here \
                 (NUC-procedures-are-skills §6)",
            ]
        );
    }

    #[test]
    fn the_front_matter_is_read_as_the_awk_of_v0_9_1_reads_it() {
        let ok = "---\nname: review\n# a comment\n- an item\n  indented: x\n\ndescription: d\n---\n# R\n";
        assert!(front_matter(ok.as_bytes(), "review"));
        assert!(front_matter(
            b"---  \nname: 'review'  \ndescription:\n--- \n",
            "review"
        ));
        assert!(front_matter(
            b"---\nname: \"review\"\ndescription: d\n---\n",
            "review"
        ));
        for (text, why) in [
            (
                "# R\n---\nname: review\ndescription: d\n---\n",
                "no opening fence",
            ),
            ("---\nname: review\ndescription: d\n", "never closed"),
            ("---\nname: other\ndescription: d\n---\n", "another name"),
            ("---\nname: review\n---\n", "no description"),
            (
                "---\nname: review\nwrapped words\ndescription: d\n---\n",
                "a line that is no key",
            ),
            (
                "---\nname: review\nmetadata:\n  description: d\n---\n",
                "a nested description",
            ),
        ] {
            assert!(!front_matter(text.as_bytes(), "review"), "{why}");
        }
    }
}
