//! The imperative shell: the only module that spawns processes, touches the file system, reads
//! the environment, or prints (ARCH-rules-outcome-hooks-answer). It reads the command line and the
//! repository, calls the rules, and reports what they return.

pub mod git;
mod hook;
mod measure;
mod tree;

use std::backtrace::{Backtrace, BacktraceStatus};
use std::collections::BTreeMap;
use std::env;
use std::io::{self, Write};
use std::panic;
use std::path::Path;
use std::process::ExitCode;

use crate::config::{self, Version};
use crate::finding::Finding;
use crate::format::{self, Format};
use crate::rules::{self, Carried, Input, Inputs, Outcome};

use tree::Tree;

/// Exit code of a run that found nothing.
const PASSED: u8 = 0;
/// Exit code of a check that reported findings.
const FOUND: u8 = 1;
/// Exit code of a run that could not decide, whatever the cause: it never passes (Q-1).
const UNDECIDED: u8 = 2;
/// Exit code of `nuc hook` alone: the hook passed so far, and its shim runs the check now
/// (API-command-line-with-measure).
const RUN_CHECK: u8 = 3;
/// Exit code of `nuc hook pre-commit` alone: the hook passed so far, and its shim runs the
/// documentation check now (API-command-line-with-measure).
const RUN_DOCS: u8 = 4;

const USAGE: &str = "usage: nuc check [--repo <path>] [--format text|json|sarif] | nuc hook <name> | nuc \
                     measure [--harness claude-code] [--transcripts <dir>] [--branch <name>]... \
                     [--since <time>] [--until <time>] [--breakdown] [--finding] | nuc explain <rule ID> | nuc \
                     --version";

/// The entry point of the binary: runs the command named on the command line and returns the exit
/// code the process ends with. A panic is an internal error: it names its cause and where it arose,
/// and ends the process with exit 101, never a pass (API-command-line-with-measure, Q-1).
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
        ["check", options @ ..] => match check_options(options) {
            Some((repo, format)) => check(repo.map(Path::new), format),
            None => Err(usage()),
        },
        ["hook", name, rest @ ..] => hook::run(name, rest),
        ["measure", options @ ..] => measure::run(options),
        ["explain", id] => explain(id),
        ["--version"] => {
            println!("nuc {}", env!("CARGO_PKG_VERSION"));
            Ok(PASSED)
        }
        _ => Err(usage()),
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprintln!("nuc: {why}");
            ExitCode::from(UNDECIDED)
        }
    }
}

/// The usage error: the commands this `nuc` knows, and how to get one that knows more.
fn usage() -> String {
    format!(
        "{USAGE} — a newer nuc may know the command: {}",
        config::INSTALL
    )
}

/// The options of `nuc check`, each at most once and in any order: the repository and the format,
/// text where none is named; none where an option is unknown, repeated, or has no value.
fn check_options<'a>(options: &[&'a str]) -> Option<(Option<&'a str>, Format)> {
    let (mut repo, mut format) = (None, None);
    let mut options = options.iter();
    while let Some(&option) = options.next() {
        let value = *options.next()?;
        match option {
            "--repo" if repo.is_none() => repo = Some(value),
            "--format" if format.is_none() => format = Some(Format::parse(value)?),
            _ => return None,
        }
    }
    Some((repo, format.unwrap_or(Format::Text)))
}

/// The version of this `nuc`, which `nuc.toml` sets the floor for.
fn running() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("the package version is X.Y.Z")
}

/// `nuc check`: the rules `nuc.toml` turns on, run over the repository that contains `repo`, or
/// the directory `nuc` runs in, the findings written in `format`.
fn check(repo: Option<&Path>, format: Format) -> Result<u8, String> {
    let start = match repo {
        Some(repo) => repo.to_path_buf(),
        None => {
            env::current_dir().map_err(|e| format!("the current directory is unreadable: {e}"))?
        }
    };
    let root = git::toplevel(&start)?;
    let tree = Tree::new(root)?;
    let text = tree
        .read(&tree.root.join("nuc.toml"))
        .map_err(config::unreadable)?
        .ok_or_else(|| config::Refusal::Missing.to_string())?;
    // A hook rule decides in `nuc hook` alone: its ID is known, and it is in no `rules::ALL`, so
    // it runs never here.
    let selected =
        config::turned_on(&text, running(), rules::ALL).map_err(|refusal| refusal.to_string())?;
    // Only what a rule turned on reads: an input no rule needs can neither refuse nor cost.
    let needs = |input: Input| selected.iter().any(|rule| rule.reads.contains(&input));
    let mut inputs = Inputs::default();
    let mut unplaced = false;
    // The text files whose paths are no UTF-8, which no finding can name as the bash does.
    let mut unspelled = std::collections::BTreeSet::new();
    // A tree without a README names no release, so no rule can say which bash it checks as,
    // whatever rules are turned on (API-version-coupling-hook-defaults).
    let readme = tree
        .read_checked(&tree.root.join("README.md"))?
        .ok_or("README.md: not found — the tree names no template release — not decided")?;
    // The release its Template release line names, read once, for every use of it.
    let carried = rules::docs_template_release::carried(&readme);
    if needs(Input::Readme) {
        inputs.readme = Some(readme);
    }
    if needs(Input::Release) {
        let reported = selected
            .iter()
            .any(|rule| rule.id == rules::docs_template_release::RULE_ID);
        match carried_release(carried, reported)? {
            Some(release) => inputs.release = release,
            None => unplaced = true,
        }
    }
    read_guides(&tree, &mut inputs, unplaced, &needs)?;
    if needs(Input::Adr) {
        read_record(&tree, carried.as_ref(), &mut inputs)?;
    }
    if needs(Input::TextFiles) {
        unspelled = read_text(
            &tree,
            &mut inputs,
            needs(Input::LinkTargets),
            carried.as_ref(),
        )?;
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
    if needs(Input::Skills) {
        inputs.skills = tree.read_skills(named_record(carried.as_ref()))?;
    }
    if needs(Input::SetupFile) {
        inputs.setup_file = tree.root.join("TEMPLATE-SETUP.md").is_file();
    }
    if needs(Input::Generated) {
        inputs.generated = generated(&tree, &inputs.text_files)?;
    }
    if needs(Input::Landing) {
        inputs.landing = tree.read_landing()?;
    }
    read_by_git(&tree, &mut inputs, &needs, carried.as_ref())?;
    read_traceability(&tree, &mut inputs, &needs, carried.as_ref())?;
    // While the setup file exists the identity rule is silent, so it reads nothing (check 13).
    if needs(Input::Identity) && !inputs.setup_file {
        inputs.identity = tree.read_identity()?;
    }
    let (findings, notes) = run_rules(&selected, &inputs, unplaced)?;
    if let Some(finding) = findings.iter().find(|f| unspelled.contains(&f.file)) {
        return Err(format!(
            "{}: a finding names a path that is no UTF-8, which nuc cannot spell as the bash does — not decided",
            finding.file
        ));
    }
    let rules: Vec<_> = selected
        .iter()
        .map(|rule| (rule.id, rule.explain))
        .collect();
    let report = format::Report {
        findings: &findings,
        notes: &notes,
        rules: &rules,
        version: env!("CARGO_PKG_VERSION"),
    };
    print!("{}", format::document(format, &report));
    eprintln!("{}", summary(selected.len(), findings.len(), &notes));
    Ok(if findings.is_empty() { PASSED } else { FOUND })
}

/// The outcome of every rule turned on, carried out alike: the first refusal ends the run not
/// decided before any finding is printed (Q-1); else the findings and the notes of them all.
fn run_rules(
    selected: &[&rules::Rule],
    inputs: &Inputs,
    unplaced: bool,
) -> Result<(Vec<Finding>, Vec<String>), String> {
    let outcomes: Vec<Outcome> = selected
        .iter()
        .map(|rule| {
            if unplaced && rule.reads.contains(&Input::Release) {
                on_every_release(rule, inputs)
            } else {
                (rule.check)(inputs)
            }
        })
        .collect();
    if let Some(refusal) = selected
        .iter()
        .zip(&outcomes)
        .find_map(|(rule, outcome)| outcome.refusal_of(rule.id, rule.reasons))
    {
        return Err(refusal);
    }
    let mut findings = Vec::new();
    let mut notes = Vec::new();
    for outcome in outcomes {
        findings.extend(outcome.findings);
        notes.extend(outcome.notes);
    }
    Ok((findings, notes))
}

/// The closing line of `nuc check`: whether it passed, and the notes of the rules, as the criteria
/// no test cites yet. Notes are no findings and never change the exit code.
fn summary(rules: usize, found: usize, notes: &[String]) -> String {
    let mut line = if found == 0 {
        format!("nuc check passed ({rules} rules)")
    } else {
        format!("nuc check: {found} finding(s)")
    };
    for note in notes {
        line.push_str("; ");
        line.push_str(note);
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

/// The guide that holds the required checks and the Project Layout, the README before v0.7.0 and
/// `CONTRIBUTING.md` from it on, and the paths of its layout: only the one the bash of the carried
/// release reads is read, and both where the release is not placed.
fn read_guides(
    tree: &Tree,
    inputs: &mut Inputs,
    unplaced: bool,
    needs: &dyn Fn(Input) -> bool,
) -> Result<(), String> {
    let guide_is_contributing = rules::guide_is_contributing(inputs.release);
    if needs(Input::Contributing) && (unplaced || guide_is_contributing) {
        inputs.contributing = tree.read_checked(&tree.root.join("CONTRIBUTING.md"))?;
    }
    if needs(Input::Layout) {
        let readme = inputs
            .readme
            .as_deref()
            .filter(|_| unplaced || !guide_is_contributing);
        inputs.layout = tree.read_layout(&[
            ("README.md", readme),
            ("CONTRIBUTING.md", inputs.contributing.as_deref()),
        ])?;
    }
    Ok(())
}

/// The Template release the README at `root` names, read through the tree as every command reads
/// the repository; none where the README is missing or its line names no release. `nuc hook` and
/// `nuc measure` ask it alike; each decides what a refusal of the read means for it.
fn template_release(root: &Path) -> Result<Option<Carried>, String> {
    let tree = Tree::new(root.to_path_buf())?;
    let readme = tree.read_checked(&root.join("README.md"))?;
    Ok(readme.and_then(|readme| rules::docs_template_release::carried(&readme)))
}

/// The earliest template release whose bash checks the rules port.
const EARLIEST_RELEASE: Version = rules::RELEASES[0];

/// The findings of a rule that checks by the release on a tree whose README names none: those it
/// reports on every release it knows, which no release could take back. A finding is the same on
/// two releases when only the citation that ends its message differs, since a later release cites
/// a renamed ADR; the message is the newest release's.
fn on_every_release(rule: &rules::Rule, inputs: &Inputs) -> Outcome {
    let runs: Vec<Outcome> = rules::RELEASES
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
    let newest = runs.last().expect("the rules know a release");
    let same = |a: &Finding, b: &Finding| {
        (a.rule, a.reason, &a.file, a.line, uncited(&a.message))
            == (b.rule, b.reason, &b.file, b.line, uncited(&b.message))
    };
    Outcome {
        findings: newest
            .findings
            .iter()
            .filter(|finding| {
                runs.iter()
                    .all(|run| run.findings.iter().any(|other| same(finding, other)))
            })
            .cloned()
            .collect(),
        refusal: runs.iter().find_map(|run| run.refusal.clone()),
        notes: first
            .notes
            .iter()
            .filter(|note| rest.iter().all(|run| run.notes.contains(note)))
            .cloned()
            .collect(),
    }
}

/// A message without the citation that ends it, ` (ADR-NNNN)` or a group with a section sign such
/// as ` (<ADR> §N)`; any other message as it is. It holds by a convention of the rules: a closing
/// group with a section sign is a fixed citation, and what tells two findings apart stands before.
fn uncited(message: &str) -> &str {
    let Some(open) = message.strip_suffix(')').and_then(|m| m.rfind(" (")) else {
        return message;
    };
    let cited = &message[open + 2..message.len() - 1];
    let numbered = cited
        .strip_prefix("ADR-")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
    if numbered || cited.contains('\u{a7}') {
        &message[..open]
    } else {
        message
    }
}

/// The template release the tree carries, as its README's line names it, for the rules whose bash
/// check changed between releases: none for `unreleased`, checked as the newest release. A release
/// before the earliest is not decided; so is a README that names none, unless the rule `reported`
/// on, the Template release rule, fails the run on it: then a rule reports only what it reports on
/// every release (API-version-coupling-hook-defaults). The outer none is that case, a release the
/// README does not name.
fn carried_release(
    carried: Option<Carried>,
    reported: bool,
) -> Result<Option<Option<Version>>, String> {
    match carried {
        Some(Carried::Unreleased) => Ok(Some(None)),
        None if reported => Ok(None),
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

/// Every text file, and the targets of their relative links where `links` asks for them; the paths
/// that are no UTF-8, which no finding can name as the bash does.
fn read_text(
    tree: &Tree,
    inputs: &mut Inputs,
    links: bool,
    carried: Option<&rules::Carried>,
) -> Result<std::collections::BTreeSet<String>, String> {
    let pinned = if pins_left_out(carried) {
        tree.read_pinned()?
    } else {
        std::collections::BTreeSet::new()
    };
    let text = tree.read_text_files(&pinned)?;
    inputs.text_files = text.files;
    if links {
        inputs.link_targets =
            tree.read_link_targets(&inputs.text_files, &inputs.adr, named_record(carried))?;
    }
    Ok(text.unspelled)
}

/// What the rules of the area `traceability` read: the specification, the markers, and, from
/// template release v0.9.0 on, where the bash checks them, the sensors.
fn read_traceability(
    tree: &Tree,
    inputs: &mut Inputs,
    needs: &dyn Fn(Input) -> bool,
    carried: Option<&rules::Carried>,
) -> Result<(), String> {
    if needs(Input::Specification) {
        inputs.specification = tree.read_specification()?;
    }
    if needs(Input::Markers) {
        inputs.markers = tree.read_markers()?;
    }
    if needs(Input::Sensors) && named_record(carried) {
        inputs.sensors = tree.read_sensors()?;
    }
    Ok(())
}

/// The entries of git's index that `shell/mode` reads, as the bash's `git ls-files -s -z` lists
/// them for `scripts/*.sh` and `.agents/skills/*/*.sh`, whose `*` matches a `/` too: each
/// `<mode> <object> <stage>\t<path>`. A path that is no UTF-8 is read with U+FFFD in place of
/// its bytes, which the rule refuses to spell only where it would report the path.
fn read_index_modes(root: &Path) -> Result<Vec<rules::shell_mode::IndexEntry>, String> {
    let out = git::run_bytes(
        root,
        &[
            "ls-files",
            "-s",
            "-z",
            "--",
            "scripts/*.sh",
            ".agents/skills/*/*.sh",
        ],
    )?;
    let mut entries = Vec::new();
    for record in out.split(|&b| b == 0).filter(|r| !r.is_empty()) {
        let Some(tab) = record.iter().position(|&b| b == b'\t') else {
            return Err("git ls-files -s wrote an entry without a tab — not decided".to_owned());
        };
        let meta = String::from_utf8_lossy(&record[..tab]);
        let path = String::from_utf8_lossy(&record[tab + 1..]);
        let fields: Vec<&str> = meta.split(' ').collect();
        let [mode, _, stage] = fields[..] else {
            return Err(format!("git ls-files -s wrote '{meta}' — not decided"));
        };
        entries.push(rules::shell_mode::IndexEntry {
            path: path.into_owned(),
            mode: mode.to_owned(),
            stage: stage.to_owned(),
        });
    }
    Ok(entries)
}

/// The rule keys a header of `scripts/check-docs.sh` lists, as the bash reads its own from template
/// release v0.10.0 on: up to the first line that opens with `set `, each line `#   <area>/<name>`,
/// with the old keys a ` (was: <keys>)` after it names.
fn rule_keys(text: &[u8]) -> std::collections::BTreeSet<String> {
    static ITEM: std::sync::OnceLock<regex::bytes::Regex> = std::sync::OnceLock::new();
    let item = ITEM.get_or_init(|| {
        regex::bytes::Regex::new(r"(?-u)^#   ([a-z]+/[a-z0-9-]+)( \(was: ([^)]+)\))?$")
            .expect("a valid pattern")
    });
    let mut keys = std::collections::BTreeSet::new();
    for line in crate::bash::awk_lines(text) {
        if line.starts_with(b"set ") {
            break;
        }
        if let Some(c) = item.captures(line) {
            let words = [c.get(1), c.get(3)].into_iter().flatten();
            for word in words.flat_map(|w| {
                w.as_bytes()
                    .split(|&b| b == b',' || crate::bash::is_space(b))
                    .filter(|w| !w.is_empty())
            }) {
                keys.insert(String::from_utf8_lossy(word).into_owned());
            }
        }
    }
    keys
}

/// What the rules read through git, and the header of `scripts/check-docs.sh`, each where a rule
/// turned on needs it: the blobs of the base, the history of the deleted ADRs, `README.md` at the
/// merge base with `main`, and the rule keys the header lists, none without the script.
fn read_by_git(
    tree: &Tree,
    inputs: &mut Inputs,
    needs: &dyn Fn(Input) -> bool,
    carried: Option<&rules::Carried>,
) -> Result<(), String> {
    if needs(Input::Base) && named_record(carried) {
        inputs.base = tree.read_base()?;
    }
    if needs(Input::History) {
        inputs.history = read_history(&tree.root, carried)?;
    }
    if needs(Input::ReleaseBase) {
        inputs.release_base = read_release_base(&tree.root)?;
    }
    if needs(Input::IndexModes) {
        inputs.index_modes = read_index_modes(&tree.root)?;
    }
    if needs(Input::RuleKeys) {
        let script = tree.root.join("scripts/check-docs.sh");
        inputs.rule_keys = if script.is_file() {
            tree.read_bytes(&script)?.map(|text| rule_keys(&text))
        } else {
            None
        };
    }
    Ok(())
}

/// `README.md` at the merge base of `HEAD` with `origin/main`, else with `main`, as the bash's
/// `main_merge_base` and `git show` read it from template release v0.10.0 on: a candidate that is
/// no commit, or that has no merge base with `HEAD`, is passed over; a merge base without the file
/// gives it empty. `git cat-file` reads the blob, which runs no textconv driver.
fn read_release_base(root: &Path) -> Result<Option<Vec<u8>>, String> {
    for candidate in ["origin/main", "main"] {
        let commit = format!("{candidate}^{{commit}}");
        if git::status_and_output(root, &["rev-parse", "-q", "--verify", &commit])?.0 != 0 {
            continue;
        }
        let (code, out) = git::status_and_output(root, &["merge-base", candidate, "HEAD"])?;
        if code != 0 {
            continue;
        }
        let base = String::from_utf8_lossy(&out).trim().to_owned();
        let (code, blob) =
            git::status_and_output(root, &["cat-file", "blob", &format!("{base}:README.md")])?;
        return Ok(Some(if code == 0 { blob } else { Vec::new() }));
    }
    Ok(None)
}

/// Whether the record is named by category: from template release v0.9.0 on, and for
/// `unreleased` or none.
fn named_record(carried: Option<&rules::Carried>) -> bool {
    match carried {
        Some(rules::Carried::Release(release)) => *release >= rules::CATEGORY_IDENTIFIERS,
        _ => true,
    }
}

/// Whether the files the blob list of the hooks pins are left out of the text files: from
/// template release v0.12.0 on, and for `unreleased` or a README that names none, as the newest
/// release (NUC-hooks-run-nuc §2).
fn pins_left_out(carried: Option<&rules::Carried>) -> bool {
    match carried {
        Some(rules::Carried::Release(release)) => *release >= rules::HOOK_PINS,
        _ => true,
    }
}

/// The files of the record and the targets of its index, named as `named_record` says.
fn read_record(
    tree: &Tree,
    carried: Option<&rules::Carried>,
    inputs: &mut Inputs,
) -> Result<(), String> {
    let named = named_record(carried);
    inputs.adr = tree.read_adr(&tree.root.join("docs/adr"), named)?;
    inputs.adr_targets = tree.read_adr_targets(&inputs.adr, named)?;
    Ok(())
}

/// The ADR files the first-parent history deleted, read by the `git log` the bash runs; none
/// without a commit, or on a tree of a release before v0.9.0. A git that cannot read the history,
/// as one before 2.31 rejects `--diff-merges`, is the rule's finding, not a refusal, as in the
/// bash; its standard error stays out of the list.
fn read_history(
    root: &Path,
    carried: Option<&rules::Carried>,
) -> Result<rules::docs_adr_identifiers::History, String> {
    use rules::docs_adr_identifiers::{History, LOG, deleted};
    // The history matters from template release v0.9.0 on, and for `unreleased` or none.
    if let Some(rules::Carried::Release(release)) = carried
        && *release < rules::CATEGORY_IDENTIFIERS
    {
        return Ok(History::default());
    }
    if git::probe(root, &["rev-parse", "-q", "--verify", "HEAD^{commit}"])?.is_none() {
        return Ok(History::default());
    }
    let output = git::command(root, &LOG)
        .output()
        .map_err(|e| format!("git could not be run ({e}) — install git and put it on the PATH"))?;
    if output.status.success() {
        return Ok(History {
            deleted: deleted(&String::from_utf8_lossy(&output.stdout)),
            error: None,
        });
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(History {
        deleted: std::collections::BTreeMap::new(),
        error: Some(stderr.lines().next().unwrap_or("").to_owned()),
    })
}

/// `nuc explain <id>`: the reason of the rule (G-6).
fn explain(id: &str) -> Result<u8, String> {
    let (id, reason) =
        config::explanation(id).ok_or_else(|| format!("no rule '{id}' is known to this nuc"))?;
    println!("{id}: {reason}");
    Ok(PASSED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_keys_of_the_header_are_read_with_their_old_keys_up_to_set() {
        let text =
            b"#!/usr/bin/env bash\n#   docs/adr-index\n#      prose\n#   docs/adr-identifiers \
                     (was: docs/adr-numbering, docs/old)\nset -uo pipefail\n#   docs/after\n";
        assert_eq!(
            rule_keys(text).into_iter().collect::<Vec<_>>(),
            [
                "docs/adr-identifiers",
                "docs/adr-index",
                "docs/adr-numbering",
                "docs/old"
            ]
        );
    }

    #[test]
    fn a_finding_is_kept_without_a_release_line_only_when_every_release_reports_it() {
        use crate::testing::cat_id;
        fn check(inputs: &Inputs) -> Outcome {
            let at = |reason, line, message: &str| Finding {
                rule: "x/y",
                reason,
                file: "a.md".to_owned(),
                line: Some(line),
                message: message.to_owned(),
            };
            let named = inputs.carries_at_least(rules::CATEGORY_IDENTIFIERS);
            let (cited, other) = if named {
                (
                    format!("({} \u{a7}6)", cat_id("NUC", "x-y")),
                    cat_id("SEC", "y-rule"),
                )
            } else {
                ("(ADR-0003)".to_owned(), "ADR-0099".to_owned())
            };
            vec![
                at("stray", 1, &format!("is stray {cited}")),
                at("unmatched", 2, &format!("reference '{other}' matches none")),
            ]
            .into()
        }
        let rule = rules::Rule {
            id: "x/y",
            reasons: &["stray", "unmatched"],
            explain: "",
            reads: &[],
            check,
        };
        let kept: Vec<String> = on_every_release(&rule, &Inputs::default())
            .findings
            .into_iter()
            .map(|f| f.message)
            .collect();
        assert_eq!(
            kept,
            [format!("is stray ({} \u{a7}6)", cat_id("NUC", "x-y"))]
        );
        assert_eq!(uncited("no (ADR-0003)"), "no");
        assert_eq!(uncited("no (see a.md)"), "no (see a.md)");
        assert_eq!(uncited("no (ADR-x)"), "no (ADR-x)");
    }

    #[test]
    fn a_rule_that_reports_under_another_id_ends_the_run_not_decided() {
        let foreign = rules::Rule {
            id: "x/a",
            reasons: &["r"],
            explain: "",
            reads: &[],
            check: |_| {
                vec![Finding {
                    rule: "x/b",
                    reason: "r",
                    file: "a.md".to_owned(),
                    line: None,
                    message: "m".to_owned(),
                }]
                .into()
            },
        };
        let refusal = run_rules(&[&foreign], &Inputs::default(), false).expect_err("not decided");
        assert!(
            refusal.starts_with("x/a: reports a finding under the rule ID 'x/b'"),
            "{refusal}"
        );
    }
}
