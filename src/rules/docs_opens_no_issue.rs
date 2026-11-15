//! The check `docs/opens-no-issue` of the template's `scripts/check-docs.sh` from v0.10.0 on:
//! while `TEMPLATE-SETUP.md` exists, no file of the template under `scripts/`, `.githooks/`,
//! `.github/`, or `.agents/`, the fixture tables aside, opens an issue on its own
//! (NUC-template-learns-from-projects §4). A tree of an earlier release has no such check.

use super::{Input, Inputs, Rule, TEMPLATE_FINDINGS};
use crate::bash;
use crate::finding::Finding;
use regex::bytes::Regex;
use std::sync::OnceLock;

pub const RULE_ID: &str = "docs/opens-no-issue";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    reasons: &["cli", "api", "graphql", "http", "script"],
    explain: "While TEMPLATE-SETUP.md exists, no script, hook, workflow, or skill of the template \
              opens an issue on its own: by 'gh issue create', a POST to the issues endpoint \
              through 'gh api', curl, or wget, the GraphQL mutation createIssue, or a script or \
              action that creates one. Only a human sends a finding about the template, and no \
              file of the template sends anything. A command continued over lines is read as \
              one, and a comment line is not read. From template release v0.10.0 on.",
    reads: &[Input::TextFiles, Input::SetupFile, Input::Release],
    check: |inputs| check(inputs).into(),
};

/// The patterns of the bash's awk, as extended regular expressions whose classes are ASCII's: the
/// awk reads them in the caller's locale, where a letter beyond ASCII beside a word, such as
/// 'égh issue create' in a UTF-8 locale, is not decided alike.
struct Forms {
    cli: Regex,
    api: Regex,
    graphql: Regex,
    http: Regex,
    script: Regex,
    action: Regex,
    path: Regex,
    get: Regex,
    post: Regex,
    field: Regex,
    data: Regex,
}

/// `word(w)` of the awk: the pattern with no letter, digit, '_', or '-' on either side.
fn word(pattern: &str) -> String {
    format!("(?-u)(^|[^[:alnum:]_-]){pattern}([^[:alnum:]_-]|$)")
}

fn forms() -> &'static Forms {
    static FORMS: OnceLock<Forms> = OnceLock::new();
    FORMS.get_or_init(|| {
        let re = |pattern: &str| Regex::new(pattern).expect("a valid pattern");
        Forms {
            cli: re(&word("gh[[:space:]]+issue[[:space:]]+(create|new)")),
            api: re(&word("gh[[:space:]]+api")),
            graphql: re(r"(?-u)(^|[^[:alnum:]_])createIssue[[:space:]]*[(]"),
            http: re(&word("(curl|wget)")),
            script: re(r"(?-u)(^|[^[:alnum:]_])issues\.create[[:space:]]*[(]"),
            action: re(r"(?-u)uses:[[:space:]]*[^[:space:]]*create-(an-)?issue"),
            path: re("(?-u)repos/[^[:space:]]*/issues([[:space:]\"'`]|$)"),
            get: re("(?-u)(-X|--method|--request)[[:space:]=]*GET"),
            post: re("(?-u)(-X|--method|--request)[[:space:]=]*POST"),
            field: re("(?-u)[[:space:]](-f|-F|--field|--raw-field|--input)([[:space:]=]|$)"),
            data: re(
                "(?-u)[[:space:]](-d|--data[a-z-]*|--json|--post-data|--post-file)([[:space:]=]|$)",
            ),
        }
    })
}

/// The form by which a command opens an issue, as the awk's `form` names it, or none.
fn form(command: &[u8]) -> Option<&'static str> {
    let f = forms();
    let to_issues = f.path.is_match(command);
    if f.cli.is_match(command) {
        Some("cli")
    } else if f.api.is_match(command)
        && to_issues
        && !f.get.is_match(command)
        && (f.post.is_match(command) || f.field.is_match(command))
    {
        Some("api")
    } else if f.graphql.is_match(command) {
        Some("graphql")
    } else if f.http.is_match(command)
        && to_issues
        && (f.post.is_match(command) || f.data.is_match(command))
    {
        Some("http")
    } else if f.script.is_match(command) || f.action.is_match(command) {
        Some("script")
    } else {
        None
    }
}

/// Whether the bash reads the file: under `scripts/`, `.githooks/`, `.github/`, or `.agents/`, and
/// not under `scripts/fixtures/`.
fn is_read(path: &str) -> bool {
    !path.starts_with("scripts/fixtures/")
        && ["scripts/", ".githooks/", ".github/", ".agents/"]
            .iter()
            .any(|dir| path.starts_with(dir))
}

/// Each command of the text that opens an issue, by the line it starts on, as the awk reads it: a
/// line ending in a backslash goes on in the next, its backslash replaced by a space; a line that
/// opens with '#' is not read where no command goes on; a command a file ends in the middle of is
/// not read.
fn commands(text: &[u8]) -> Vec<(usize, &'static str)> {
    let mut found = Vec::new();
    let mut command: Vec<u8> = Vec::new();
    let mut at = 0;
    for (i, line) in bash::awk_lines(text).into_iter().enumerate() {
        if command.is_empty() {
            let indent = line.iter().take_while(|&&b| bash::is_space(b)).count();
            if line.get(indent) == Some(&b'#') {
                continue;
            }
            at = i + 1;
        }
        if let Some(head) = line.strip_suffix(b"\\") {
            command.extend_from_slice(head);
            command.push(b' ');
            continue;
        }
        command.extend_from_slice(line);
        if let Some(reason) = form(&command) {
            found.push((at, reason));
        }
        command.clear();
    }
    found
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    if !inputs.carries_at_least(TEMPLATE_FINDINGS) || !inputs.setup_file {
        return Vec::new();
    }
    let mut found = Vec::new();
    for (path, text) in &inputs.text_files {
        if !is_read(path) {
            continue;
        }
        for (line, reason) in commands(text) {
            found.push(Finding {
                rule: RULE_ID,
                reason,
                file: path.clone(),
                line: Some(line),
                message: format!(
                    "opens an issue on its own ({reason}) — only a human sends a finding, and no \
                     file of the template sends anything (NUC-template-learns-from-projects §4)"
                ),
            });
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forms_of(text: &str) -> Vec<(usize, &'static str)> {
        commands(text.as_bytes())
    }

    #[test]
    fn each_form_of_opening_an_issue_is_named_as_the_awk_names_it() {
        assert_eq!(forms_of("gh issue create --title x\n"), [(1, "cli")]);
        assert_eq!(forms_of("x=$(gh  issue new)\n"), [(1, "cli")]);
        assert_eq!(
            forms_of("gh api repos/o/r/issues -f title=x\n"),
            [(1, "api")]
        );
        assert_eq!(
            forms_of("gh api --method POST repos/o/r/issues\n"),
            [(1, "api")]
        );
        assert_eq!(
            forms_of("gh api -X GET repos/o/r/issues -f state=open\n"),
            []
        );
        assert_eq!(
            forms_of("mutation { createIssue (input: {}) }\n"),
            [(1, "graphql")]
        );
        assert_eq!(
            forms_of("curl -d '{}' https://api.github.com/repos/o/r/issues\n"),
            [(1, "http")]
        );
        assert_eq!(
            forms_of("wget --post-data=x https://api.github.com/repos/o/r/issues\n"),
            [(1, "http")]
        );
        assert_eq!(
            forms_of("await github.rest.issues.create({})\n"),
            [(1, "script")]
        );
        assert_eq!(
            forms_of("      - uses: some/create-an-issue@v2\n"),
            [(1, "script")]
        );
    }

    #[test]
    fn a_continued_command_is_read_as_one_from_its_first_line() {
        assert_eq!(
            forms_of("echo\ncurl https://api.github.com/repos/o/r/issues \\\n  --data x\n"),
            [(2, "http")]
        );
        assert_eq!(forms_of("  # gh issue create\n"), []);
        assert_eq!(forms_of("gh issue \\\n# create\n"), []);
        assert_eq!(forms_of("x \\\n  # gh issue create\n"), [(1, "cli")]);
        assert_eq!(forms_of("gh issue create \\"), []);
    }

    #[test]
    fn a_word_beside_another_does_not_count() {
        assert_eq!(forms_of("my-gh issue create\n"), []);
        assert_eq!(forms_of("gh issue created\n"), []);
        assert_eq!(forms_of("xcreateIssue(\n"), []);
    }

    #[test]
    fn only_the_files_of_the_template_are_read_while_the_setup_runs() {
        let text = b"gh issue create\n".to_vec();
        let inputs = |setup| Inputs {
            setup_file: setup,
            text_files: [
                ("scripts/a.sh".to_owned(), text.clone()),
                ("scripts/fixtures/t.txt".to_owned(), text.clone()),
                (".agents/skills/s/SKILL.md".to_owned(), text.clone()),
                ("docs/x.md".to_owned(), text.clone()),
            ]
            .into(),
            ..Inputs::default()
        };
        let files: Vec<String> = check(&inputs(true)).into_iter().map(|f| f.file).collect();
        assert_eq!(files, [".agents/skills/s/SKILL.md", "scripts/a.sh"]);
        assert!(check(&inputs(false)).is_empty());
        let mut earlier = inputs(true);
        earlier.release = Some(crate::config::Version(0, 9, 1));
        assert!(check(&earlier).is_empty());
    }
}
