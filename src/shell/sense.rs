//! `nuc sense <name>`: the report of a sensor of the template, as the bash sensor of the tree's
//! release writes it (API-git-only-sensors-under-sense). The shell reads `nuc.toml` as `nuc check`
//! does, the README's release line, and what the sensor needs from the tree and from git; the
//! sensor decides; the shell writes the report on the standard output and exits 0, or ends the
//! run not decided with the sensor's key (API-command-line-with-sense).

use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::config;
use crate::sense::{self, Refusal, revision_due};

use super::tree::Tree;
use super::{PASSED, git};

/// `nuc sense <name> [--repo <path>]`: the report of the sensor on the repository that contains
/// `path`, or the directory `nuc` runs in. `nuc.toml` is judged before the name, so that a `nuc`
/// too old for the project says so whatever sensor it is asked for (G-5).
pub(super) fn run(name: &str, options: &[&str]) -> Result<u8, String> {
    let repo = match options {
        [] => None,
        ["--repo", path] => Some(PathBuf::from(path)),
        _ => return Err(super::usage()),
    };
    let start = match repo {
        Some(repo) => repo,
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
    config::read(&text, super::running()).map_err(|refusal| refusal.to_string())?;
    let Some(sensor) = sense::find(name) else {
        return Err(format!(
            "{name}: no sensor nuc ports — it ports {} — not decided [{}]",
            sense::ALL
                .iter()
                .map(|sensor| sensor.name)
                .collect::<Vec<_>>()
                .join(", "),
            sense::UNKNOWN
        ));
    };
    let carried = super::template_release(&tree.root)?;
    let release = sense::release(sensor, carried).map_err(|refusal| refusal.to_string())?;
    let report = match sensor.name {
        "revision-due" => revision_due(&tree, release),
        _ => unreachable!("every sensor of sense::ALL has its reader"),
    }
    .map_err(|refusal| refusal.to_string())?;
    io::stdout()
        .write_all(&report)
        .and_then(|()| io::stdout().flush())
        .map_err(|e| format!("the report could not be written: {e}"))?;
    Ok(PASSED)
}

/// What `revision-due` needs: the overview where it is a file, the commit that set the date, the
/// count of the changes outside `docs/` since, and where the revision is due, the entries they
/// touched that still exist; git asked as the bash asks it.
fn revision_due(tree: &Tree, release: config::Version) -> Result<Vec<u8>, Refusal> {
    let refuse = |reason| {
        move |message: String| Refusal {
            sensor: revision_due::SENSOR.name,
            reason,
            message: unwrapped(message),
        }
    };
    let undecided = refuse("git");
    let overview_path = tree.root.join(revision_due::OVERVIEW);
    let overview = if overview_path.is_file() {
        tree.read_checked(&overview_path)
            .map_err(refuse("file"))?
            .map(String::into_bytes)
    } else {
        None
    };
    let line = revision_due::read(overview.as_deref(), release)?;
    let root = &tree.root;
    let head = git::probe(root, &["rev-parse", "--verify", "-q", "HEAD"]).map_err(undecided)?;
    let range = match (&head, &line.date) {
        (None, _) => None,
        (Some(_), None) => Some("HEAD".to_owned()),
        (Some(_), Some(date)) => {
            let pattern = revision_due::anchor_pattern(date, release);
            let pattern = String::from_utf8(pattern).expect("the date is ASCII");
            // No textconv driver of the repository runs on the diffs `-G` reads (Q-7): the bash's
            // git would run one, so on a repository that sets one for the overview the two may
            // find another commit.
            let log = git::run(
                root,
                &[
                    "log",
                    "--no-textconv",
                    "--reverse",
                    "--format=%H",
                    &format!("-G{pattern}"),
                    "HEAD",
                    "--",
                    revision_due::OVERVIEW,
                ],
            )
            .map_err(undecided)?;
            log.lines()
                .next()
                .filter(|anchor| !anchor.is_empty())
                .map(|anchor| format!("{anchor}..HEAD"))
        }
    };
    let Some(range) = range else {
        return Ok(revision_due::report(&line, 0, &[], release));
    };
    let outside_docs = ["--", ".", ":(exclude)docs"];
    let mut args = vec!["rev-list", "--count", "--no-merges", range.as_str()];
    args.extend(outside_docs);
    let count = git::run(root, &args).map_err(undecided)?;
    let count: u64 = count
        .trim_end()
        .parse()
        .map_err(|_| undecided(format!("git rev-list --count wrote '{}'", count.trim_end())))?;
    if !revision_due::is_due(&line, count) {
        return Ok(revision_due::report(&line, count, &[], release));
    }
    let mut args = vec![
        "log",
        "--no-merges",
        "--name-only",
        "--format=%x01",
        range.as_str(),
    ];
    args.extend(outside_docs);
    let log = git::run_bytes(root, &args).map_err(undecided)?;
    let scopes: Vec<_> = revision_due::scopes(&log)
        .into_iter()
        .filter(|(_, entry)| exists(root, revision_due::scope_path(entry)))
        .collect();
    Ok(revision_due::report(&line, count, &scopes, release))
}

/// The message of a refusal the tree or git gave, without the words `Refusal` adds itself.
fn unwrapped(message: String) -> String {
    message
        .strip_suffix(" — not decided")
        .map(str::to_owned)
        .unwrap_or(message)
}

/// Whether the path under the root exists, as the bash's `-e` tests it: it follows a symbolic link
/// and reads nothing (Q-12).
fn exists(root: &Path, relative: &[u8]) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let mut path = root.as_os_str().to_owned();
        path.push("/");
        path.push(std::ffi::OsStr::from_bytes(relative));
        Path::new(&path).exists()
    }
    #[cfg(not(unix))]
    {
        std::str::from_utf8(relative)
            .map(|relative| Path::new(&format!("{}/{relative}", root.display())).exists())
            .unwrap_or(false)
    }
}
