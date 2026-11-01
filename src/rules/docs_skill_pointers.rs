//! Check 11 of the template's `scripts/check-docs.sh`: every skill under `.agents/skills/` has a
//! symlink of its name under `.claude/skills/` that resolves to it, and every entry there is such
//! a symlink (ADR-0003 of the template).

use super::{Input, Inputs, Rule};
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/skill-pointers";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Every skill under .agents/skills/, a directory with a SKILL.md, has a symlink of \
              the same name under .claude/skills/ that resolves to it, and every entry there is \
              such a symlink. Claude Code reads only the pointer directory, so a skill without \
              its pointer is invisible to it; a copy instead of a symlink is a second source \
              that drifts, and a skill that lives only in the pointer directory is one no other \
              agent finds. Each message names the command that fixes it.",
    reads: &[Input::Skills],
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

fn finding(name: &str, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        file: format!(".claude/skills/{name}"),
        line: None,
        message,
    }
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let mut found = Vec::new();
    for (name, pointer) in &inputs.skills.skills {
        let message = match pointer {
            Pointer::Symlink => continue,
            Pointer::Other => format!(
                "is a copy of the skill, not a symlink — a second source drifts; replace it: rm -r .claude/skills/{name} && ln -s ../../.agents/skills/{name} .claude/skills/{name}"
            ),
            Pointer::Missing => format!(
                "no pointer — the skill .agents/skills/{name} is invisible to Claude Code; add it: ln -s ../../.agents/skills/{name} .claude/skills/{name}"
            ),
        };
        found.push(finding(name, message));
    }
    for (name, entry) in &inputs.skills.pointers {
        let message = match entry {
            Entry::NotSymlink { skill: true } | Entry::Points { at_skill: true, .. } => continue,
            Entry::NotSymlink { skill: false } => "is not a symlink into .agents/skills/ — a skill lives there, where every agent reads it, and is pointed at from here (ADR-0003)".to_owned(),
            Entry::Dangles => format!(
                "dangles — its target does not exist; point it at .agents/skills/{name} or delete it"
            ),
            Entry::Points { target, .. } => {
                format!("points at '{target}', not at the skill .agents/skills/{name}")
            }
        };
        found.push(finding(name, message));
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
            },
            ..Inputs::default()
        };
        let found: Vec<String> = check(&inputs)
            .into_iter()
            .map(|f| format!("{}: {}", f.file, f.message))
            .collect();
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
}
