//! `nuc hook <name>`: the git state a hook decides on, read through git, and the answer the
//! reference shim acts on (ADR-0021). It runs no command but git and no script of the repository
//! (Q-7): where a hook's check runs, it exits 3 and the shim runs it.

use std::env;
use std::path::{Path, PathBuf};

use crate::config::{self, Refusal};
use crate::hook::{self, Answer, HookRule};
use crate::rules;

use super::tree::Tree;
use super::{FOUND, PASSED, RUN_CHECK, git, running};

/// The hooks this `nuc` decides, by the name git gives them.
const HOOKS: &[&str] = &["pre-commit"];

/// `nuc hook <name>`, run in the directory git runs the hook in.
pub(super) fn run(name: &str, args: &[&str]) -> Result<u8, String> {
    if !HOOKS.contains(&name) {
        return Err(format!(
            "no hook '{name}' is known to this nuc — a newer nuc may know it: {}",
            config::INSTALL
        ));
    }
    if !args.is_empty() {
        return Err(format!("nuc hook {name} takes no arguments"));
    }
    let dir =
        env::current_dir().map_err(|e| format!("the current directory is unreadable: {e}"))?;
    let root = work_tree(&dir)?;
    let on = turned_on(root.as_deref())?;
    let state = hook::Commit {
        branch: git::probe(&dir, &["symbolic-ref", "--short", "-q", "HEAD"])?
            .map(|branch| branch.trim_end_matches(['\n', '\r']).to_owned()),
        merging: git::probe(&dir, &["rev-parse", "-q", "--verify", "MERGE_HEAD"])?.is_some(),
        // A test of existence, which reads nothing (Q-12).
        check_script: root
            .as_ref()
            .is_some_and(|root| root.join("scripts/check-all.sh").is_file()),
    };
    let answer = hook::pre_commit(&state, &|rule: &HookRule| {
        on.iter().any(|on| on.id == rule.id)
    });
    Ok(match answer {
        Answer::Pass => PASSED,
        Answer::Refuse(finding) => {
            println!("{finding}");
            FOUND
        }
        Answer::RunCheck => RUN_CHECK,
    })
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
