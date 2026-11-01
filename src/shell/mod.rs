//! The imperative shell: the only module that spawns processes, touches the file system, reads
//! the environment, or prints (ADR-0009). It reads the command line and the
//! repository, calls the rules, and reports what they return.

pub mod git;
mod tree;

use std::backtrace::{Backtrace, BacktraceStatus};
use std::collections::BTreeMap;
use std::env;
use std::io::{self, Write};
use std::panic;
use std::path::Path;
use std::process::ExitCode;

use crate::adr;
use crate::config::{self, Version};
use crate::finding::Finding;
use crate::rules::{self, Carried, Input, Inputs};

use tree::Tree;

/// Exit code of a run that found nothing.
const PASSED: u8 = 0;
/// Exit code of a check that reported findings.
const FOUND: u8 = 1;
/// Exit code of a run that could not decide, whatever the cause: it never passes (Q-1).
const UNDECIDED: u8 = 2;

const USAGE: &str = "usage: nuc check [--repo <path>] | nuc explain <rule ID> | nuc --version";

/// The entry point of the binary: runs the command named on the command line and returns the
/// exit code the process ends with. A panic is an internal error: it names its cause and where it
/// arose, and ends the process with exit 101, never a pass (ADR-0019, Q-1).
pub fn main() -> ExitCode {
    panic::set_hook(Box::new(|info| {
        let at = info
            .location()
            .map(|at| format!(" at {}:{}", at.file(), at.line()))
            .unwrap_or_default();
        let cause = info.payload_as_str().unwrap_or("a panic without a message");
        // Written so that a closed standard error cannot make the hook itself panic.
        let mut err = io::stderr();
        let _ = writeln!(err, "nuc: internal error{at}: {cause} — not decided");
        let trace = Backtrace::capture();
        if trace.status() == BacktraceStatus::Captured {
            let _ = writeln!(err, "{trace}");
        }
    }));
    run()
}

/// The command named on the command line, run.
fn run() -> ExitCode {
    // In a debug build only, the variable makes a panic for the test of Q-1 to show.
    #[cfg(debug_assertions)]
    if env::var_os("NUC_TEST_PANIC").is_some() {
        panic!("NUC_TEST_PANIC is set");
    }
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["check"] => check(None),
        ["check", "--repo", repo] => check(Some(Path::new(repo))),
        ["explain", id] => explain(id),
        ["--version"] => {
            println!("nuc {}", env!("CARGO_PKG_VERSION"));
            Ok(PASSED)
        }
        _ => Err(format!(
            "{USAGE} — a newer nuc may know the command: {}",
            config::INSTALL
        )),
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprintln!("nuc: {why}");
            ExitCode::from(UNDECIDED)
        }
    }
}

/// The version of this `nuc`, which `nuc.toml` sets the floor for.
fn running() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("the package version is X.Y.Z")
}

/// `nuc check`: the rules `nuc.toml` turns on, run over the repository that contains `repo`, or
/// the directory `nuc` runs in.
fn check(repo: Option<&Path>) -> Result<u8, String> {
    let start = match repo {
        Some(repo) => repo.to_path_buf(),
        None => {
            env::current_dir().map_err(|e| format!("the current directory is unreadable: {e}"))?
        }
    };
    let root = git::toplevel(&start)?;
    let tree = Tree::new(root)?;
    let text = tree
        .read(&tree.root.join("nuc.toml"))?
        .ok_or_else(|| config::Refusal::Missing.to_string())?;
    let config = config::load(&text, running()).map_err(|refusal| refusal.to_string())?;
    let selected = config::select(&config, rules::ALL).map_err(|r| r.to_string())?;
    // Only what a rule turned on reads: an input no rule needs can neither refuse nor cost.
    let needs = |input: Input| selected.iter().any(|rule| rule.reads.contains(&input));
    let mut inputs = Inputs::default();
    let mut unplaced = false;
    if needs(Input::Readme) {
        inputs.readme = tree.read_checked(&tree.root.join("README.md"))?;
    }
    if needs(Input::Release) {
        let reported = selected
            .iter()
            .any(|rule| rule.id == rules::docs_template_release::RULE_ID);
        match carried_release(&tree, reported)? {
            Some(release) => inputs.release = release,
            None => unplaced = true,
        }
    }
    if needs(Input::Adr) {
        inputs.adr = tree.read_adr(&tree.root.join("docs/adr"))?;
        inputs.adr_targets = tree.read_adr_targets(&inputs.adr)?;
    }
    if needs(Input::TextFiles) {
        inputs.text_files = tree.read_text_files()?;
    }
    if needs(Input::LinkTargets) {
        inputs.link_targets = tree.read_link_targets(&inputs.text_files, &inputs.adr)?;
    }
    if needs(Input::Agents) {
        let agents = tree.root.join("AGENTS.md");
        inputs.agents = if agents.is_file() {
            tree.read_bytes(&agents)?
        } else {
            None
        };
    }
    if needs(Input::CheckLists) {
        inputs.check_lists = tree.read_check_lists()?;
    }
    if needs(Input::Layout) {
        inputs.layout = tree.read_layout(inputs.readme.as_deref())?;
    }
    if needs(Input::Skills) {
        inputs.skills = tree.read_skills()?;
    }
    if needs(Input::SetupFile) {
        inputs.setup_file = tree.root.join("TEMPLATE-SETUP.md").is_file();
    }
    if needs(Input::Generated) {
        inputs.generated = generated(&tree, &inputs.text_files)?;
        undecided_widths(&inputs)?;
    }
    if needs(Input::Landing) {
        inputs.landing = tree.read_landing()?;
    }
    if needs(Input::Specification) {
        inputs.specification = tree.read_specification()?;
    }
    if needs(Input::Markers) {
        inputs.markers = tree.read_markers()?;
    }
    // While the setup file exists the identity rule is silent, so it reads nothing (check 13).
    if needs(Input::Identity) && !inputs.setup_file {
        inputs.identity = tree.read_identity()?;
    }
    let findings: Vec<Finding> = selected
        .iter()
        .flat_map(|rule| {
            if unplaced && rule.reads.contains(&Input::Release) {
                on_every_release(rule, &inputs)
            } else {
                (rule.check)(&inputs)
            }
        })
        .collect();
    for finding in &findings {
        println!("{finding}");
    }
    eprintln!("{}", summary(&selected, &inputs, findings.len()));
    Ok(if findings.is_empty() { PASSED } else { FOUND })
}

/// The closing line of `nuc check`: whether it passed, and, while the coverage rule runs, the
/// criteria no test cites yet, as the bash's traceability check names them. They are pending
/// work and never change the exit code.
fn summary(selected: &[&rules::Rule], inputs: &Inputs, found: usize) -> String {
    let mut line = if found == 0 {
        format!("nuc check passed ({} rules)", selected.len())
    } else {
        format!("nuc check: {found} finding(s)")
    };
    if selected
        .iter()
        .any(|rule| rule.id == rules::traceability_coverage::RULE_ID)
    {
        let pending = rules::traceability_coverage::pending(inputs);
        if !pending.is_empty() {
            line.push_str(&format!(
                "; {} criteria without a test: {}",
                pending.len(),
                pending.join(" ")
            ));
        }
    }
    line.push('.');
    line
}

/// The Markdown files with a wide line that git's attributes mark `linguist-generated`, as the
/// bash asks `git check-attr` for them and reads its answer: a path git quotes is not one.
fn generated(
    tree: &Tree,
    text_files: &BTreeMap<String, Vec<u8>>,
) -> Result<std::collections::BTreeSet<String>, String> {
    let wide: Vec<&str> = text_files
        .iter()
        .filter(|(file, text)| {
            file.ends_with(".md") && !rules::docs_prose_width::wide_lines(text).is_empty()
        })
        .map(|(file, _)| file.as_str())
        .collect();
    if wide.is_empty() {
        return Ok(std::collections::BTreeSet::new());
    }
    let mut args = vec!["check-attr", "linguist-generated", "--"];
    args.extend(&wide);
    let answer = git::run(&tree.root, &args)?;
    Ok(answer
        .lines()
        .filter_map(|line| {
            ["set", "true"].iter().find_map(|value| {
                line.strip_suffix(&format!(": linguist-generated: {value}"))
                    .map(str::to_owned)
            })
        })
        .collect())
}

/// Refuses a wide line the prose rule would measure but cannot count as the bash does (Q-1).
fn undecided_widths(inputs: &Inputs) -> Result<(), String> {
    let inactive = adr::inactive_paths(&inputs.adr);
    for (file, text) in &inputs.text_files {
        if !file.ends_with(".md") || inactive.contains(file) || inputs.generated.contains(file) {
            continue;
        }
        if let Some(n) = rules::docs_prose_width::undecided_lines(text).first() {
            return Err(format!(
                "{file}:{n}: a line over 100 bytes holds a byte of no valid UTF-8 or the byte 0x1F, which the bash counts otherwise — not decided"
            ));
        }
    }
    Ok(())
}

/// The earliest template release whose bash checks the rules port.
const EARLIEST_RELEASE: Version = rules::RELEASES[0];

/// The findings of a rule that checks by the release on a tree whose README names none: those it
/// reports on every release it knows, which no release could take back.
fn on_every_release(rule: &rules::Rule, inputs: &Inputs) -> Vec<Finding> {
    let runs: Vec<Vec<Finding>> = rules::RELEASES
        .iter()
        .map(|&release| {
            let inputs = Inputs {
                release: Some(release),
                ..inputs.clone()
            };
            (rule.check)(&inputs)
        })
        .collect();
    let (first, rest) = runs.split_first().expect("the rules know a release");
    first
        .iter()
        .filter(|finding| rest.iter().all(|run| run.contains(finding)))
        .cloned()
        .collect()
}

/// The template release the tree carries, for the rules whose bash check changed between
/// releases: none for `unreleased`, checked as the newest release. A release before the earliest
/// is not decided; so is a README that names none, unless the rule `reported` on, the Template
/// release rule, fails the run on it: then a rule reports only what it reports on every release
/// (ADR-0021). The outer none is that case, a release the
/// README does not name.
fn carried_release(tree: &Tree, reported: bool) -> Result<Option<Option<Version>>, String> {
    let readme = tree.read_checked(&tree.root.join("README.md"))?;
    match readme.as_deref().and_then(rules::docs_template_release::carried) {
        Some(Carried::Unreleased) => Ok(Some(None)),
        // The Template release rule reports a README that names no release, never a missing one.
        None if reported && readme.is_some() => Ok(None),
        Some(Carried::Release(release)) if release >= EARLIEST_RELEASE => Ok(Some(Some(release))),
        Some(Carried::Release(release)) => Err(format!(
            "README.md: template release v{release} is before v{EARLIEST_RELEASE}, the earliest whose checks this nuc ports — not decided"
        )),
        None => Err(
            "README.md: no Template release line names the release the tree carries, which a rule turned on checks by — not decided"
                .to_owned(),
        ),
    }
}

/// `nuc explain <id>`: the reason of the rule (G-6).
fn explain(id: &str) -> Result<u8, String> {
    let rule = rules::find(id).ok_or_else(|| format!("no rule '{id}' is known to this nuc"))?;
    println!("{}: {}", rule.id, rule.explain);
    Ok(PASSED)
}
