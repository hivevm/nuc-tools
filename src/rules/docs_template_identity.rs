//! Check 13 of the template's `scripts/check-docs.sh`: once `TEMPLATE-SETUP.md` is gone, none of
//! the template's placeholders is left in the README, the Dev Container, LICENSE, SECURITY.md,
//! CODE_OF_CONDUCT.md, CODEOWNERS, and the four documents of `docs/`. A file that does not exist
//! has nothing to check.
//!
//! The badge pattern is this repository's correction of the template's: a badge of a repository
//! whose name only starts with the template's, such as `hivevm/nuc-tools`, is the project's own.

use super::{Input, Inputs, Rule};
use crate::adr::{awk_lines, contains, is_space};
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/template-identity";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "Once TEMPLATE-SETUP.md is gone, none of the template's placeholders is left: the \
              name NUC in the README's title, intro, Why NUC section, and the Dev Container, the \
              template's repository in a README badge, the placeholder copyright holder, \
              security contact, and reporting contact, a CODEOWNERS with no active rule or its \
              TODO, the '<Project Name>' titles and scaffold entries of the documents, and a \
              TODO command or scaffold sentence in the README. A placeholder that survives the \
              setup describes the template, not the project.",
    reads: &[Input::SetupFile, Input::Identity],
    check,
};

/// Every file the placeholders stand in, which the shell reads where it is a file.
pub const FILES: &[&str] = &[
    "README.md",
    ".devcontainer/devcontainer.json",
    "LICENSE",
    "SECURITY.md",
    "CODE_OF_CONDUCT.md",
    ".github/CODEOWNERS",
    "docs/SPECIFICATION.md",
    "docs/ARCHITECTURE.md",
    "docs/GLOSSARY.md",
    "docs/CONVENTIONS.md",
];

/// A placeholder: the file, whether a line of it shows the placeholder, and the message.
type Placeholder = (&'static str, fn(&[u8]) -> bool, &'static str);

const PLACEHOLDERS: &[Placeholder] = &[
    (
        "README.md",
        |l| l.starts_with("# NUC — an Agentic".as_bytes()),
        "the title is still the template's; give the project its name",
    ),
    (
        "README.md",
        |l| l.starts_with(b"**NUC** is named after"),
        "the intro still explains the template's name; say what the project is",
    ),
    (
        "README.md",
        |l| l == b"## Why NUC",
        "the Why NUC section still presents the template; delete it",
    ),
    (
        "README.md",
        |l| l.starts_with(b"Describe what this project does"),
        "the Overview is still the scaffold's sentence; describe the project",
    ),
    (
        "README.md",
        |l| l.starts_with("TODO — show a minimal example".as_bytes()),
        "the Usage section is still TODO; show how the project is used, or say there is nothing to use yet",
    ),
    (
        "README.md",
        template_badge,
        "a badge still points at the template's repository; repoint or delete it",
    ),
    (
        ".devcontainer/devcontainer.json",
        |l| contains(l, b"\"name\": \"NUC DevContainer\""),
        "the Dev Container still carries the template's name",
    ),
    (
        "LICENSE",
        copyright_placeholder,
        "the copyright holder is still the placeholder; name the maintainer",
    ),
    (
        "SECURITY.md",
        |l| contains(l, b"TODO: add a security contact"),
        "the security contact is still the placeholder",
    ),
    (
        "CODE_OF_CONDUCT.md",
        |l| contains(l, b"TODO: add a contact address"),
        "the reporting contact is still the placeholder",
    ),
    (
        ".github/CODEOWNERS",
        |l| l.starts_with(b"# TODO: add a code owner"),
        "the instruction to add a code owner is still there; remove it once the rules are active",
    ),
];

const TITLE: &str = "the title still carries '<Project Name>'; give the project its name";

const SCAFFOLD: &[Placeholder] = &[
    (
        "docs/SPECIFICATION.md",
        scaffold_criterion,
        "a criterion is still the scaffold's placeholder; write it, or delete the item",
    ),
    (
        "docs/ARCHITECTURE.md",
        |l| l.starts_with(b"- **<Part>**"),
        "the scaffold's building block is still listed; describe the parts, or delete the entry",
    ),
    (
        "docs/GLOSSARY.md",
        |l| l.starts_with(b"- **<Term>**"),
        "the scaffold's entry is still listed; write the first term, or delete the entry",
    ),
    (
        "docs/CONVENTIONS.md",
        |l| l.starts_with(b"- **<Convention>**"),
        "the scaffold's entry is still listed; write the first convention, or delete the entry",
    ),
    (
        "README.md",
        todo_command,
        "a Build, Test, Lint, or Run command is still TODO; name the command the toolchain ADR chose",
    ),
];

fn check(inputs: &Inputs) -> Vec<Finding> {
    if inputs.setup_file {
        return Vec::new();
    }
    let mut found = Vec::new();
    let mut report = |file: &str, message: &str| {
        found.push(Finding {
            rule: RULE_ID,
            file: file.to_owned(),
            line: None,
            message: message.to_owned(),
        });
    };
    let any_line = |file: &str, matches: fn(&[u8]) -> bool| {
        inputs
            .identity
            .get(file)
            .is_some_and(|text| awk_lines(text).into_iter().any(matches))
    };
    for &(file, matches, message) in PLACEHOLDERS {
        if any_line(file, matches) {
            report(file, message);
        }
    }
    let codeowners = ".github/CODEOWNERS";
    if inputs.identity.contains_key(codeowners)
        && !any_line(codeowners, |l| {
            l.first().is_some_and(|&b| b != b'#' && !is_space(b))
        })
    {
        report(
            codeowners,
            "no active rule; name the code owner and uncomment the two rules",
        );
    }
    for doc in ["SPECIFICATION", "ARCHITECTURE", "GLOSSARY", "CONVENTIONS"] {
        let file = format!("docs/{doc}.md");
        if any_line(&file, |l| {
            l.starts_with(b"# ") && contains(l, b"<Project Name>")
        }) {
            report(&file, TITLE);
        }
    }
    for &(file, matches, message) in SCAFFOLD {
        if any_line(file, matches) {
            report(file, message);
        }
    }
    found
}

/// `^\[!\[[^]]*\]\([^)]*hivevm/nuc([/)?#]|$)`: a badge whose link names the template's repository.
fn template_badge(line: &[u8]) -> bool {
    let Some(rest) = line.strip_prefix(b"[![") else {
        return false;
    };
    let Some(close) = rest.iter().position(|&b| b == b']') else {
        return false;
    };
    let Some(target) = rest[close + 1..].strip_prefix(b"(") else {
        return false;
    };
    let end = target
        .iter()
        .position(|&b| b == b')')
        .unwrap_or(target.len());
    let name: &[u8] = b"hivevm/nuc";
    (0..=end.saturating_sub(name.len())).any(|at| {
        at + name.len() <= end
            && target[at..].starts_with(name)
            && target
                .get(at + name.len())
                .is_none_or(|b| matches!(b, b'/' | b')' | b'?' | b'#'))
    })
}

/// Whether the byte is a word byte of GNU grep's `\b` in the C locale.
fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Whether the word stands at `at` with a word boundary on both sides.
fn word_at(line: &[u8], at: usize, word: &[u8]) -> bool {
    line[at..].starts_with(word)
        && (at == 0 || !is_word(line[at - 1]))
        && line.get(at + word.len()).is_none_or(|&b| !is_word(b))
}

/// `^Copyright\b.*\bMaintainer\b`.
fn copyright_placeholder(line: &[u8]) -> bool {
    word_at(line, 0, b"Copyright")
        && (b"Copyright".len()..line.len()).any(|at| word_at(line, at, b"Maintainer"))
}

/// `\*\*[GQ]-[0-9]+\*\* — (…|<Quality)`, anywhere on the line.
fn scaffold_criterion(line: &[u8]) -> bool {
    (0..line.len()).any(|at| {
        let Some(rest) = line[at..].strip_prefix(b"**") else {
            return false;
        };
        let Some(rest) = rest
            .strip_prefix(b"G-")
            .or_else(|| rest.strip_prefix(b"Q-"))
        else {
            return false;
        };
        let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
        if digits == 0 {
            return false;
        }
        let Some(rest) = rest[digits..].strip_prefix("** — ".as_bytes()) else {
            return false;
        };
        rest.starts_with("…".as_bytes()) || rest.starts_with(b"<Quality")
    })
}

/// `^- \*\*(Build|Test|Lint|Run):\*\* TODO`.
fn todo_command(line: &[u8]) -> bool {
    let Some(rest) = line.strip_prefix(b"- **") else {
        return false;
    };
    ["Build", "Test", "Lint", "Run"].iter().any(|name| {
        rest.strip_prefix(name.as_bytes())
            .and_then(|r| r.strip_prefix(b":** TODO"))
            .is_some()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(files: &[(&str, &str)], setup: bool) -> Vec<String> {
        let inputs = Inputs {
            setup_file: setup,
            identity: files
                .iter()
                .map(|(p, t)| ((*p).to_owned(), t.as_bytes().to_vec()))
                .collect(),
            ..Inputs::default()
        };
        check(&inputs)
            .into_iter()
            .map(|f| format!("{}: {}", f.file, f.message))
            .collect()
    }

    fn readme(line: &str) -> Vec<String> {
        found(&[("README.md", &format!("# Fixture\n\n{line}\n"))], false)
    }

    /// The cases of check 13 in the template's `scripts/test-check-docs.sh`.
    #[test]
    fn each_placeholder_of_the_bash_self_test_fails_after_setup() {
        for (line, message) in [
            (
                "# NUC — an Agentic Fixture",
                "README.md: the title is still the template's",
            ),
            (
                "**NUC** is named after the small *nucleus colony*.",
                "README.md: the intro still explains",
            ),
            (
                "Describe what this project does, who it is for, and its main goals.",
                "README.md: the Overview is still",
            ),
            (
                "TODO — show a minimal example of using the project.",
                "README.md: the Usage section is still TODO",
            ),
            (
                "[![Checks](https://github.com/hivevm/nuc/actions/workflows/checks.yml/badge.svg)](https://github.com/hivevm/nuc/actions/workflows/checks.yml)",
                "README.md: a badge still points at the template",
            ),
            (
                "[![License](https://img.shields.io/github/license/hivevm/nuc)](LICENSE)",
                "README.md: a badge still points at the template",
            ),
            (
                "- **Build:** TODO <!-- e.g. `make build` -->",
                "README.md: a Build, Test, Lint, or Run command is still TODO",
            ),
            (
                "- **Lint:** TODO",
                "README.md: a Build, Test, Lint, or Run command is still TODO",
            ),
        ] {
            let got = readme(line);
            assert!(
                got.len() == 1 && got[0].starts_with(message),
                "{line}: {got:?}"
            );
        }
        assert_eq!(
            readme("## Why NUC"),
            ["README.md: the Why NUC section still presents the template; delete it"]
        );
    }

    #[test]
    fn a_derived_badge_and_a_link_to_the_template_pass() {
        assert_eq!(
            readme(
                "[![Checks](https://github.com/hivevm/nuc-tools/actions/workflows/checks.yml/badge.svg)](https://github.com/hivevm/nuc-tools/actions/workflows/checks.yml)"
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            readme("Created from [NUC](https://github.com/hivevm/nuc)."),
            Vec::<String>::new()
        );
        assert_eq!(readme("## Why NUC\r"), Vec::<String>::new());
    }

    #[test]
    fn the_placeholders_of_the_other_files_fail() {
        for (file, text, message) in [
            (
                ".devcontainer/devcontainer.json",
                "{ \"name\": \"NUC DevContainer\" }\n",
                "the Dev Container still",
            ),
            (
                "LICENSE",
                "MIT License\n\nCopyright (c) 2026 Maintainer\n",
                "the copyright holder",
            ),
            (
                "LICENSE",
                "Apache License\nVersion 2.0, January 2004\n\nCopyright 2026 Maintainer\n",
                "the copyright holder",
            ),
            (
                "SECURITY.md",
                "TODO: add a security contact\n",
                "the security contact",
            ),
            (
                "CODE_OF_CONDUCT.md",
                "TODO: add a contact address\n",
                "the reporting contact",
            ),
            (
                ".github/CODEOWNERS",
                "# /docs/SPECIFICATION.md @owner\n# /docs/adr/ @owner\n",
                "no active rule",
            ),
            (
                ".github/CODEOWNERS",
                "# TODO: add a code owner and uncomment the rules.\n/docs/adr/ @someone\n",
                "the instruction to add a code owner",
            ),
            (
                "docs/SPECIFICATION.md",
                "# Specification — <Project Name>\n",
                "the title still carries '<Project Name>'",
            ),
            (
                "docs/GLOSSARY.md",
                "# Glossary — <Project Name>\n",
                "the title still carries '<Project Name>'",
            ),
            (
                "docs/SPECIFICATION.md",
                "# Specification — Fixture\n\n## Goals / Success Criteria\n\n1. **G-1** — …\n",
                "a criterion is still",
            ),
            (
                "docs/SPECIFICATION.md",
                "# Specification — Fixture\n\n## Quality Goals\n\n- **Q-1** — <Quality, e.g. Performance>: why.\n",
                "a criterion is still",
            ),
            (
                "docs/ARCHITECTURE.md",
                "# Fixture\n\n- **<Part>** — what it is.\n",
                "the scaffold's building block",
            ),
            (
                "docs/GLOSSARY.md",
                "# Fixture\n\n- **<Term>** — what it is.\n",
                "the scaffold's entry",
            ),
            (
                "docs/CONVENTIONS.md",
                "# Fixture\n\n- **<Convention>** — what it is.\n",
                "the scaffold's entry",
            ),
        ] {
            let got = found(&[(file, text)], false);
            assert!(
                got.len() == 1 && got[0].starts_with(&format!("{file}: {message}")),
                "{file}: {got:?}"
            );
        }
    }

    #[test]
    fn a_tree_still_in_setup_and_one_done_pass() {
        assert_eq!(
            found(&[("README.md", "- **Build:** TODO\n")], true),
            Vec::<String>::new()
        );
        let done = [
            (".github/CODEOWNERS", "# owners\n\n* @someone\n"),
            ("LICENSE", "MIT License\n\nCopyright (c) 2026 Someone\n"),
            (
                "docs/SPECIFICATION.md",
                "# Specification — Fixture\n\n## Goals / Success Criteria\n\n1. **G-1** — When asked, the system shall answer.\n",
            ),
            ("README.md", "# Fixture\n\n- **Build:** `make build`\n"),
        ];
        assert_eq!(found(&done, false), Vec::<String>::new());
    }

    #[test]
    fn the_word_boundaries_read_as_grep_reads_them() {
        assert!(copyright_placeholder(b"Copyright (c) Maintainer"));
        assert!(!copyright_placeholder(b"Copyrighted by Maintainer"));
        assert!(!copyright_placeholder(b"Copyright (c) Maintainers"));
        assert!(!copyright_placeholder(b" Copyright Maintainer"));
        assert!(template_badge(b"[![x](a hivevm/nuc)"));
        assert!(!template_badge(b"[![x](hivevm/nucleus)"));
        assert!(!template_badge(b"[![x] (hivevm/nuc)"));
    }
}
