//! Check 9 of the template's `scripts/check-docs.sh`: the jobs of `.github/workflows/checks.yml`
//! and the required checks the README names agree, the workflows and `scripts/check-all.sh` run
//! the same scripts, every job runs a script, and every script under `scripts/` runs. From
//! v0.6.0 a workflow `check-all.sh` names on a `# Not a gate:` line is exempt from running a
//! script, and a job with a matrix is named by its check runs, `<job> (<values>)`.

use super::{Input, Inputs, Rule, read_lines};
use crate::adr::show;
use crate::bash;
use crate::bash::is_space;
use crate::config::Version;
use crate::finding::Finding;

pub const RULE_ID: &str = "docs/check-lists";

pub const RULE: Rule = Rule {
    id: RULE_ID,
    explain: "The jobs of .github/workflows/checks.yml and the required checks named between the \
              'required-checks' markers of README.md list the same jobs, and the workflows and \
              scripts/check-all.sh run the same scripts. A check that runs in only one of them \
              is a gate somebody believes in and does not have. A job that runs no script runs \
              in CI and never in the local gate, and a script neither list runs is a check \
              nobody runs, unless check-all.sh names it on an '# On instruction:' line. From \
              template release v0.6.0 a workflow named on a '# Not a gate:' line publishes \
              rather than checks, and a job with a matrix is named by its check runs.",
    reads: &[Input::Readme, Input::CheckLists, Input::Release],
    check: |inputs| check(inputs).into(),
};

/// The files check 9 compares, each as `-f` finds it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckLists {
    /// `.github/workflows/checks.yml`.
    pub checks: Option<Vec<u8>>,
    /// `scripts/check-all.sh`.
    pub check_all: Option<Vec<u8>>,
    /// The workflows the bash's globs list, `*.yml` and then `*.yaml`, each sorted, by path.
    pub workflows: Vec<(String, Vec<u8>)>,
    /// The name of every file directly under `.github/workflows/`.
    pub workflow_files: std::collections::BTreeSet<String>,
    /// The name of every regular file `scripts/*.sh`, sorted, as `find -type f` lists them.
    pub scripts: Vec<String>,
}

/// The release from which a publishing workflow is no gate and a matrix job is named by its runs.
const NOT_A_GATE_AND_MATRIX: Version = Version(0, 6, 0);

const CHECKS: &str = ".github/workflows/checks.yml";
const CHECK_ALL: &str = "scripts/check-all.sh";
const README: &str = "README.md";

fn finding(file: &str, message: String) -> Finding {
    Finding {
        rule: RULE_ID,
        file: file.to_owned(),
        line: None,
        message,
    }
}

fn check(inputs: &Inputs) -> Vec<Finding> {
    let lists = &inputs.check_lists;
    let (Some(checks), Some(all), Some(readme)) = (&lists.checks, &lists.check_all, &inputs.readme)
    else {
        let missing = [
            (CHECKS, lists.checks.is_some()),
            (CHECK_ALL, lists.check_all.is_some()),
            (README, inputs.readme.is_some()),
        ]
        .into_iter()
        .find(|(_, present)| !present)
        .map_or(CHECKS, |(file, _)| file);
        return vec![finding(
            missing,
            "not found — the check lists cannot be compared".to_owned(),
        )];
    };
    let v0_6 = inputs.carries_at_least(NOT_A_GATE_AND_MATRIX);
    let mut found = Vec::new();
    let not_gate = if v0_6 {
        not_gate(all, lists, &mut found)
    } else {
        Vec::new()
    };
    let workflows = Workflows::read(lists, checks, &not_gate, v0_6, &mut found);
    if workflows.jobs.is_empty() {
        found.push(finding(
            CHECKS,
            "no jobs found under 'jobs:' — the check lists cannot be compared".to_owned(),
        ));
        return found;
    }
    let all_scripts: Vec<Vec<u8>> = read_lines(all)
        .into_iter()
        .filter_map(run_line_script)
        .map(<[u8]>::to_vec)
        .collect();
    if all_scripts.is_empty() {
        found.push(finding(
            CHECK_ALL,
            "no 'run <label> <script>' invocations found — the check lists cannot be compared"
                .to_owned(),
        ));
        return found;
    }
    for s in &workflows.scripts {
        if !all_scripts.contains(s) {
            found.push(finding(
                CHECK_ALL,
                format!(
                    "does not run '{}', which a workflow runs — CI and the local gate disagree",
                    show(s)
                ),
            ));
        }
    }
    for s in &all_scripts {
        if !workflows.scripts.contains(s) {
            found.push(finding(
                CHECK_ALL,
                format!(
                    "runs '{}', which no job of a workflow runs — CI and the local gate disagree",
                    show(s)
                ),
            ));
        }
    }
    let on_instruction: Vec<&[u8]> = bash::sed_lines(all)
        .filter_map(on_instruction_script)
        .collect();
    for name in &lists.scripts {
        let s = format!("scripts/{name}");
        if s == CHECK_ALL {
            continue;
        }
        let runs =
            all_scripts.iter().any(|a| a == s.as_bytes()) || on_instruction.contains(&s.as_bytes());
        if !runs {
            found.push(finding(&s, format!(
                "no workflow and not {CHECK_ALL} runs it — add it to both lists, or name it on an '# On instruction:' line of {CHECK_ALL}"
            )));
        }
    }
    readme_checks(readme.as_bytes(), &workflows, v0_6, &mut found);
    found
}

/// The workflows named on `# Not a gate:` lines of `check-all.sh`, each one checked: no
/// `checks.yml`, and a file of `.github/workflows/`.
fn not_gate(all: &[u8], lists: &CheckLists, found: &mut Vec<Finding>) -> Vec<String> {
    let mut names = Vec::new();
    for line in bash::sed_lines(all) {
        let Some(name) = not_gate_name(line) else {
            continue;
        };
        let name = show(name);
        if name == CHECKS {
            found.push(finding(
                CHECK_ALL,
                format!("'# Not a gate: {name}' names the workflow of the required checks"),
            ));
        } else if !is_workflow_path(&name)
            || !lists
                .workflow_files
                .contains(&name[".github/workflows/".len()..])
        {
            found.push(finding(
                CHECK_ALL,
                format!("'# Not a gate: {name}' names no workflow of this repository"),
            ));
        }
        names.push(name);
    }
    names
}

/// Whether the name matches `^\.github/workflows/[A-Za-z0-9._-]+\.ya?ml$`.
fn is_workflow_path(name: &str) -> bool {
    let Some(file) = name.strip_prefix(".github/workflows/") else {
        return false;
    };
    let stem = file
        .strip_suffix(".yml")
        .or_else(|| file.strip_suffix(".yaml"));
    stem.is_some_and(|stem| !stem.is_empty() && stem.bytes().all(is_script_byte))
}

/// What the workflows say: the jobs of `checks.yml`, those with a matrix, and every script a
/// step of any workflow runs, in order.
struct Workflows {
    jobs: Vec<Vec<u8>>,
    matrix_jobs: Vec<Vec<u8>>,
    scripts: Vec<Vec<u8>>,
}

impl Workflows {
    fn read(
        lists: &CheckLists,
        checks: &[u8],
        not_gate: &[String],
        v0_6: bool,
        found: &mut Vec<Finding>,
    ) -> Self {
        let mut workflows = Self {
            jobs: Vec::new(),
            matrix_jobs: Vec::new(),
            scripts: Vec::new(),
        };
        for (rel, text) in &lists.workflows {
            let is_checks = rel == CHECKS;
            let text = if is_checks { checks } else { text };
            let is_gate = !not_gate.iter().any(|name| name == rel);
            workflows.read_one(rel, text, is_checks, is_gate, v0_6, found);
        }
        workflows
    }

    fn read_one(
        &mut self,
        rel: &str,
        text: &[u8],
        is_checks: bool,
        is_gate: bool,
        v0_6: bool,
        found: &mut Vec<Finding>,
    ) {
        let mut job: Option<Vec<u8>> = None;
        let mut job_scripts = 0;
        let mut in_jobs = false;
        let mut key: Option<&[u8]> = None;
        let mut job_done = |job: &mut Option<Vec<u8>>, job_scripts: usize| {
            if let Some(name) = job.take()
                && is_gate
                && job_scripts == 0
            {
                found.push(finding(rel, format!(
                    "job '{}' runs no script under scripts/ — the local gate cannot run what it does",
                    show(&name)
                )));
            }
        };
        for line in read_lines(text) {
            if line.starts_with(b"jobs:") {
                in_jobs = true;
                continue;
            }
            if !in_jobs {
                continue;
            }
            if let Some(name) = job_line(line) {
                job_done(&mut job, job_scripts);
                job = Some(name.to_vec());
                job_scripts = 0;
                if is_checks {
                    self.jobs.push(name.to_vec());
                }
                key = None;
                continue;
            }
            if v0_6 {
                if let Some(k) = key_line(line) {
                    key = Some(k);
                }
                let matrix = (key == Some(b"strategy") && indented(line, 6, b"matrix:"))
                    || flow_matrix(line);
                if is_checks
                    && matrix
                    && let Some(name) = &job
                    && !self.matrix_jobs.contains(name)
                {
                    self.matrix_jobs.push(name.clone());
                }
            }
            if let Some(script) = step_script(line) {
                self.scripts.push(script.to_vec());
                job_scripts += 1;
            }
        }
        job_done(&mut job, job_scripts);
    }
}

/// The required checks between the README's markers against the jobs of `checks.yml`.
fn readme_checks(readme: &[u8], workflows: &Workflows, v0_6: bool, found: &mut Vec<Finding>) {
    let mut in_block = false;
    let mut names: Vec<Vec<u8>> = Vec::new();
    for line in read_lines(readme) {
        if crate::bash::contains(line, b"required-checks begin") {
            in_block = true;
            continue;
        }
        if crate::bash::contains(line, b"required-checks end") {
            in_block = false;
            continue;
        }
        if in_block {
            names.extend(check_names(line, v0_6));
        }
    }
    if names.is_empty() {
        found.push(finding(
            README,
            "no job names between the 'required-checks' markers — the required status checks are unverifiable"
                .to_owned(),
        ));
        return;
    }
    let mut bases: Vec<&[u8]> = Vec::new();
    for name in &names {
        let base = crate::bash::find(name, b" (").map_or(name.as_slice(), |at| &name[..at]);
        bases.push(base);
        let (shown, shown_base) = (show(name), show(base));
        let is_job = workflows.jobs.iter().any(|j| j == base);
        let has_matrix = workflows.matrix_jobs.iter().any(|j| j == base);
        let message = if !is_job {
            format!(
                "required check '{shown}' is not a job of {CHECKS} — a ruleset would wait for a check that never reports"
            )
        } else if name.as_slice() == base && has_matrix {
            format!(
                "required check '{shown}' is a job with a matrix, which reports as '{shown} (<values>)' — name each check run"
            )
        } else if name.as_slice() != base && !has_matrix {
            format!(
                "required check '{shown}' names a check run of '{shown_base}', which has no matrix and reports as '{shown_base}'"
            )
        } else {
            continue;
        };
        found.push(finding(README, message));
    }
    for job in &workflows.jobs {
        if !bases.contains(&job.as_slice()) {
            found.push(finding(README, format!(
                "job '{}' of {CHECKS} is missing from the required checks — it would run without gating a merge",
                show(job)
            )));
        }
    }
}

/// The names `grep -oE` finds on the line: `` `[a-z][a-z0-9-]*` ``, and from v0.6.0 with an
/// optional ` (<values>)` before the closing backquote.
fn check_names(line: &[u8], v0_6: bool) -> Vec<Vec<u8>> {
    let mut names = Vec::new();
    let mut i = 0;
    while i < line.len() {
        match check_name_at(&line[i..], v0_6) {
            Some(length) => {
                names.push(line[i + 1..i + length - 1].to_vec());
                i += length;
            }
            None => i += 1,
        }
    }
    names
}

/// The length of a backquoted check name at the start of `s`, the backquotes included.
fn check_name_at(s: &[u8], v0_6: bool) -> Option<usize> {
    if s.first() != Some(&b'`') || !s.get(1).is_some_and(u8::is_ascii_lowercase) {
        return None;
    }
    let name = 2 + s[2..]
        .iter()
        .take_while(|&&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        .count();
    if v0_6 && s[name..].starts_with(b" (") {
        let values = s[name + 2..]
            .iter()
            .take_while(|&&b| !matches!(b, b'`' | b'(' | b')'))
            .count();
        let close = name + 2 + values;
        if values > 0 && s[close..].starts_with(b")`") {
            return Some(close + 2);
        }
    }
    (s.get(name) == Some(&b'`')).then_some(name + 1)
}

/// A job line, `^  ([a-z][a-z0-9-]*):[[:space:]]*$`: its name.
fn job_line(line: &[u8]) -> Option<&[u8]> {
    let rest = line.strip_prefix(b"  ")?;
    if !rest.first()?.is_ascii_lowercase() {
        return None;
    }
    let length = rest
        .iter()
        .take_while(|&&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        .count();
    let after = rest[length..].strip_prefix(b":")?;
    after
        .iter()
        .all(|&b| is_space(b))
        .then_some(&rest[..length])
}

/// A key of the job, `^ {4}([a-z][a-z-]*):`: its name.
fn key_line(line: &[u8]) -> Option<&[u8]> {
    let rest = line.strip_prefix(b"    ")?;
    if !rest.first()?.is_ascii_lowercase() {
        return None;
    }
    let length = rest
        .iter()
        .take_while(|&&b| b.is_ascii_lowercase() || b == b'-')
        .count();
    rest[length..].starts_with(b":").then_some(&rest[..length])
}

/// Whether the line is `^ {n}<word>`: exactly `n` spaces, then the word.
fn indented(line: &[u8], n: usize, word: &[u8]) -> bool {
    line.len() > n && line[..n].iter().all(|&b| b == b' ') && line[n..].starts_with(word)
}

/// Whether the line is a strategy in flow style, `^ {4}strategy:.*[{,][[:space:]]*matrix:`.
fn flow_matrix(line: &[u8]) -> bool {
    if !indented(line, 4, b"strategy:") {
        return false;
    }
    let rest = &line[4 + b"strategy:".len()..];
    rest.iter().enumerate().any(|(i, &b)| {
        matches!(b, b'{' | b',') && {
            let after = &rest[i + 1..];
            let spaces = after.iter().take_while(|&&b| is_space(b)).count();
            after[spaces..].starts_with(b"matrix:")
        }
    })
}

/// Whether the byte is in `[A-Za-z0-9._-]`.
fn is_script_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-')
}

/// The script at the start of `s`, `scripts/[A-Za-z0-9._-]+\.sh` with `+` (or `*` when
/// `empty`) as long as it can be.
fn script_at(s: &[u8], empty: bool) -> Option<&[u8]> {
    let rest = s.strip_prefix(b"scripts/")?;
    let run = rest.iter().take_while(|&&b| is_script_byte(b)).count();
    let least = if empty { 3 } else { 4 };
    (least..=run)
        .rev()
        .find(|&end| rest[..end].ends_with(b".sh"))
        .map(|end| &s[..b"scripts/".len() + end])
}

/// The script a step runs, `run:[[:space:]]+bash[[:space:]]+(scripts/…\.sh)`, leftmost.
fn step_script(line: &[u8]) -> Option<&[u8]> {
    (0..line.len()).find_map(|i| {
        let rest = line[i..].strip_prefix(b"run:")?;
        let rest = space_then(rest, b"bash")?;
        let spaces = rest.iter().take_while(|&&b| is_space(b)).count();
        if spaces == 0 {
            return None;
        }
        script_at(&rest[spaces..], false)
    })
}

/// The text after one or more spaces and then the word.
fn space_then<'a>(s: &'a [u8], word: &[u8]) -> Option<&'a [u8]> {
    let spaces = s.iter().take_while(|&&b| is_space(b)).count();
    if spaces == 0 {
        return None;
    }
    s[spaces..].strip_prefix(word)
}

/// The script a `run "<label>" <script>` line of `check-all.sh` runs.
fn run_line_script(line: &[u8]) -> Option<&[u8]> {
    let spaces = line.iter().take_while(|&&b| is_space(b)).count();
    let rest = line[spaces..].strip_prefix(b"run")?;
    let spaces = rest.iter().take_while(|&&b| is_space(b)).count();
    if spaces == 0 {
        return None;
    }
    let rest = rest[spaces..].strip_prefix(b"\"")?;
    let close = rest.iter().position(|&b| b == b'"')?;
    let rest = &rest[close + 1..];
    let spaces = rest.iter().take_while(|&&b| is_space(b)).count();
    if spaces == 0 {
        return None;
    }
    script_at(&rest[spaces..], false)
}

/// The text after `^#[[:space:]]*<label>[[:space:]]*`.
fn comment_value<'a>(line: &'a [u8], label: &[u8]) -> Option<&'a [u8]> {
    let rest = line.strip_prefix(b"#")?;
    let spaces = rest.iter().take_while(|&&b| is_space(b)).count();
    let rest = rest[spaces..].strip_prefix(label)?;
    let spaces = rest.iter().take_while(|&&b| is_space(b)).count();
    Some(&rest[spaces..])
}

/// The script an `# On instruction:` line names, `scripts/[A-Za-z0-9._-]*\.sh`.
fn on_instruction_script(line: &[u8]) -> Option<&[u8]> {
    script_at(comment_value(line, b"On instruction:")?, true)
}

/// The workflow a `# Not a gate:` line names: everything up to the first space.
fn not_gate_name(line: &[u8]) -> Option<&[u8]> {
    let value = comment_value(line, b"Not a gate:")?;
    Some(&value[..value.iter().take_while(|&&b| !is_space(b)).count()])
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHECKS_YML: &str =
        "on: [push]\njobs:\n  docs:\n    steps:\n      - run: bash scripts/check-docs.sh\n";
    const ALL: &str = "#!/usr/bin/env bash\nrun \"Docs\" scripts/check-docs.sh\n";
    const README_TEXT: &str =
        "# P\n\n<!-- required-checks begin -->\n`docs`\n<!-- required-checks end -->\n";

    struct Tree {
        checks: Option<String>,
        all: Option<String>,
        readme: Option<String>,
        workflows: Vec<(String, String)>,
        scripts: Vec<String>,
        release: Option<Version>,
    }

    impl Tree {
        fn new() -> Self {
            Self {
                checks: Some(CHECKS_YML.to_owned()),
                all: Some(ALL.to_owned()),
                readme: Some(README_TEXT.to_owned()),
                workflows: Vec::new(),
                scripts: vec!["check-all.sh".to_owned(), "check-docs.sh".to_owned()],
                release: Some(NOT_A_GATE_AND_MATRIX),
            }
        }

        fn messages(&self) -> Vec<String> {
            let mut workflows = vec![(
                CHECKS.to_owned(),
                self.checks.clone().unwrap_or_default().into_bytes(),
            )];
            workflows.extend(
                self.workflows
                    .iter()
                    .map(|(p, t)| (p.clone(), t.clone().into_bytes())),
            );
            workflows.sort();
            let inputs = Inputs {
                readme: self.readme.clone(),
                release: self.release,
                check_lists: CheckLists {
                    checks: self.checks.clone().map(String::into_bytes),
                    check_all: self.all.clone().map(String::into_bytes),
                    workflow_files: workflows
                        .iter()
                        .map(|(p, _)| p[".github/workflows/".len()..].to_owned())
                        .collect(),
                    workflows,
                    scripts: self.scripts.clone(),
                },
                ..Inputs::default()
            };
            check(&inputs)
                .into_iter()
                .map(|f| format!("{}: {}", f.file, f.message))
                .collect()
        }
    }

    #[test]
    fn lists_that_agree_pass() {
        assert_eq!(Tree::new().messages(), Vec::<String>::new());
    }

    #[test]
    fn a_missing_file_stops_the_check() {
        let mut tree = Tree::new();
        tree.all = None;
        tree.readme = None;
        assert_eq!(
            tree.messages(),
            ["scripts/check-all.sh: not found — the check lists cannot be compared"]
        );
    }

    #[test]
    fn a_job_missing_from_the_readme_and_a_name_that_is_no_job_fail() {
        let mut tree = Tree::new();
        tree.readme = Some(README_TEXT.replace("`docs`", "`build`"));
        assert_eq!(
            tree.messages(),
            [
                "README.md: required check 'build' is not a job of .github/workflows/checks.yml — a ruleset would wait for a check that never reports",
                "README.md: job 'docs' of .github/workflows/checks.yml is missing from the required checks — it would run without gating a merge",
            ]
        );
    }

    #[test]
    fn scripts_that_run_in_one_list_only_or_in_none_fail() {
        let mut tree = Tree::new();
        tree.all = Some(format!("{ALL}run \"Shell\" scripts/check-shell.sh\n"));
        tree.scripts.push("sensor-x.sh".to_owned());
        tree.workflows.push((
            ".github/workflows/other.yml".to_owned(),
            "jobs:\n  lint:\n    steps:\n      - run: make lint\n  t:\n    steps:\n      - run: bash scripts/test-x.sh\n".to_owned(),
        ));
        // bash: a check the local runner has and CI does not
        // bash: a script that neither list runs
        // bash: a job of another workflow that runs no script
        assert_eq!(
            tree.messages(),
            [
                ".github/workflows/other.yml: job 'lint' runs no script under scripts/ — the local gate cannot run what it does",
                "scripts/check-all.sh: does not run 'scripts/test-x.sh', which a workflow runs — CI and the local gate disagree",
                "scripts/check-all.sh: runs 'scripts/check-shell.sh', which no job of a workflow runs — CI and the local gate disagree",
                "scripts/sensor-x.sh: no workflow and not scripts/check-all.sh runs it — add it to both lists, or name it on an '# On instruction:' line of scripts/check-all.sh",
            ]
        );
        tree.all = Some(format!(
            "{ALL}run \"Shell\" scripts/check-shell.sh\n# On instruction: scripts/sensor-x.sh\n"
        ));
        // bash: a script the local runner names as run on instruction
        assert_eq!(tree.messages().len(), 3);
    }

    #[test]
    fn a_last_line_without_a_newline_is_not_read_as_bash_reads_it() {
        let mut tree = Tree::new();
        tree.all = Some(ALL.trim_end().to_owned());
        assert_eq!(
            tree.messages(),
            [
                "scripts/check-all.sh: no 'run <label> <script>' invocations found — the check lists cannot be compared"
            ]
        );
    }

    fn release(publish: &str) -> (String, String) {
        (
            ".github/workflows/release.yml".to_owned(),
            format!("on: [push]\njobs:\n  publish:\n    steps:\n      - run: make dist\n{publish}"),
        )
    }

    /// The cases of check 9 that v0.6.0 of the template's `scripts/test-check-docs.sh` adds.
    #[test]
    fn from_v0_6_0_a_workflow_named_as_no_gate_runs_no_script() {
        let mut tree = Tree::new();
        tree.workflows.push(release(""));
        let unnamed = tree.messages();
        // bash: a publishing workflow not named as no gate
        assert_eq!(
            unnamed,
            [
                ".github/workflows/release.yml: job 'publish' runs no script under scripts/ — the local gate cannot run what it does"
            ]
        );
        tree.all = Some(format!(
            "{ALL}# Not a gate: .github/workflows/release.yml\n"
        ));
        // bash: a publishing workflow the local runner names as no gate, its steps inline
        assert_eq!(tree.messages(), Vec::<String>::new());
        for (line, message) in [
            // bash: a '# Not a gate:' line that names no workflow
            (
                "# Not a gate: .github/workflows/gone.yml",
                "scripts/check-all.sh: '# Not a gate: .github/workflows/gone.yml' names no workflow of this repository",
            ),
            // bash: a '# Not a gate:' line that names a file outside .github/workflows/
            (
                "# Not a gate: README.md",
                "scripts/check-all.sh: '# Not a gate: README.md' names no workflow of this repository",
            ),
            // bash: a '# Not a gate:' line that names the required checks' workflow
            (
                "# Not a gate: .github/workflows/checks.yml",
                "scripts/check-all.sh: '# Not a gate: .github/workflows/checks.yml' names the workflow of the required checks",
            ),
        ] {
            tree.all = Some(format!(
                "{ALL}# Not a gate: .github/workflows/release.yml\n{line}"
            ));
            assert_eq!(tree.messages(), [message], "{line}");
        }
        tree.workflows = vec![release("      - run: bash scripts/sign.sh\n")];
        tree.all = Some(format!(
            "{ALL}# Not a gate: .github/workflows/release.yml\n"
        ));
        // bash: a script a workflow that is no gate runs, and the local runner does not
        assert_eq!(
            tree.messages(),
            [
                "scripts/check-all.sh: does not run 'scripts/sign.sh', which a workflow runs — CI and the local gate disagree"
            ]
        );
    }

    fn matrix_tree(names: &str, strategy: &str) -> Tree {
        let mut tree = Tree::new();
        tree.checks = Some(format!(
            "{CHECKS_YML}  build:\n{strategy}    steps:\n      - run: bash scripts/check-build.sh\n"
        ));
        tree.all = Some(format!("{ALL}run \"Build\" scripts/check-build.sh\n"));
        tree.scripts.push("check-build.sh".to_owned());
        tree.readme = Some(README_TEXT.replace("`docs`", &format!("`docs`, {names}")));
        tree
    }

    const BLOCK: &str = "    strategy:\n      matrix:\n        os: [ubuntu-latest, macos-latest]\n";
    const FLOW: &str =
        "    strategy: { fail-fast: false, matrix: { os: [ubuntu-latest, macos-latest] } }\n";
    const OUTPUT: &str = "    outputs:\n      matrix: ${{ steps.m.outputs.matrix }}\n";
    const RUNS: &str = "`build (ubuntu-latest)`, `build (macos-latest)`";

    #[test]
    fn from_v0_6_0_a_job_with_a_matrix_is_named_by_its_check_runs() {
        // bash: a job with a matrix named by its check runs
        assert_eq!(matrix_tree(RUNS, BLOCK).messages(), Vec::<String>::new());
        // bash: a strategy in flow style named by its check runs
        assert_eq!(matrix_tree(RUNS, FLOW).messages(), Vec::<String>::new());
        // bash: a job that computes a matrix as an output, named by its name
        assert_eq!(
            matrix_tree("`build`", OUTPUT).messages(),
            Vec::<String>::new()
        );
        // bash: a job with a matrix named by its bare name
        assert_eq!(
            matrix_tree("`build`", BLOCK).messages(),
            [
                "README.md: required check 'build' is a job with a matrix, which reports as 'build (<values>)' — name each check run"
            ]
        );
        // bash: a check run named for a job without a matrix
        assert_eq!(
            matrix_tree("`build (ubuntu-latest)`", "").messages(),
            [
                "README.md: required check 'build (ubuntu-latest)' names a check run of 'build', which has no matrix and reports as 'build'"
            ]
        );
    }

    /// Verifies: ADR-0021
    #[test]
    fn before_v0_6_0_every_workflow_is_a_gate_and_a_check_run_is_no_name() {
        let mut tree = matrix_tree(RUNS, BLOCK);
        tree.release = Some(Version(0, 5, 1));
        assert_eq!(
            tree.messages(),
            [
                "README.md: job 'build' of .github/workflows/checks.yml is missing from the required checks — it would run without gating a merge"
            ]
        );
        let mut tree = matrix_tree("`build`", BLOCK);
        tree.release = Some(Version(0, 5, 0));
        assert_eq!(tree.messages(), Vec::<String>::new());
        let mut tree = Tree::new();
        tree.release = Some(Version(0, 5, 0));
        tree.workflows.push(release(""));
        tree.all = Some(format!(
            "{ALL}# Not a gate: .github/workflows/release.yml\n"
        ));
        assert_eq!(
            tree.messages(),
            [
                ".github/workflows/release.yml: job 'publish' runs no script under scripts/ — the local gate cannot run what it does"
            ]
        );
    }

    #[test]
    fn the_patterns_read_as_the_bash_reads_them() {
        assert_eq!(
            step_script(b"      - run:  bash  scripts/a.sh.sh --x"),
            Some(&b"scripts/a.sh.sh"[..])
        );
        assert_eq!(step_script(b"- run: bash scripts/.sh"), None);
        assert_eq!(
            on_instruction_script(b"#On instruction:scripts/.sh"),
            Some(&b"scripts/.sh"[..])
        );
        assert_eq!(job_line(b"  build-1:\r"), Some(&b"build-1"[..]));
        assert_eq!(job_line(b"   build:"), None);
        assert_eq!(
            check_names(b"`docs` `Build` ` x` `a (b)` `c ()` `d (e`", true),
            [b"docs".to_vec(), b"a (b)".to_vec()]
        );
        assert_eq!(check_names(b"`a (b)` `c`", false), [b"c".to_vec()]);
    }

    // bash: a toolchain job with its commands inline and nothing in the local runner
    #[test]
    fn a_toolchain_job_with_its_commands_inline_runs_no_script() {
        let mut tree = Tree::new();
        tree.checks = Some(format!(
            "{CHECKS_YML}  build:\n    steps:\n      - uses: actions/setup-something@v1\n      - run: make test\n"
        ));
        tree.readme = Some(README_TEXT.replace("`docs`", "`docs`, `build`"));
        assert_eq!(
            tree.messages(),
            [
                ".github/workflows/checks.yml: job 'build' runs no script under scripts/ — the local gate cannot run what it does"
            ]
        );
    }

    /// Three more cases of check 9 in the template's `scripts/test-check-docs.sh`.
    #[test]
    fn another_workflow_and_a_setup_step_read_as_the_bash_reads_them() {
        let other = (
            ".github/workflows/other.yml".to_owned(),
            "jobs:\n  lint:\n    steps:\n      - run: bash scripts/check-docs.sh\n".to_owned(),
        );
        let mut tree = Tree::new();
        tree.workflows.push(other.clone());
        // bash: another workflow that runs a script the local runner runs, its job not required
        assert_eq!(tree.messages(), Vec::<String>::new());
        tree.readme = Some(README_TEXT.replace("`docs`", "`docs`, `lint`"));
        // bash: a required check that is a job of another workflow than checks.yml
        assert_eq!(
            tree.messages(),
            [
                "README.md: required check 'lint' is not a job of .github/workflows/checks.yml — a ruleset would wait for a check that never reports"
            ]
        );
        // bash: a toolchain job that runs the build script after a setup step
        let mut tree = Tree::new();
        tree.checks = Some(
            "jobs:\n  docs:\n    steps:\n      - uses: actions/checkout@v5\n      - run: bash scripts/check-docs.sh\n"
                .to_owned(),
        );
        assert_eq!(tree.messages(), Vec::<String>::new());
    }
}
