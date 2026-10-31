//! The imperative shell: the only module that spawns processes, touches the file system, reads
//! the environment, or prints (ADR-draft-functional-core). It reads the command line and the
//! repository, calls the rules, and reports what they return.

pub mod git;

use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::ExitCode;

use crate::config::{self, Version};
use crate::finding::Finding;
use crate::rules::{self, Inputs};

/// Exit code of a run that found nothing.
const PASSED: u8 = 0;
/// Exit code of a check that reported findings.
const FOUND: u8 = 1;
/// Exit code of a run that could not decide, whatever the cause: it never passes (Q-1).
const UNDECIDED: u8 = 2;

const USAGE: &str = "usage: nuc check [--repo <path>] | nuc explain <rule ID> | nuc --version";

/// The entry point of the binary: runs the command named on the command line and returns the
/// exit code the process ends with.
pub fn main() -> ExitCode {
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
        _ => Err(USAGE.to_owned()),
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
    let text = read(&root.join("nuc.toml"))?.ok_or_else(|| config::Refusal::Missing.to_string())?;
    let config = config::parse(&text).map_err(|refusal| refusal.to_string())?;
    let selected = config::select(&config, running(), rules::ALL).map_err(|r| r.to_string())?;
    let inputs = Inputs {
        readme: read_checked(&root.join("README.md"))?,
    };
    let findings: Vec<Finding> = selected
        .iter()
        .flat_map(|rule| (rule.check)(&inputs))
        .collect();
    for finding in &findings {
        println!("{finding}");
    }
    if findings.is_empty() {
        eprintln!("nuc check passed ({} rules).", selected.len());
        Ok(PASSED)
    } else {
        eprintln!("nuc check: {} finding(s).", findings.len());
        Ok(FOUND)
    }
}

/// `nuc explain <id>`: the reason of the rule (G-6).
fn explain(id: &str) -> Result<u8, String> {
    let rule = rules::find(id).ok_or_else(|| format!("no rule '{id}' is known to this nuc"))?;
    println!("{}: {}", rule.id, rule.explain);
    Ok(PASSED)
}

/// The bytes of the file, `None` when it does not exist, or why it cannot be read.
fn read_bytes(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: unreadable — {e}", path.display())),
    }
}

/// The text of a file `nuc` itself reads, which must be UTF-8.
fn read(path: &Path) -> Result<Option<String>, String> {
    read_bytes(path)?
        .map(|bytes| {
            String::from_utf8(bytes)
                .map_err(|_| format!("{}: unreadable — not UTF-8", path.display()))
        })
        .transpose()
}

/// The text of a file the rules read as the bash checks do, byte by byte: a byte that is no
/// UTF-8 stands for itself, as for grep in the C locale. A NUL byte makes grep read the file as
/// binary, which it reports or not depending on where the byte falls; `nuc` does not decide
/// such a file (Q-1).
fn read_checked(path: &Path) -> Result<Option<String>, String> {
    read_bytes(path)?
        .map(|bytes| {
            if bytes.contains(&0) {
                return Err(format!(
                    "{}: holds a NUL byte, which the bash checks read as binary — not decided",
                    path.display()
                ));
            }
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        })
        .transpose()
}
