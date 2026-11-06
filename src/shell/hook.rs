//! `nuc hook <name>`: the git state a hook decides on, read through git, and the answer the
//! reference shim acts on (ADR-0021). It runs no command but git and no script of the repository
//! (Q-7): where a hook's check runs, it exits 3 and the shim runs it.

use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::config::{self, Refusal};
use crate::hook::{self, Answer, HookRule};
use crate::rules;

use super::tree::Tree;
use super::{FOUND, PASSED, RUN_CHECK, git, running};

/// How a hook decides on the state of a commit, with the hook rules turned on.
type Decide = fn(&hook::Commit, &dyn Fn(&HookRule) -> bool) -> Answer;

/// The hooks this `nuc` decides, by the name git gives them, each with the check script its shim
/// runs on 3 and its decision.
const HOOKS: &[(&str, &str, Decide)] = &[
    ("pre-commit", "scripts/check-all.sh", hook::pre_commit),
    (
        "pre-merge-commit",
        "scripts/check-docs.sh",
        hook::pre_merge_commit,
    ),
];

/// `nuc hook <name>`, run in the directory git runs the hook in.
pub(super) fn run(name: &str, args: &[&str]) -> Result<u8, String> {
    if name == "pre-push" {
        return pre_push(args);
    }
    if name == "reference-transaction" {
        return reference_transaction(args);
    }
    let Some(&(_, script, decide)) = HOOKS.iter().find(|(known, _, _)| *known == name) else {
        return Err(format!(
            "no hook '{name}' is known to this nuc — a newer nuc may know it: {}",
            config::INSTALL
        ));
    };
    if !args.is_empty() {
        return Err(format!("nuc hook {name} takes no arguments"));
    }
    let dir =
        env::current_dir().map_err(|e| format!("the current directory is unreadable: {e}"))?;
    same_git_dir(&dir)?;
    let root = work_tree(&dir)?;
    let on = turned_on(root.as_deref())?;
    let state = hook::Commit {
        branch: git::probe(&dir, &["symbolic-ref", "--short", "-q", "HEAD"])?
            .map(|branch| branch.trim_end_matches(['\n', '\r']).to_owned()),
        merging: git::probe(&dir, &["rev-parse", "-q", "--verify", "MERGE_HEAD"])?.is_some(),
        // A test of existence, which reads nothing (Q-12).
        check_script: root
            .as_ref()
            .is_some_and(|root| root.join(script).is_file()),
    };
    let on = |rule: &HookRule| on.iter().any(|on| on.id == rule.id);
    Ok(carry_out(decide(&state, &on)))
}

/// The exit code of an answer, its finding printed on the standard output.
fn carry_out(answer: Answer) -> u8 {
    match answer {
        Answer::Pass => PASSED,
        Answer::Refuse(finding) => {
            println!("{finding}");
            FOUND
        }
        Answer::RunCheck => RUN_CHECK,
    }
}

/// The file in the git directory of the worktree that carries the tree `pre-push` decided to its
/// `--record` call, and the record of green trees in the common git directory, as the bash keeps it.
const PENDING: &str = "checks-pending-tree";
const RECORD: &str = "checks-green-trees";

/// `nuc hook pre-push [<remote> [<url>]]`, with git's refs on the standard input, or
/// `nuc hook pre-push --record` after the checks passed.
fn pre_push(args: &[&str]) -> Result<u8, String> {
    let record = match args {
        ["--record"] => true,
        [] | [_] | [_, _] if !args.iter().any(|arg| arg.starts_with("--")) => false,
        _ => {
            return Err(
                "usage: nuc hook pre-push [<remote> [<url>]] | nuc hook pre-push --record"
                    .to_owned(),
            );
        }
    };
    let dir =
        env::current_dir().map_err(|e| format!("the current directory is unreadable: {e}"))?;
    same_git_dir(&dir)?;
    let git_path = |which: &str| {
        git::run(&dir, &["rev-parse", "--path-format=absolute", which])
            .map(|out| PathBuf::from(out.trim_end_matches(['\n', '\r'])))
    };
    let pending = git_path("--git-dir")?.join(PENDING);
    let green = git_path("--git-common-dir")?.join(RECORD);
    if record {
        return record_pending(&pending, &green);
    }
    let pushed = hook::pushed_refs(&read_stdin()?)?;
    let root = work_tree(&dir)?;
    let on = turned_on(root.as_deref())?;
    let mut refs = Vec::new();
    for pushed in &pushed {
        let drafts = pushed
            .local_object
            .as_deref()
            .map(|object| drafts(&dir, object));
        refs.push((pushed.remote_ref.clone(), drafts));
    }
    let tree = match &root {
        Some(_) if git::run(&dir, &["status", "--porcelain"])?.is_empty() => {
            git::probe(&dir, &["rev-parse", "-q", "--verify", "HEAD^{tree}"])?
                .map(|tree| tree.trim_end_matches(['\n', '\r']).to_owned())
        }
        _ => None,
    };
    let recorded = match &tree {
        Some(tree) => read_in_git_dir(&green)?.is_some_and(|text| hook::holds(&text, tree)),
        None => false,
    };
    // The README is read only for a notice, and a README that cannot be read words it as the
    // newest release: a hook refuses nothing for the release (ADR-0023).
    let notice = refs.iter().any(|(_, drafts)| matches!(drafts, Some(1..)));
    let state = hook::Push {
        contributing: !notice || contributing(root.as_deref()).unwrap_or(true),
        refs,
        tree,
        recorded,
        // A test of existence, which reads nothing (Q-12).
        check_script: root
            .as_ref()
            .is_some_and(|root| root.join("scripts/check-all.sh").is_file()),
    };
    let on = |rule: &HookRule| on.iter().any(|on| on.id == rule.id);
    let out = hook::pre_push(&state, &on);
    for notice in &out.notices {
        eprintln!("{notice}");
    }
    if out.answer == Answer::RunCheck {
        // The tree decided now is the one `--record` records, whatever the checks do meanwhile, and
        // only for the run of the shim that wrote it.
        match &state.tree {
            Some(tree) => fs::write(&pending, format!("{tree} {}\n", shim_run()))
                .map_err(|e| format!("{}: unwritable — {e} — not decided", pending.display()))?,
            None => remove(&pending)?,
        }
    }
    Ok(carry_out(out.answer))
}

/// The standard input git hands the hook, which reads no file.
fn read_stdin() -> Result<String, String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| format!("the standard input of the hook is unreadable: {e} — not decided"))?;
    Ok(input)
}

/// `nuc hook reference-transaction <state>`, with the updates on the standard input: in the
/// `prepared` state, where a refusal aborts the transaction, a move of the local `main` only to a
/// commit `origin/main` has as the transaction leaves it; every other state passes.
fn reference_transaction(args: &[&str]) -> Result<u8, String> {
    let [state] = args else {
        return Err("usage: nuc hook reference-transaction <state>".to_owned());
    };
    if *state != "prepared" {
        return Ok(PASSED);
    }
    let updates = hook::ref_updates(&read_stdin()?)?;
    let dir =
        env::current_dir().map_err(|e| format!("the current directory is unreadable: {e}"))?;
    same_git_dir(&dir)?;
    // Git runs a hook at the root of the work tree, or in the git directory of a bare repository
    // (githooks(5)), so `nuc.toml` is read there without asking git: this hook runs on every
    // update of a ref, and each git call costs a process (Q-5).
    let on = turned_on(Some(&dir))?;
    // A transaction that touches no main moves none, and needs no more of git.
    if !updates
        .iter()
        .any(|update| update.reference == "refs/heads/main")
    {
        return Ok(PASSED);
    }
    let refs = git::run(
        &dir,
        &[
            "for-each-ref",
            "--format=%(refname) %(objectname)",
            "refs/heads/main",
            "refs/remotes/origin/main",
        ],
    )?;
    let object = |name: &str| {
        refs.lines()
            .find_map(|line| line.strip_prefix(name)?.strip_prefix(' '))
            .map(str::to_owned)
    };
    // Without origin/main, a repository that has no remote's main yet, nothing is decided.
    let Some(remote) = hook::remote_main(&updates, object("refs/remotes/origin/main")) else {
        return Ok(PASSED);
    };
    let current = object("refs/heads/main");
    let moves = hook::moves_of_main(&updates, current.as_deref());
    // A commit origin/main has: one it is or descends from. Where git cannot tell, it has none,
    // as the bash reads a failing merge-base.
    let has = |commit: &str| {
        git::probe(&dir, &["merge-base", "--is-ancestor", commit, &remote])
            .is_ok_and(|answer| answer.is_some())
    };
    let on = |rule: &HookRule| on.iter().any(|on| on.id == rule.id);
    Ok(carry_out(hook::reference_transaction(
        &moves,
        current.as_deref(),
        &has,
        &on,
    )))
}

/// The count of draft ADRs the object carries, as the bash counts the names `git ls-tree` lists:
/// none where git cannot list them, since the notice decides nothing.
fn drafts(dir: &Path, object: &str) -> usize {
    git::run(dir, &["ls-tree", "--name-only", object, "docs/adr/"])
        .map(|names| {
            names
                .lines()
                .filter(|name| {
                    name.strip_prefix("docs/adr/draft-")
                        .is_some_and(|rest| rest.ends_with(".md"))
                })
                .count()
        })
        .unwrap_or(0)
}

/// Whether the draft notice names `CONTRIBUTING.md`, as the README's release line places it: from
/// v0.7.0 on, and where no release is named, as the newest; a release before v0.5.0 is decided as
/// v0.5.0 (ADR-0023).
fn contributing(root: Option<&Path>) -> Result<bool, String> {
    let Some(root) = root else {
        return Ok(true);
    };
    let tree = Tree::new(root.to_path_buf())?;
    let readme = tree.read_checked(&root.join("README.md"))?;
    Ok(
        match readme
            .as_deref()
            .and_then(rules::docs_template_release::carried)
        {
            Some(rules::Carried::Release(release)) => release >= rules::CONTRIBUTING_GUIDE,
            _ => true,
        },
    )
}

/// The run of the shim that calls `nuc hook pre-push` and then its `--record`, as the shim names
/// it in `NUC_HOOK_RUN`: two pushes from one worktree at once each record only their own tree.
fn shim_run() -> String {
    env::var("NUC_HOOK_RUN").unwrap_or_default()
}

/// `--record`: the pending tree moved into the record of green trees; nothing without one, and
/// not decided where another run's push replaced it meanwhile, whose checks are not these.
fn record_pending(pending: &Path, green: &Path) -> Result<u8, String> {
    let Some(text) = read_in_git_dir(pending)? else {
        return Ok(PASSED);
    };
    let line = text.trim_end_matches(['\n', '\r']);
    let (tree, run_of) = line.split_once(' ').unwrap_or((line, ""));
    if run_of != shim_run() {
        return Err(format!(
            "{}: another push from this worktree decided its tree meanwhile, so this one is not \
             recorded — not decided",
            pending.display()
        ));
    }
    let existing = read_in_git_dir(green)?.unwrap_or_default();
    let next = green.with_extension(format!("{}", std::process::id()));
    if let Err(e) =
        fs::write(&next, hook::record(&existing, tree)).and_then(|()| fs::rename(&next, green))
    {
        let _ = fs::remove_file(&next);
        return Err(format!(
            "{}: unwritable — {e} — not decided",
            green.display()
        ));
    }
    remove(pending)?;
    Ok(PASSED)
}

/// The text of a file `nuc` keeps in the git directory git names, none where it is no file, as
/// the bash's `-f` finds it. It reads only the two files of the green-tree record, whose paths git
/// gives.
fn read_in_git_dir(path: &Path) -> Result<Option<String>, String> {
    // A test of existence, which reads nothing (Q-12).
    if !path.is_file() {
        return Ok(None);
    }
    match fs::read(path) {
        Ok(bytes) => String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| format!("{}: unreadable — not UTF-8 — not decided", path.display())),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!(
            "{}: unreadable — {e} — not decided",
            path.display()
        )),
    }
}

/// The file removed, where it exists.
fn remove(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(format!(
            "{}: cannot be removed — {e} — not decided",
            path.display()
        )),
        _ => Ok(()),
    }
}

/// Not decided where git hands the hook a git directory, as `git --git-dir` and a linked worktree
/// do, other than the one `nuc` finds from the directory it runs in: `nuc` takes the repository
/// never from the environment (ADR-0011), so it would decide on another one, and a hook that
/// cannot tell never passes (Q-1). The handed directory is only named to git, to be spelled as
/// git spells the one it finds.
fn same_git_dir(dir: &Path) -> Result<(), String> {
    let Some(handed) = env::var_os("GIT_DIR") else {
        return Ok(());
    };
    let handed = handed.into_string().map_err(|_| {
        "git hands the hook a git directory that is no UTF-8 — not decided".to_owned()
    })?;
    let spelled = |args: &[&str]| {
        git::run(dir, args).map(|out| out.trim_end_matches(['\n', '\r']).to_owned())
    };
    let found = spelled(&["rev-parse", "--absolute-git-dir"]);
    let named = spelled(&["--git-dir", &handed, "rev-parse", "--absolute-git-dir"]);
    match (found, named) {
        (Ok(found), Ok(named)) if found == named => Ok(()),
        (found, named) => Err(format!(
            "git hands the hook the git directory {}, and nuc finds {} from {} — nuc reads no \
             repository from the environment, so it cannot decide this one — not decided",
            named.unwrap_or(handed),
            found.unwrap_or_else(|_| "none".to_owned()),
            dir.display()
        )),
    }
}

/// The root of the work tree git runs the hook in, none where the repository has no work tree.
fn work_tree(dir: &Path) -> Result<Option<PathBuf>, String> {
    let inside = git::run(dir, &["rev-parse", "--is-inside-work-tree"])?;
    if inside.trim_end() != "true" {
        return Ok(None);
    }
    git::toplevel(dir).map(Some)
}

/// The hook rules turned on: every one without `nuc.toml`; with it, those its list names, after
/// it is read as `nuc check` reads it, fail closed (ADR-0023).
fn turned_on(root: Option<&Path>) -> Result<Vec<&'static HookRule>, String> {
    let Some(root) = root else {
        return Ok(hook::ALL.iter().collect());
    };
    let tree = Tree::new(root.to_path_buf())?;
    let Some(text) = tree.read(&root.join("nuc.toml"))? else {
        return Ok(hook::ALL.iter().collect());
    };
    let config = config::load(&text, running()).map_err(|refusal| refusal.to_string())?;
    if let Some(unknown) = config
        .rules
        .iter()
        .find(|id| rules::find(id).is_none() && hook::find(id).is_none())
    {
        return Err(Refusal::UnknownRule(unknown.clone()).to_string());
    }
    Ok(hook::ALL
        .iter()
        .filter(|rule| config.rules.iter().any(|id| id == rule.id))
        .collect())
}
